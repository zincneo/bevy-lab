# 003：Windows、Linux 与 Android 打包

本节只写文档，不提供单独的可运行示例。打包不是某个 ECS 系统可以在运行时演示的功能，
而是把 Rust 可执行文件、平台运行时、资源目录和平台配置组合成最终发行物的过程。

Bevy 负责生成应用本身和运行时资源读取逻辑，但不会替我们生成 Windows 安装向导、Linux
发行版安装包或 Android 签名 APK。通常需要先用 Cargo 编译，再用对应平台的打包工具组织
发行目录。

## 一、所有平台都遵循的资源原则

开发目录通常是：

```text
my-game/
├── Cargo.toml
├── src/
├── assets/
│   ├── images/
│   ├── audio/
│   └── fonts/
└── target/
```

`AssetPlugin` 默认把 `assets` 作为默认资源源。代码中的资源路径只写资源根目录下面的相对
路径：

```rust
asset_server.load::<Image>("images/player.png");
```

不要把开发机的绝对路径写进代码，也不要把 `assets/` 重复写成：

```rust
// 错误示例：会尝试寻找 assets/assets/images/player.png
asset_server.load::<Image>("assets/images/player.png");
```

打包时必须保证 `images/player.png` 在目标平台的资源源中仍然处于相同的相对位置。不同
平台的区别主要在于“资源源放在哪里”：

| 平台 | Bevy 读取资源的常见来源 | 安装包中的典型位置 |
| --- | --- | --- |
| Windows | 文件系统 `FileAssetReader` | 可执行文件旁的 `assets/` |
| Linux | 文件系统 `FileAssetReader` | 可执行文件旁的 `assets/` 或安装前缀下的 `share/<app>/assets/` |
| Android | APK 内的 Android `AssetManager` | APK 的 `assets/` 内容树 |

资源文件不会因为执行 `cargo build --release` 就自动复制到 `target/release`。发行流程必须
明确地复制或嵌入它们。

## 二、准备发布构建

发布构建应该使用 `--release`：

```bash
nix develop
cargo build --release
```

在本项目中，Bevy 依赖 GitHub 上的 `main` 分支，因此每次更新依赖后都应该重新检查目标平台
的编译结果。发布版本通常还会调整 Cargo profile，例如减小调试信息、启用 LTO 或减少代码
生成单元；这些优化会影响构建时间和最终体积，应在独立的发布配置中逐项验证。

不要把 `dynamic_linking` 作为最终发行方案的默认设置。动态链接可以缩短开发期编译时间，
但发行时还要额外分发对应的 Bevy 动态库，并且会影响优化方式。最终包更常见的是不启用
动态链接，让可执行文件包含所需的 Rust 和 Bevy 代码。

## 三、Windows 打包

### 1. 编译 Windows 可执行文件

在 Windows 开发机中，最直接的命令是：

```powershell
cargo build --release
```

生成的程序通常在：

```text
target\release\my-game.exe
```

如果在其它主机交叉编译，需要额外准备 Windows Rust target、链接器和对应的系统库；这不
是 Bevy 或 `AssetPlugin` 自动提供的能力。初次学习时，优先在目标平台本机构建，能够更快
排除图形驱动、链接器和运行库差异。

### 2. 组织可运行目录

最简单的 Windows 发行目录是：

```text
my-game-windows/
├── my-game.exe
└── assets/
    ├── images/
    │   └── player.png
    ├── audio/
    │   └── click.ogg
    └── fonts/
        └── game.ttf
```

安装包或 ZIP 至少要把 `.exe` 和整个 `assets/` 目录一起放进去。用户启动程序时，当前工作
目录不一定是 `.exe` 所在目录，所以发布版最好在创建 `App` 前把资源根目录解析为程序目录下
的 `assets`，而不是依赖启动快捷方式的工作目录：

```rust
use std::{env, path::PathBuf};

fn packaged_asset_root() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join("assets")))
        .unwrap_or_else(|| PathBuf::from("assets"))
}

let asset_root = packaged_asset_root();
App::new()
    .add_plugins(DefaultPlugins.set(AssetPlugin {
        file_path: asset_root.to_string_lossy().into_owned(),
        ..default()
    }))
    .run();
```

这段逻辑只适用于外部资源目录。开发时仍然可以使用相对的 `assets`，或者通过构建配置在
开发和发布模式之间切换。

### 3. 制作 Windows 安装包

