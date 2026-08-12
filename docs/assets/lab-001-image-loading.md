# 001：加载和使用图片资源

运行示例：

```bash
nix develop
just run assets 001
```

本实验使用两张图片演示三种常见实体创建方式，并观察按需异步加载：

1. UI 由 BSN 创建，`ImageNode` 直接写资源路径；
2. UI 由普通 Bundle 创建，先通过 `AssetServer` 得到 `Handle<Image>`，再构造 `ImageNode`；
3. 2D `Sprite` 使用另一张图片，并且只有点击 UI 按钮后才创建实体、请求图片资源。

启动时只会请求 UI 使用的图片。点击按钮后，系统才第一次调用 `AssetServer::load` 请求
Sprite 图片；这个调用立即返回句柄，读取和解码在 Bevy 的资源管线中异步完成。

图片文件位于：

```text
assets/images/bevy-icon.png
assets/images/bevy-bird-dark.png
```

前一张图片来自上一级 Bevy 仓库的 `assets/branding/icon.png`，后一张来自
`assets/branding/bevy_bird_dark.png`，都复制到当前项目自己的资源目录中。运行时使用的路径
不是工作区的绝对路径，而是相对于 Bevy 资源根目录的：

```text
images/bevy-icon.png
images/bevy-bird-dark.png
```

## 一、图片资源、句柄和显示组件

图片加载涉及三个不同概念：

| 概念 | 作用 |
| --- | --- |
| `Image` | Bevy 载入内存后的图片资源数据 |
| `Handle<Image>` | 指向资源的可复制、可共享句柄，不是图片本身 |
| `ImageNode` / `Sprite` | 把图片句柄放到实体上，分别交给 UI 渲染和 2D Sprite 渲染 |

`AssetServer::load` 通常是加载入口：

```rust
let image: Handle<Image> = asset_server.load("images/bevy-icon.png");
```

调用会立即返回句柄，但图片文件的读取和解码是异步进行的。实体可以先创建，实际图片在资源准备好后才会显示；因此不应该把 `Handle<Image>` 当成已经可用的 `Image` 数据。

同一个资源路径重复调用 `load` 会得到可复用的资源引用。示例中的两个 UI 图片都使用
`bevy-icon.png`，Bevy 会通过资源路径和句柄管理共享的图片资源，而不是为每个实体复制一份
图片。`bevy-bird-dark.png` 没有在启动阶段调用 `load`，因此不会因为 Sprite 摄像头已经创建
就提前进入这次实验的加载流程。

## 二、UI 图片：普通 Bundle 方式

普通 Bundle 方式是在 Rust 系统中显式获得句柄，并把它放进 `ImageNode`：

```rust
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let image = asset_server.load::<Image>("images/bevy-icon.png");

    commands.spawn((
        Node {
            width: px(170),
            height: px(170),
            ..default()
        },
        ImageNode::new(image),
    ));
}
```

在当前 Bevy 中，`ImageNode` 会自动要求实体拥有 `Node`，但图片显示区域的宽高、边距和布局仍然应该由 `Node` 配置。示例的 `bundle_image_panel` 使用 `children!` 创建一个面板，再把 `ImageNode::new(handle)` 和控制尺寸的 `Node` 放到同一个子实体上。这个 `handle` 是在 `Startup` 中取得的，因此 UI 图片会在应用启动后立即进入加载流程。

这种方式的特点是：

- `AssetServer` 和句柄来源在 Rust 代码中清晰可见；
- 适合根据运行时资源、配置或状态选择图片；
- 可以直接复用同一个 `Handle<Image>` 创建多个 UI 或 Sprite 实体；
- UI 树仍然是普通的 ECS Bundle 和 `ChildOf`/`Children` 关系。

## 三、UI 图片：BSN 方式

BSN 支持把资源路径直接写入支持 `FromTemplate` 的组件字段：

```rust
fn bsn_image_panel(parent: Entity) -> impl Scene {
    bsn! {
        ChildOf(parent)
        Node { width: px(170), height: px(170) }
        ImageNode { image: "images/bevy-icon.png" }
    }
}
```

这里没有手动调用 `AssetServer::load`。`ImageNode` 的 `image` 字段是 `Handle<Image>`，BSN 会把字符串路径转换成场景的资源依赖，并在场景解析时通过资源系统取得句柄。

