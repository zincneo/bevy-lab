# 002：AssetPlugin 与资源路径、格式和加载时机

运行示例：

```bash
nix develop
just run assets 002
```

这个示例使用 `DefaultPlugins`，但关闭了主窗口，只在终端输出资源状态。它在 `Startup`
中请求一张图片，却不把图片放进任何实体，随后观察这张图片何时真正进入 `Assets<Image>`。

```text
AssetPlugin::load("images/bevy-icon.png")
        │
        ├── 立即返回 Handle<Image>
        ├── 后台读取和解码
        └── 完成后进入 Assets<Image>
```

## 一、AssetPlugin 负责什么

`AssetPlugin` 是 Bevy 资源系统的基础插件。它主要负责：

- 创建 `AssetServer`，作为应用请求资源的入口；
- 配置默认资源源和资源根目录；
- 根据资源路径找到对应的 `AssetLoader`；
- 启动异步读取、解析和依赖跟踪；
- 将已经加载的资源存放在类型化的 `Assets<T>` Resource 中；
- 维护 `Handle<T>` 与资源生命周期之间的关系。

`AssetPlugin` 本身不是图片、音频或模型解码器。它提供的是统一的资源管线，具体格式由
其它插件注册对应的 `AssetLoader`。例如，`ImagePlugin` 注册图片类型，`GltfPlugin` 注册
glTF 模型类型，`AudioPlugin` 注册音频类型。

因此，下面两部分要分开理解：

| 部分 | 作用 |
| --- | --- |
| `AssetPlugin` | 资源根目录、路径解析、异步请求、句柄和生命周期 |
| 格式插件与 `AssetLoader` | 把某种文件格式解码成具体的 Bevy Asset 类型 |

`DefaultPlugins` 会把常用的格式插件一起加入。只添加 `AssetPlugin` 时，并不会自动获得
所有图片、字体、音频和模型格式的加载器。

## 二、常见资源格式

下面列出 `DefaultPlugins` 中最常见的资源类型。格式是否可用还受 `Cargo.toml` 的 Bevy
feature 控制；没有启用对应 feature 时，即使文件扩展名正确也不能加载。

| 资源用途 | Bevy 类型 / 主要插件 | 常见扩展名 | 说明 |
| --- | --- | --- | --- |
| 纹理、UI 图片 | `Image` / `ImagePlugin` | `.png`、`.hdr`、`.ktx2` | 当前项目默认启用；JPEG、WebP、BMP、GIF、TGA、TIFF、QOI 等需要对应 image feature |
| 音频 | `AudioSource` / `AudioPlugin` | `.ogg` | 当前默认启用 Vorbis；`.mp3`、`.wav`、`.flac` 需要额外的音频 feature |
| 3D 模型 | `Gltf` / `GltfPlugin` | `.gltf`、`.glb` | 会继续加载模型引用的网格、材质和纹理 |
| 字体 | `Font` / `TextPlugin` | `.ttf`、`.otf` | 用于 `TextFont` 等文字渲染组件 |
| 着色器 | `Shader` / `RenderPlugin` | `.wgsl`、`.wesl` | `.spv` 需要启用 SPIR-V 支持 |
| 序列化 World | `DynamicWorld` / `WorldSerializationPlugin` | `.scn`、`.scn.ron` | 从序列化文件恢复实体和组件 |
| 动画图 | `AnimationGraph` / `AnimationPlugin` | `.animgraph`、`.animgraph.ron` | 描述动画节点和混合关系 |

这些不是“扩展名被 `AssetPlugin` 写死”的列表，而是当前 Bevy 中由各个插件注册的加载器。
如果项目启用了不同的 feature，实际支持的格式集合也会变化。

### 图片格式的 feature

当前项目使用顶层 `bevy` 的默认 feature，并额外启用了 `bevy_feathers`。默认图片功能中
最常遇到的是：

- `png`：普通 UI 图标、精灵和带透明通道的贴图；
- `hdr`：高动态范围纹理；
- `ktx2`：常用于压缩纹理和 3D 纹理资源。

JPEG、WebP 等格式不是仅凭文件后缀就自动获得的。如果要使用它们，需要在依赖中显式打开
对应 feature，例如：

```toml
[dependencies]
bevy = { git = "https://github.com/bevyengine/bevy.git", branch = "main", features = [
    "jpeg",
] }
```

具体 feature 名称和格式支持应以当前 Bevy 版本的 `Cargo.toml` 为准。

## 三、AssetPlugin 的资源根目录

示例显式设置了资源根目录：

```rust
DefaultPlugins
    .set(AssetPlugin {
        file_path: "assets".into(),
        ..default()
    })
```

项目目录和资源路径的对应关系是：

```text
项目根目录/
├── assets/
│   └── images/bevy-icon.png
└── examples/
    └── assets/lab-002-asset-plugin.rs
```

因此代码中写：

```rust
asset_server.load::<Image>("images/bevy-icon.png");
```

路径是相对于 `AssetPlugin::file_path` 的，不要再把 `assets/` 重复写进资源路径。若把资源
根目录改为 `data/assets`，代码仍然写 `images/bevy-icon.png`，只是在发布目录中把文件放到：

```text
data/assets/images/bevy-icon.png
```

资源路径通常使用 `/` 分隔，并且不应该写依赖当前机器的绝对路径。绝对路径会破坏发布时
的目录结构，也可能被 `AssetPlugin` 的路径安全策略拒绝。