Bevy 不内置安装向导。可以选择：

- 直接发布包含 `.exe` 和 `assets/` 的 ZIP；
- 使用 WiX、NSIS、Inno Setup 等 Windows 安装工具，把整个发行目录复制到安装位置；
- 在 CI 中先运行 `cargo build --release`，再把安装工具的输出作为发布产物。

安装工具的职责是复制文件、创建快捷方式和卸载信息；它不应该改变 `assets/` 内部的相对
目录，否则 `AssetServer` 的路径将找不到资源。若程序依赖额外的系统运行库，也应在安装包
中声明或安装对应运行库。

## 四、Linux 打包

### 1. 编译 Linux 可执行文件

在 Linux 环境中：

```bash
cargo build --release
```

常见输出为：

```text
target/release/my-game
```

可以先用一个可移动目录验证：

```text
my-game-linux/
├── my-game
└── assets/
    ├── images/
    ├── audio/
    └── fonts/
```

确认从不同工作目录启动仍然能找到资源后，再选择发行格式。

### 2. 常见 Linux 发行形式

Linux 没有一个由 Bevy 统一规定的安装包格式，常见选择包括：

- **压缩包**：适合内部测试和独立分发，解压后直接运行；
- **AppImage**：把程序和资源组织成一个可执行镜像，适合不依赖系统包管理器的分发；
- **Flatpak**：把应用、资源和运行时权限放进沙盒，适合桌面应用商店分发；
- **发行版包**：例如 Debian/Ubuntu 的 `.deb` 或 Fedora 的 `.rpm`，由包管理器安装到系统
 目录。

这些格式都需要把资源作为安装内容。若采用系统目录布局，常见结构是：

```text
/usr/bin/my-game
/usr/share/my-game/assets/images/player.png
/usr/share/my-game/assets/audio/click.ogg
```

此时应把 `AssetPlugin::file_path` 配置为实际安装的资源目录，或者在启动时根据 Linux 的
安装前缀计算资源根目录。不要假设 `/usr/share` 在开发机上存在。

### 3. Linux 图形和动态库

`cargo build --release` 只负责 Rust 依赖和程序本身。发行包还要考虑目标机器上的图形驱动、
窗口系统和音频后端。AppImage、Flatpak 或发行版包可以分别声明这些依赖，但它们不会改变
Bevy 的资源路径；资源目录仍须和 `AssetPlugin` 的配置一致。

## 五、Android 打包

Android 的流程与 Windows/Linux 不同：Rust 程序先编译为 Android ABI 的共享库，然后由一个
Gradle Android 工程把共享库、Manifest、签名配置和资源组装成 APK 或 AAB。

### 1. 准备 Android 工具链

在 Nix 开发环境之外，还需要准备 Android SDK、NDK、Java/Gradle，以及 Rust Android target。
项目环境变量通常包括：

```bash
rustup target add aarch64-linux-android
cargo install cargo-ndk

export ANDROID_SDK_ROOT=/path/to/android-sdk
export ANDROID_NDK_ROOT=/path/to/android-sdk/ndk/<version>
```

Bevy 当前的官方 Android 示例使用 `GameActivity`，推荐 `cargo-ndk` 生成各 ABI 的共享库，
再用 Gradle 构建 Android 工程。旧的 `cargo-apk` 流程较简单，但不支持当前默认的
`GameActivity`，不应作为新项目的首选。

### 2. 编译 Rust 共享库

以 64 位 ARM 为例：

```bash
cargo ndk \
    -t arm64-v8a \
    -P 26 \
    -o android/app/src/main/jniLibs \
    build --release
```

其中：

- `-t arm64-v8a` 选择 Android ABI；还可以分别构建 `armeabi-v7a`、`x86` 或 `x86_64`；
- `-P 26` 设置 Android 平台 API 级别，应与项目支持的最低版本和 Bevy 依赖兼容；
- `-o .../jniLibs` 让 Gradle 能在对应 ABI 目录找到生成的 `.so` 文件；
- `--release` 使用发行优化。

实际项目通常会为每个要支持的 ABI 重复这一步，或在 CI 中为多个 target 构建。

### 3. 把资源放进 APK

Android 版本的 Bevy 默认资源读取器使用 Android `AssetManager`，而不是 Linux/Windows 的
普通文件系统。因此不能只把 `assets/` 留在 Rust 项目旁边；Gradle 必须把它声明为 APK 的
asset source：