BSN 场景需要通过 `spawn_scene` 等场景 API 生成：

```rust
commands.spawn_scene(bsn_image_panel(ui_root));
```

本例中 BSN 场景的根实体包含 `ChildOf(ui_root)`，因此它会成为 Bundle 创建的 UI 根实体的子实体。BSN 只负责静态实体结构和资源字段声明，图片仍然由 Bevy Asset 系统异步加载。

### BSN 方式与 Bundle 方式的区别

| 对比项 | BSN | Bundle |
| --- | --- | --- |
| 图片路径 | 可直接写在 `ImageNode { image: "..." }` | 先 `asset_server.load` 再传入句柄 |
| 结构表达 | 适合固定 UI 树、父子关系和场景复用 | 适合系统中按条件创建实体 |
| 运行时选择资源 | 可通过 Rust 表达式插入句柄，但仍受场景语法约束 | 直接根据资源或配置选择句柄 |
| 资源依赖 | 由 Scene 注册并等待/解析 | 由 `AssetServer` 返回句柄后直接使用 |

如果图片路径本身是运行时数据，通常使用 Bundle 或在 BSN 中嵌入 Rust 表达式：

```rust
let image = asset_server.load(path_from_config);
commands.spawn_scene(bsn! {
    ImageNode { image: {image} }
});
```

固定的界面结构更适合 BSN；动态的图片选择更适合显式保存和传递句柄。

## 四、点击后按需异步加载 Sprite

示例没有在 `Startup` 中创建 Sprite，也没有在 `setup` 中调用
`asset_server.load("images/bevy-bird-dark.png")`。Sprite 摄像头可以提前存在，但摄像头本身
不会请求任何图片资源。

BSN 按钮通过 `ui_widgets::Button` 和 `on(load_sprite_on_click)` 注册实体观察者：

```rust
bsn! {
    Button
    on(load_sprite_on_click)
    Children [(
        Text::new("Load Sprite image on demand")
    )]
}
```

按钮被鼠标激活后，观察者才执行下面的逻辑：

```rust
fn load_sprite_on_click(
    _event: On<Activate>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 第一次请求 Sprite 专用图片；返回句柄不等于图片已经读取完成。
    let image = asset_server.load("images/bevy-bird-dark.png");

    commands.spawn((
        Sprite::from_image(image),
        Transform::from_xyz(0.0, -250.0, 0.0),
        RenderLayers::layer(1),
    ));
}
```

这里有两个时间点：

| 时间点 | 发生的事情 |
| --- | --- |
| 应用启动 | 创建两台摄像头和 UI；只请求 `bevy-icon.png` |
| 点击按钮 | 创建 Sprite 实体，并第一次请求 `bevy-bird-dark.png` |
| 点击之后的若干帧 | 资源系统读取、解码图片；Sprite 在句柄就绪后显示 |

示例中的状态文本会通过 `AssetServer::is_loaded_with_dependencies` 显示“正在等待”或“已经
就绪”，这样可以直接看到句柄返回和图片真正可用之间的区别。状态资源还会阻止重复点击
重复创建 Sprite；真实项目也可以用 State 或其它业务资源管理更复杂的加载流程。

## 五、2D Sprite 图片

`Sprite` 不属于 UI。它由 2D 渲染摄像头处理，通常和 `Transform` 一起生成：

```rust
let image = asset_server.load::<Image>("images/bevy-bird-dark.png");
commands.spawn((
    Sprite::from_image(image),
    Transform::from_xyz(0.0, -250.0, 0.0),
));
```

`Sprite::from_image` 只负责把图片句柄放入 Sprite。默认情况下 Sprite 使用图片尺寸；如果需要固定显示尺寸，可以设置：

```rust
Sprite {
    image,
    custom_size: Some(Vec2::new(128.0, 128.0)),
    ..default()
}
```

在本实验中，上面的代码只会在按钮观察者中执行。Sprite 的位置、旋转和缩放由 `Transform`
控制，不使用 UI 的 `Node` 布局。它和 `ImageNode` 可以共享同一个 `Handle<Image>`，但属于
不同的渲染组件和布局体系；本例为了区分两种资源，Sprite 使用 `bevy-bird-dark.png`。