## 四、路径字符串、句柄和真正的加载

需要区分四种状态：

| 写法或状态 | 是否开始读取文件 | 说明 |
| --- | --- | --- |
| `"images/bevy-icon.png"` 字符串 | 否 | 只是一个路径值，保存在变量中不会访问磁盘 |
| `asset_server.load(path)` | 是 | 立即向资源管线提交加载请求，返回 `Handle<T>`，不会阻塞当前系统 |
| `LoadState::Loading` | 已经请求 | 文件可能正在读取、解码，或者等待依赖资源 |
| `LoadState::Loaded` 且依赖也就绪 | 已完成 | 资源数据已经放入 `Assets<T>`，可以安全用于渲染或其它系统 |

所以 Bevy 不是“等到实体第一次使用时才自动加载”。更准确的说法是：

1. 只有路径字符串时，不会发生加载；
2. 调用 `AssetServer::load` 时，加载请求立刻开始；
3. 调用本身立即返回，读取和解码在后台异步完成；
4. 资源准备好后，才可以从 `Assets<T>` 取得实际资源数据。

下面的代码甚至没有创建 `ImageNode` 或 `Sprite`，图片仍然会被加载：

```rust
let image: Handle<Image> = asset_server.load("images/bevy-icon.png");
```

这正是本实验要证明的重点：是否加载由 `load` 请求决定，而不是由实体是否已经使用该
句柄决定。

### `Handle<T>` 不是图片本身

```rust
let image: Handle<Image> = asset_server.load("images/bevy-icon.png");

commands.spawn((
    Sprite::from_image(image.clone()),
    Transform::default(),
));
```

`Handle<Image>` 是指向 `Assets<Image>` 中某个条目的强引用。它可以复制和共享，但不包含
图片像素数据。图片还没有完成时，实体可以先创建；渲染系统会在资源可用后使用它。

多个系统对相同路径调用 `load` 时，Bevy 会复用同一个资源路径对应的加载记录，而不是为每个
实体重复读取一份图片。只要仍有强句柄存活，资源就可以继续留在内存中；当不再需要资源时，
应该释放不必要的强句柄，让资源系统回收它。

## 五、BSN、Bundle 与“使用时加载”的关系

不同的实体创建方式不会改变 AssetPlugin 的基本时机：

```rust
// Bundle：执行这行时就开始请求。
let image = asset_server.load::<Image>("images/bevy-icon.png");
commands.spawn((ImageNode::new(image), Node::default()));
```

```rust
// BSN：场景解析 ImageNode 的路径字段时取得句柄并开始请求。
bsn! {
    ImageNode { image: "images/bevy-icon.png" }
}
```

如果希望“真正需要时才加载”，应该延后调用 `load` 的代码，而不是只延后把句柄放进组件：

```rust
fn create_sprite_when_needed(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 只有系统被触发时，下面的请求才会发生。
    let image = asset_server.load("images/bevy-bird-dark.png");
    commands.spawn(Sprite::from_image(image));
}
```

这就是 001 中按钮观察者采用的方式：按钮点击以前没有调用 Sprite 图片的 `load`，点击以后
才开始异步加载。调用之后仍然不会同步等待图片读取完成。

## 六、示例代码如何观察加载状态

示例的 `Startup` 系统请求图片并保存句柄：

```rust
let image = asset_server.load::<Image>(IMAGE_PATH);
println!("current state: {:?}", asset_server.load_state(image.id()));
commands.insert_resource(AssetDemo {
    image,
    updates: 0,
    reported_loading: false,
});
```

后续 `Update` 查询两部分状态：

```rust
let state = asset_server.load_state(demo.image.id());
if asset_server.is_loaded_with_dependencies(&demo.image) {
    // Image is available in Assets<Image>.
}
```

`load_state` 只描述主资源的状态；`is_loaded_with_dependencies` 还会确认它依赖的资源也已经
准备好。glTF 等复合资源尤其应该使用后者，因为模型文件完成并不代表纹理和网格都完成。

示例在资源就绪后通过 `AppExit` 退出，因此不需要创建实体或窗口来证明资源确实已经加载。

## 七、自定义格式

如果项目有自己的配置、关卡或数据格式，`AssetPlugin` 不会因为文件后缀是 `.json`、`.ron`
或 `.data` 就自动知道如何解析。需要：

1. 定义一个实现 `Asset` 的资源类型；
2. 定义一个实现 `AssetLoader` 的加载器；
3. 在 `App` 中注册该加载器；
4. 再使用 `AssetServer::load::<YourAsset>("path/to/file.data")`。

这套机制让自定义资源也能使用和图片、模型相同的句柄、异步加载、依赖跟踪和生命周期管理。
自定义加载器属于后续专题，本实验只需要记住：不认识的扩展名必须由应用自己提供加载器。

## 八、结论

- `AssetPlugin` 管理资源管线，不负责单独解析所有格式；
- 常见格式由 `ImagePlugin`、`AudioPlugin`、`GltfPlugin`、`TextPlugin` 等注册加载器；
- 路径字符串本身不会加载资源；
- `AssetServer::load` 被调用时就会立刻提交异步加载请求；
- `load` 返回的是句柄，不会等待文件读取和解码；
- 资源进入 `Assets<T>` 并且依赖就绪后，才算真正可以使用；
- 想实现按需加载，就延后调用 `AssetServer::load`，不要等待实体第一次渲染才期待 Bevy 自动请求。
