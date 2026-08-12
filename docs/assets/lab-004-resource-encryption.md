# 004 resource-encryption：加密资源的加载与发布

本节讨论一个常见的发布需求：源码仓库中可以保留便于调试的 `png`，但发布包中不希望用户直接看到可以双击打开的原始图片，而是只携带加密后的资源文件。程序在真正需要该资源时读取密文，验证并解密，再交给 Bevy 的资源系统和渲染系统使用。

这是一篇设计说明，不提供可直接运行的 lab。资源加密通常需要一个离线打包工具、一个自定义 `AssetLoader`、密钥管理策略和发布脚本，单个 Bevy 示例无法完整演示这些环节。

## 先明确安全边界

资源加密主要解决的是“不要让用户轻易从安装目录中拿到原始文件”，例如：

- 安装包中没有可以直接打开的 `player.png`；
- 用户修改或替换密文后，程序能够检测到文件被篡改；
- 资源包可以统一使用自己的文件格式和校验规则；
- 发布包和开发目录可以使用不同的资源组织方式。

它不能保证资源永远不会被提取。一个离线运行、又必须在本地显示图片的程序，运行时最终一定会拥有解密密钥和明文数据。攻击者仍然可能：

1. 逆向程序，找到密钥或密钥派生过程；
2. 在解密函数返回之后转储进程内存；
3. 在图片上传到 GPU 后，从渲染资源或显存中提取内容；
4. 直接修改程序，让它跳过校验或保存解密结果。

因此这里的目标应当表述为“提高资源提取和篡改的成本”，而不是“让资源无法破解”。如果资源必须对授权用户保密，密钥应当由服务器或操作系统安全存储提供，而不能只依赖一个写在客户端里的常量。

## Bevy 中应该放在哪一层处理

`AssetPlugin` 负责建立资源系统的基础设施，例如资源根目录、资源读取、资源加载任务和资源事件；它本身不会把任意文件自动解密成 `Image`。普通 `png` 之所以能够工作，是因为默认插件已经注册了对应的图片加载器。

对于单个加密文件，最合适的扩展点通常是自定义 `AssetLoader`：

```text
AssetServer.load("images/player.enc")
        │
        ├─ 根据 .enc 找到 EncryptedImageLoader
        ├─ 读取密文文件
        ├─ 校验文件头、版本、nonce 和认证标签
        ├─ 使用密钥进行 AEAD 解密
        ├─ 将明文 PNG/JPEG 解码为 Bevy Image
        └─ 把 Image 放入 Assets<Image>，返回 Handle<Image>
```

这样 UI 的 `ImageNode`、2D 的 `Sprite` 或 BSN 中引用图片的代码都可以继续使用 `Handle<Image>`。它们只关心资源句柄，不需要知道磁盘上实际是 `.png` 还是 `.enc`。