## 六、摄像头和渲染层级

示例创建了两个 `Camera2d`：

```text
Camera 0：order = 0，RenderLayers::layer(1)
└── 按钮点击后创建的 Sprite（先绘制）

Camera 1：order = 1，RenderLayers::layer(0)，IsDefaultUiCamera，透明清屏（后绘制）
└── UI 根实体
    ├── BSN 创建的 ImageNode
    ├── Bundle 创建的 ImageNode
    └── 按钮
```

`Camera.order` 越小越先绘制。Sprite 摄像头先绘制第 1 层，UI 摄像头随后绘制默认 UI，因此
UI 位于 Sprite 上方。UI 摄像头只渲染第 0 层，因而不会把 Sprite 再绘制一次；同时使用透明
清屏配置，避免清除已经绘制好的 Sprite。示例的全屏 UI 根节点还使用半透明背景，这样下方
的 Sprite 在资源加载完成后可以透过 UI 背景看到：

```rust
Camera {
    order: 1,
    clear_color: ClearColorConfig::None,
    ..default()
}
RenderLayers::layer(0)
```

`RenderLayers` 是摄像头和实体之间的可见性筛选，不是资源加载机制。它只决定哪个摄像头绘制哪个实体；图片句柄的加载方式与实体属于 UI 还是 Sprite 无关。

## 七、图片何时算加载完成

句柄返回后，图片可能仍在读取。需要在切换场景或显示依赖图片的界面前确认资源状态时，可以保存句柄并检查：

```rust
#[derive(Resource)]
struct UiImages {
    logo: Handle<Image>,
}

fn wait_for_image(asset_server: Res<AssetServer>, images: Res<UiImages>) {
    if asset_server.is_loaded_with_dependencies(&images.logo) {
        // The image and its dependencies are ready.
    }
}
```

更大的应用通常会把一组预加载句柄放进 Resource，在加载状态完成后再切换 State；小型界面可以先生成实体，让资源就绪后自然显示。不要只保存一个临时的弱句柄，也不要在所有实体销毁后仍无条件保留大量强句柄，否则会影响资源释放。

## 八、应用打包时资源放在哪里

`cargo build` 只编译 Rust 程序，不会自动把项目的 `assets/` 目录复制进 `target/`。开发时，`DefaultPlugins` 默认从当前工作目录的 `assets/` 查找：

```text
项目根目录/
├── assets/
│   └── images/
│       ├── bevy-icon.png
│       └── bevy-bird-dark.png
└── target/debug/...
```

发布应用时，需要把资源目录作为应用数据一起分发，或者把 AssetPlugin 的资源根目录改到安装目录：

```rust
App::new()
    .add_plugins(DefaultPlugins.set(AssetPlugin {
        file_path: "data/assets".into(),
        ..default()
    }))
```

对应的发布目录可以是：

```text
my-game/
├── my-game
└── data/
    └── assets/images/
        ├── bevy-icon.png
        └── bevy-bird-dark.png
```

如果希望把资源编译进程序，则需要使用 Bevy 的 embedded asset 机制或其它自定义 AssetSource；那是另一种打包策略，不等同于普通 `assets/` 文件加载。当前实验先使用最直观、最适合开发迭代的外部资源目录方式。

## 九、选择方式总结

| 场景 | 推荐写法 |
| --- | --- |
| 固定 UI 树中的图片 | BSN 中使用 `ImageNode { image: "path" }` |
| 运行时根据配置选择 UI 图片 | `AssetServer::load` + `ImageNode::new(handle)` |
| 2D 世界中的图片 | 在需要时 `AssetServer::load` + `Sprite::from_image(handle)` |
| 多个实体使用同一图片 | 克隆并共享同一个 `Handle<Image>` |
| 发布时读取外部资源 | 随应用分发 `assets/`，或配置 `AssetPlugin::file_path` |
| 发布时嵌入资源 | embedded asset / 自定义 AssetSource |

核心关系是：`AssetServer` 负责取得资源句柄，`Image` 是资源数据，`ImageNode` 和 `Sprite` 分别把同一类图片放进 UI 与 2D 渲染流程。BSN 和 Bundle 只是两种创建实体、声明句柄的方式，并不会改变图片资源本身的加载机制。