```gradle
android {
    sourceSets {
        main {
            assets.srcDirs += files("../../assets")
        }
    }
}
```

如果源目录的内容是：

```text
assets/
└── images/player.png
```

APK 的 `assets` 内容树中应能直接找到：

```text
images/player.png
```

Rust 代码仍然使用：

```rust
asset_server.load::<Image>("images/player.png");
```

不要把 APK 内部路径写成 `assets/images/player.png`，除非 Gradle 配置确实额外创建了这一层
目录。Android 的 `AssetManager` 会按照 Bevy 资源路径读取打包后的内容。

Android 的 `assets/` 与 `res/` 不是一回事：

- Bevy `AssetServer` 读取游戏资源时使用 `assets/` 内容树；
- `res/` 用于 Android 的图标、主题、布局等资源，需要通过 Android API 或 Manifest 引用；
- 游戏内图片、音频、字体和 glTF 文件通常应放在 `assets/`，不要放到 `res/drawable` 代替
  Bevy 资源路径。

### 4. 构建 APK 或 AAB

Rust 共享库和资源准备好后，在 Android 工程目录执行：

```bash
./gradlew assembleDebug
```

调试 APK 通常位于：

```text
app/build/outputs/apk/debug/app-debug.apk
```

用于发布时，需要配置 Android 签名密钥，再构建签名的 release APK 或 Play 商店使用的 AAB：

```bash
./gradlew assembleRelease
./gradlew bundleRelease
```

没有签名的 debug APK 只适合本地测试，不能作为正式发布包。安装到设备进行测试：

```bash
adb install -r app/build/outputs/apk/debug/app-debug.apk
adb logcat | grep 'RustStdoutStderr\|bevy\|wgpu'
```

### 5. APK 中资源的生命周期

把文件放进 APK 只解决“程序能读取文件”的问题，不代表启动时会把所有文件解码进内存。
Android 仍然遵循 `AssetServer::load` 的异步流程：

1. APK 中存在相对路径文件；
2. 系统调用 `AssetServer::load`；
3. `AndroidAssetReader` 通过 `AssetManager` 读取字节；
4. 对应 `AssetLoader` 异步解码；
5. 资源进入 `Assets<T>` 后，实体才能使用它。

因此可以把大量资源放入 APK，同时只在进入关卡或点击按钮时请求需要的资源。不过 APK 本身
仍然会包含这些文件，按需加载节省的是运行时内存和启动时间，不会减少安装包大小。

## 六、外部资源与嵌入资源

Windows 和 Linux 可以选择两种方案：

### 外部资源目录

```text
my-game/
├── my-game(.exe)
└── assets/
    └── ...
```

优点是开发迭代和热重载方便，也可以独立替换资源；缺点是安装包必须保证整个目录完整复制，
用户也能直接看到和修改资源。

### 嵌入程序或平台包

可以使用 Bevy 的 embedded asset 机制或自定义 `AssetSource`，把资源编译进程序或放进 APK
内部资源树。这样不需要旁边的文件夹，但会增加构建复杂度，资源更新也必须重新构建程序。

Android APK 的 `assets/` 目录属于“随平台包分发的资源”，并不等同于把字节编译进 Rust
二进制；它仍由 Android `AssetManager` 在运行时读取。

## 七、发布前检查清单

- 使用 `cargo build --release`，确认没有依赖开发期动态链接库；
- 目标目录中存在可执行文件或 Android `.so`；
- Windows/Linux 的发行目录包含完整 `assets/`；
- Android Gradle 工程把资源目录加入 `sourceSets.main.assets`；
- 代码中的资源相对路径与安装包内路径完全一致，大小写也一致；
- 使用发布目录的实际启动方式测试，而不是只在项目根目录执行 `cargo run`；
- 对每个资源检查 `is_loaded_with_dependencies` 或相应加载事件，避免运行时才发现路径错误；
- Android release 包使用正式签名，并在目标 ABI 和 Android 版本的设备上测试。

## 八、参考资料

- [Bevy 官方 examples README：Android Setup、Build & Run](https://github.com/bevyengine/bevy/blob/main/examples/README.md#android)
- [Bevy 官方 Android 示例的 Gradle 资源配置](https://github.com/bevyengine/bevy/blob/main/examples/mobile/android_example/app/build.gradle)
- [Bevy AndroidAssetReader 源码](https://github.com/bevyengine/bevy/blob/main/crates/bevy_asset/src/io/android.rs)