Bevy 官方的自定义资源示例也是通过实现 `AssetLoader`，读取字节并返回自定义资产类型：[custom asset loader example](https://github.com/bevyengine/bevy/blob/main/examples/asset/custom_asset.rs)。加密图片只是在读取字节后多了一步“认证并解密”，最终资产类型仍然可以是 `Image`。

### 为什么不优先改写 `AssetReader`

`AssetReader` 更接近“从哪里取得原始字节”，适合把整个资源来源替换成压缩包、内存映射文件、远程存储或一个整体加密的资源容器。

如果只是让每个图片文件变成一个独立的 `.enc` 文件，使用自定义 `AssetLoader` 更直观：

- `.enc` 扩展名可以直接把文件路由到加密加载器；
- 加载器可以在解密前校验自己的文件头和版本；
- 不会影响其它仍然使用默认加载器的资源；
- 资源解密和图片解码位于同一个异步加载流程中。

只有在设计“一个加密归档包含大量资源”的格式时，才有必要进一步考虑自定义 `AssetReader` 或归档层。

## Bevy 有没有类似 Unity AssetBundle 的默认方案

简短答案：**当前 Bevy 没有默认提供一个 Unity AssetBundle 式的“遍历整个 `assets/`、生成单个归档文件、自动压缩并加密、运行时自动读取”的完整方案。**

Bevy 提供的是几个可以组合的基础能力，而不是固定的资源打包格式：

| Bevy 能力 | 解决的问题 | 是否自动生成一个大文件 | 是否自动加密 |
| --- | --- | --- | --- |
| `AssetMode::Unprocessed` | 直接从资源源读取原始文件 | 否 | 否 |
| `AssetMode::Processed` + `AssetProcessor` | 在开发或构建阶段转换、优化、保存处理后的资源 | 通常仍是每个资源一个输出文件 | 否 |
| `embedded_asset!` | 把资源字节编译进应用程序二进制 | 资源会进入可执行文件，但不是独立的资源包 | 否 |
| 自定义 `AssetLoader` | 把某种文件扩展名解码成 Bevy 资产 | 不负责归档 | 可以在加载器中实现解密 |
| 自定义 `AssetReader` | 从自定义存储中按逻辑路径读取字节 | 可以读取你定义的 `.pak` 或归档 | 可以在读取层实现解密 |

### `AssetProcessor` 不是 AssetBundle

当前 Bevy 的 `AssetProcessor` 更接近“资源导入和预处理管线”：它可以读取源资源、运行 `AssetProcessor`/`AssetTransformer`、再通过 `AssetSaver` 写出处理后的资源。`AssetMode::Processed` 默认会把结果放到类似 `imported_assets/Default` 的处理目录中，而不是把所有资源自动合并成一个容器。

因此可以把它作为打包流程的前置阶段：

```text
assets/                     # 源文件
        │
        ▼ AssetProcessor：转换、优化、生成运行时格式
imported_assets/Default/    # 处理后的独立文件
        │
        ▼ 自己的 pack 工具：建立索引、压缩、加密
game.pak                   # 最终发布资源包
```

Bevy 官方的 [asset processing example](https://github.com/bevyengine/bevy/blob/main/examples/asset/processing/asset_processing.rs) 展示了处理器、转换器和保存器如何组合，但它不是通用的资源归档器。你可以编写自定义 processor 或 saver 生成加密的独立文件；如果目标是单个 `.pak`，仍需要另外设计归档索引和运行时读取逻辑。

### `embedded_asset!` 是“嵌入可执行文件”，不是资源归档

Bevy 也支持通过 `embedded_asset!` 在编译时把字节放进应用程序：

```rust,ignore
embedded_asset!(app, "assets/images/player.png");
let image = asset_server.load("embedded://my_game/assets/images/player.png");
```

这样发布包中可以没有外部的 `player.png`，因为资源已经位于可执行文件中。它适合少量启动资源、内置 shader、默认字体或 loading screen 图片；Bevy 自己也使用这种方式嵌入部分内部资源。参考 [embedded asset example](https://github.com/bevyengine/bevy/blob/main/examples/asset/embedded_asset.rs)。

但它有几个限制：

- 它不提供资源加密；用户仍可以从二进制或运行时内存中提取资源；
- 大量图片会直接增大可执行文件，不方便增量更新和 DLC 分发；
- 不会自动把目录按依赖关系整理成 Unity 那种可独立挂载的 AssetBundle；
- 资源路径和嵌入注册仍由程序代码决定。

因此，`embedded_asset!` 可以解决“外部不要有散落的小文件”，但不能替代“可更新、可索引、可加密的资源包”。

## Unity 式单文件资源包是怎么做的

Unity AssetBundle、许多自研引擎的 `.pak`、`.bundle` 或 `.assets` 文件，本质上都是一个**应用自定义的容器格式**。它们通常包含：

- 文件头和格式版本；
- 逻辑资源路径到数据位置的索引；
- 资源的偏移量、压缩前后大小和校验值；
- 压缩或平台专用的运行时数据；
- 可选的依赖关系和类型信息；
- 一个或多个压缩、加密的数据块。

一个简化的 `.pak` 可以看起来像这样：

```text
┌──────────────────────────────────────────────┐
│ magic / version / index offset / index size   │
├──────────────────────────────────────────────┤
│ index:                                        │
│   images/player.png -> offset, size, flags    │
│   fonts/main.ttf   -> offset, size, flags     │
├──────────────────────────────────────────────┤
│ compressed and/or encrypted payload blocks   │
└──────────────────────────────────────────────┘
```

应用仍然请求逻辑路径：

```text
asset_server.load("images/player.png")
        │
        ▼
自定义 AssetReader 在 game.pak 的索引中查找路径
        │
        ├─ 定位数据块
        ├─ 解密并验证认证标签
        ├─ 解压缩
        └─ 返回 PNG 字节
        │
        ▼
Bevy 根据 .png 选择默认图片 loader
        │
        ▼
Assets<Image>
```

这也是为什么使用自定义 `AssetReader` 后，UI 和 Sprite 的业务代码通常不需要改：它们继续请求原来的逻辑路径，只有底层资源源从文件系统换成了资源包。Bevy 官方的 [custom asset reader example](https://github.com/bevyengine/bevy/blob/main/examples/asset/custom_asset_reader.rs) 展示了如何替换资源读取层；示例中的 reader 只是包装默认 reader，实际项目可以把它改成读取自己的归档索引。

### 归档工具和运行时 reader 的职责

离线 pack 工具负责：

1. 扫描资源目录并规范化相对路径；
2. 让 Bevy 的 `AssetProcessor` 先完成转换和优化（如果项目使用它）；
3. 建立路径索引和依赖信息；
4. 先压缩，再加密每个数据块或整个容器；
5. 写出 `game.pak` 以及必要的版本、哈希或签名信息。

运行时 `AssetReader` 负责：

1. 打开资源包并读取索引；
2. 根据 `AssetServer` 请求的逻辑路径查找条目；
3. 读取对应范围的数据；
4. 解密、认证和解压缩；
5. 将字节返回给现有的 `AssetLoader`。

`zip`、`tar` 等 Rust 库可以用作容器基础，`zstd`、`lz4_flex` 等库可以负责压缩，但它们本身不等于安全的资源保护。加密仍应使用前文介绍的 AEAD，并且要先压缩再加密：加密后的数据近似随机，继续压缩通常没有效果。

## 单个大文件还是多个资源包

把所有资源合成一个文件并不总是最优：

- 只加载一张图片时，如果整个大文件整体加密，可能需要读取或解密大量无关数据；
- 更新一个小资源可能导致整个资源包重新下载；
- 一个资源包损坏可能影响所有资源；
- 未加密的索引会暴露路径和文件大小，加密索引则需要先解密索引才能随机访问。

比较实用的做法是按更新和加载边界拆成多个包，例如：

```text
core.pak       # 启动必需的资源
ui.pak         # UI、字体和图标
level-001.pak  # 某个关卡及其依赖
dlc-forest.pak # 可选内容
```

每个包内部仍然可以使用独立条目和独立 nonce。这样既能得到“安装目录中只有少量资源包”的效果，也保留按场景加载、增量更新和失败隔离的能力。

## 两种加密布局的取舍

### 每个条目独立加密

```text
game.pak
├── index
├── player.png entry: nonce + ciphertext + tag
├── enemy.png entry: nonce + ciphertext + tag
└── font.ttf entry: nonce + ciphertext + tag
```

优点是随机访问简单、只解密当前资源、认证失败范围小。缺点是每个条目需要保存 nonce、认证标签和额外元数据。

### 整个资源包加密

```text
game.pak
├── header
└── encrypted(index + all payloads)
```

实现上可以更简单，但必须考虑索引如何访问，以及加载一张小图片时是否需要解密很大的连续数据。若采用整体加密，通常还要把 payload 分成多个带独立 nonce 和认证标签的块，最后仍会接近“分块加密的资源包”。

对 Bevy 的第一次实现，建议采用“一个包 + 明文或轻量保护的索引 + 每个资源条目独立 AEAD 加密”，再由自定义 `AssetReader` 按路径随机读取。若还需要隐藏文件名，可以加密索引，但要接受启动时读取和解密索引的成本。

## 加密算法：使用带认证的 AEAD

不要使用 XOR、简单字节移位、固定密钥 AES-ECB 或自定义加密算法。这些方式无法可靠地保护内容，通常也无法发现密文被修改。

更合适的是 AEAD（Authenticated Encryption with Associated Data，带附加数据认证的加密）：

- **机密性**：没有密钥不能直接还原明文；
- **完整性和认证**：密文、nonce 或附加数据被修改时，解密失败；
- **关联数据**：可以把相对资源路径、文件类型或版本绑定到认证范围，防止把一个合法密文悄悄换成另一个资源。

Rust 中常见的选择是：

| 库 | 常用类型 | 适合的场景 |
| --- | --- | --- |
| [`aes-gcm`](https://docs.rs/aes-gcm/latest/aes_gcm/) | `Aes256Gcm` | 目标平台有 AES 硬件加速，或团队已经统一使用 AES-GCM |
| [`chacha20poly1305`](https://docs.rs/chacha20poly1305/latest/chacha20poly1305/) | `XChaCha20Poly1305` | 纯软件环境，或者希望使用更大的 nonce 空间来简化多文件 nonce 管理 |

两者都是 AEAD。无论选择哪一个，每个文件都必须使用唯一且不可预测的 nonce；同一个密钥下重复使用 nonce 会破坏安全性。`XChaCha20Poly1305` 使用较大的 nonce，适合离线打包工具为大量文件随机生成 nonce，但它并不会自动替你管理 nonce。

### 一个简单的资源文件封装

加密后的文件不必伪装成 PNG。可以定义自己的二进制封装，例如：

```text
魔数：BEVYENC
格式版本：1
算法标识：XChaCha20-Poly1305
原始格式：png
nonce：每个文件随机生成
密文和认证标签：AEAD 输出
```

实际字节布局可以按项目需要设计，但应满足这些条件：

1. 先检查魔数和版本，再决定如何解析；
2. nonce 可以明文存储，它不是密钥，但必须每个文件唯一；
3. 密文必须包含 AEAD 的认证标签；
4. 资源相对路径可以作为 AAD，防止 `a.enc` 被替换成 `b.enc` 后仍被当作合法资源；
5. 解密和认证成功前，不要把数据交给 PNG/JPEG 解码器。

认证标签不是额外的密码。它是 AEAD 用来判断“这份密文确实由持有密钥的一方生成，且中途没有被修改”的校验结果。

## Rust 库如何分工

加密资源通常不只需要一个密码学原语：

- `aes-gcm` 或 `chacha20poly1305`：负责加密、解密和认证；
- [`argon2`](https://docs.rs/argon2/latest/argon2/)：只有在“用户输入密码，需要从密码派生密钥”时才使用。它是内存困难的密钥派生函数，不是文件加密算法；
- [`secrecy`](https://docs.rs/secrecy/latest/secrecy/)：通过 `SecretBox` 等类型减少密钥被意外打印或复制的机会；
- [`zeroize`](https://docs.rs/zeroize/latest/zeroize/)：在密钥或临时明文不再使用时尽量清零内存。

`secrecy` 和 `zeroize` 可以降低日志、调试输出或普通生命周期管理造成的泄露风险，但它们不能阻止逆向工程，也不能保证操作系统、编译器或 GPU 驱动没有任何数据副本。

对于没有用户密码的离线游戏，通常不需要为了“看起来更安全”而使用 Argon2。项目可以由离线打包工具生成随机数据密钥，并在发布程序中以某种形式提供该密钥；但只要程序能离线解密，熟练的攻击者最终仍可能从程序中找到它。

## Debug 和 Release 使用不同资源

Debug 目录保留原始文件，方便美术调整和快速迭代；Release 目录只放打包器生成的密文。关键点是使用编译配置选择**路径和加载器**，而不是在应用启动后再把原始 PNG 加密一次。

```rust,ignore
#[cfg(debug_assertions)]
const ASSET_ROOT: &str = "assets";

#[cfg(not(debug_assertions))]
const ASSET_ROOT: &str = "assets-packed";

#[cfg(debug_assertions)]
const PLAYER_IMAGE: &str = "images/player.png";

#[cfg(not(debug_assertions))]
const PLAYER_IMAGE: &str = "images/player.enc";
```

启动 App 时可以让 `AssetPlugin` 使用对应的根目录：

```rust,ignore
App::new()
    .add_plugins(DefaultPlugins.set(AssetPlugin {
        file_path: ASSET_ROOT.into(),
        ..default()
    }))
    // Release 构建中还需要注册 EncryptedImageLoader。
    .run();
```

加载图片的业务代码仍然是普通的 Bevy 代码：

```rust,ignore
let image: Handle<Image> = asset_server.load(PLAYER_IMAGE);
```

Debug 版本的 `PLAYER_IMAGE` 会走 Bevy 默认的 PNG loader。Release 版本的 `PLAYER_IMAGE` 会走注册到 `.enc` 扩展名上的 `EncryptedImageLoader`。推荐使用 `#[cfg(debug_assertions)]` 和 `#[cfg(not(debug_assertions))]` 明确分离两条实现；这样可以在 Release 构建中不编译仅供调试的分支，也能避免把原始资源路径误带进发布逻辑。

这里的 `debug_assertions` 是 Cargo 配置的编译条件。通常 Debug 构建开启、Release 构建关闭，但自定义 profile 可以改变这一点；如果项目有特殊的发布 profile，应当在构建脚本中确认它选择了正确的资源目录。

如果使用同一个 `assets/` 目录，也必须在发布前删除或排除所有 `.png`、`.jpg` 等原始文件。更不容易出错的做法是让发布流程生成独立的 `assets-packed/` 目录，并只把它复制进安装包。

## 离线资源打包流程

不要在应用第一次启动时读取原始 PNG 并加密。这样做既不能隐藏安装包中的原始文件，也会增加首次启动时间。

建议单独维护一个不会随最终应用分发的离线工具，例如 `tools/asset-pack` 或 workspace 中的 `xtask`：

```text
assets-src/
└── images/player.png       # 只在开发和打包机上保留

        │  cargo run -p asset-pack --release -- assets-src assets-packed
        ▼

assets-packed/
└── images/player.enc       # Release 安装包只复制这个目录
```

打包器的步骤应当是：

1. 遍历 `assets-src` 中允许发布的文件；
2. 为每个文件生成新的随机 nonce；
3. 把相对路径、原始格式和格式版本放入封装头或 AAD；
4. 使用 AEAD 加密并写出 `.enc` 文件；
5. 检查输出目录中不存在原始扩展名文件；
6. 再执行 `cargo build --release` 和平台安装包制作。

一种更明确的目录组织方式是：

```text
assets-src/                 # 不进入最终安装包
assets/                     # Debug 使用，保留原始文件
assets-packed/              # Release 使用，只放 .enc
tools/asset-pack/           # 离线转换工具
```

如果项目希望源码目录仍然只有一个 `assets/`，也可以让打包器先生成临时目录，再在发布阶段将密文覆盖到 staging 目录；但必须确保 `zip`、安装包制作工具或 Android Gradle 任务拿到的是 staging 目录，而不是源码目录。

## 自定义加密图片加载器的职责

下面是加载器的伪代码，省略了具体错误类型、密钥来源和当前 Bevy 版本中的图片解码函数：

```rust,ignore
struct EncryptedImageLoader;

impl AssetLoader for EncryptedImageLoader {
    type Asset = Image;
    type Settings = ();
    type Error = LoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Image, Self::Error> {
        let encrypted = read_all(reader).await?;
        let envelope = parse_envelope(&encrypted)?;
        let aad = relative_path_as_aad(load_context.path());
        let plaintext = decrypt_and_verify(envelope, key(), aad)?;
        let image = decode_original_image(envelope.format, &plaintext)?;
        Ok(image)
    }

    fn extensions(&self) -> &[&str] {
        &["enc"]
    }
}
```

应用启动时在 Release 版本注册这个 loader：

```rust,ignore
#[cfg(not(debug_assertions))]
app.init_asset_loader::<EncryptedImageLoader>();
```

`AssetServer` 会异步执行加载器，所以解密和图片解码不应写在渲染系统中阻塞主循环。系统只需要保存返回的 `Handle<Image>`，在资源状态变为可用后再创建 `ImageNode` 或 `Sprite`。

解密成功后，明文会暂时存在 CPU 内存中，随后 Bevy 可能把它上传为 GPU 纹理。因此应当在解码完成后尽快释放临时明文缓冲区；这只能减少暴露时间，不能改变“运行时必须有明文”的事实。

## 密钥放在哪里

密钥方案取决于要防护的对象：

| 场景 | 可接受方案 | 需要知道的限制 |
| --- | --- | --- |
| 防止普通用户直接浏览资源 | 把密钥编译进程序，配合 AEAD 和自定义文件格式 | 逆向程序仍可能找到密钥 |
| 需要绑定用户授权 | 登录或许可证服务下发密钥 | 离线运行、断网和缓存策略会变复杂 |
| 移动端本地保护 | 使用系统安全存储保存包装密钥 | Root、调试环境或运行时注入仍可能取得明文 |
| 高价值内容 | 服务端按需下发、分片或使用专门 DRM | 需要完整的服务端和运营体系 |

不要把明文密钥放到：

- `assets/key.txt` 或其它会随安装包分发的文件；
- `config.toml`、环境变量示例或日志；
- 只改了变量名、但仍然能直接搜索到的字符串常量。

这并不意味着“把密钥拆成几段”就能提供强保护。拆分、混淆和运行时拼接最多提高静态搜索成本，不能替代真正的授权服务或平台安全存储。

## 资源替换和篡改检查

AEAD 认证失败时，加载器必须返回错误，不要尝试把失败的字节当作 PNG 继续解码。可以在错误处理中记录资源路径和失败原因，但不要打印密钥、完整密文或明文内容。

如果需要防止资源交换，可以把这些数据放入 AAD：

- 资源相对路径；
- 资源类型，例如 `image/png`；
- 打包格式版本；
- 可选的产品版本或授权标识。

这样即使攻击者拥有两个合法的 `.enc` 文件，把文件内容互换也会因为 AAD 不匹配而加载失败。注意这只能检测交换和篡改，不能阻止攻击者把资源替换为自己重新生成的、同样使用已知密钥的文件。

## 推荐的最小方案

对于第一次实现资源保护，可以按下面的范围控制复杂度：

1. 用一个独立的离线工具把 `assets-src/**/*.png` 转成 `assets-packed/**/*.enc`；
2. 使用 `XChaCha20Poly1305`，每个文件生成随机 nonce，并把相对路径作为 AAD；
3. 自定义一个扩展名为 `.enc`、最终资产类型为 `Image` 的 `AssetLoader`；
4. Debug 使用 `assets/images/player.png` 和默认 PNG loader；
5. Release 使用 `assets-packed/images/player.enc` 和加密图片 loader；
6. 发布前检查安装包中没有原始图片；
7. 测试缺少文件、错误密钥、篡改密文、错误版本和错误路径时都会安全失败。

这个方案可以保持 UI、Sprite 和 BSN 的业务代码不变，同时把“资源文件是什么格式”和“如何解密”限制在资源管线中。后续如果需要整体加密归档、许可证校验或服务器下发密钥，再把资源读取层扩展为自定义 `AssetReader` 或授权服务即可。

## 参考资料

- [Bevy custom asset loader example](https://github.com/bevyengine/bevy/blob/main/examples/asset/custom_asset.rs)
- [Bevy asset processing example](https://github.com/bevyengine/bevy/blob/main/examples/asset/processing/asset_processing.rs)
- [Bevy embedded asset example](https://github.com/bevyengine/bevy/blob/main/examples/asset/embedded_asset.rs)
- [Bevy custom asset reader example](https://github.com/bevyengine/bevy/blob/main/examples/asset/custom_asset_reader.rs)
- [`bevy_asset` 中 `AssetPlugin` 和 `AssetMode`](https://github.com/bevyengine/bevy/blob/main/crates/bevy_asset/src/lib.rs)
- [`aes-gcm` 文档](https://docs.rs/aes-gcm/latest/aes_gcm/)
- [`chacha20poly1305` 文档](https://docs.rs/chacha20poly1305/latest/chacha20poly1305/)
- [`argon2` 文档](https://docs.rs/argon2/latest/argon2/)
- [`secrecy` 文档](https://docs.rs/secrecy/latest/secrecy/)
- [`zeroize` 文档](https://docs.rs/zeroize/latest/zeroize/)
