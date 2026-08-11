# Lab 017：DefaultPlugins 提供的常用核心 Resource 和 Message

`DefaultPlugins` 会把许多基础插件加入 App。它们不只是“让程序跑起来”，还会向当前 `World` 注册常用的 Resource、Message、组件和系统。

这个 lab 只介绍核心运行时、资源加载和窗口循环，不展开渲染、UI、设备输入或复杂控制。`State`、普通 Message 和时间推进的详细用法已经在前面的 lab 中介绍，这里只补充它们与默认插件的关系。

## 常用核心 Resource

### `FrameCount`

`FrameCountPlugin` 提供 `FrameCount`，记录主循环已经执行过的帧数。它适合做调试输出、简单的执行次数统计和测试中的循环条件。

### `Time`

`TimePlugin` 提供 `Time`、`Time<Virtual>` 和 `Time<Fixed>` 等时间 Resource。系统通过它们读取帧间隔和累计时间。时间的详细推进规则见 basic 011。

### `DiagnosticsStore`

`DiagnosticsPlugin` 初始化 `DiagnosticsStore`。它是诊断数据的集合，可以保存帧时间、FPS、实体数量或自定义指标。默认主要提供存储能力，具体指标通常还需要额外注册诊断插件或自行注册。

```rust
fn read_diagnostics(diagnostics: Res<DiagnosticsStore>) {
    for diagnostic in diagnostics.iter() {
        println!("{}", diagnostic.path());
    }
}
```

诊断数据适合调试和性能观察，不应该作为游戏逻辑的计时来源。

### `AssetServer`

`AssetPlugin` 提供 `AssetServer`，负责按路径请求资源并返回 `Handle<T>`。它本身不负责渲染，资源类型可以是图像、音频、场景或自定义资产。

```rust
fn request_asset(asset_server: Res<AssetServer>) {
    let handle: Handle<MyAsset> = asset_server.load("data/example.asset");
    // 把 handle 保存到组件或自己的 Resource 中。
}
```

`Assets<T>` 是资源集合，`AssetEvent<T>` 是资源加载、修改和移除时发送的 Message。具体图像、网格和场景的使用方式放到对应主题中学习。

### `WinitSettings`

使用窗口时，`WinitPlugin` 会注册 `WinitSettings` Resource。它决定窗口获得焦点和失去焦点时，事件循环如何再次驱动 App 的 Schedule：

- `WinitSettings::game()`：获得焦点时连续更新，失去焦点时使用低功耗响应模式，适合游戏；
- `WinitSettings::desktop_app()`：主要由窗口事件或定时唤醒，适合编辑器和普通桌面应用；
- `WinitSettings::continuous()`：无论窗口是否聚焦都尽可能连续更新。

也可以直接设置 `focused_mode` 和 `unfocused_mode`，例如 `UpdateMode::reactive(Duration::from_millis(250))`。

## 窗口组件与常用 Message

窗口不是一个全局 Resource，而是带有 `Window` 组件的 Entity。默认主窗口还带有 `PrimaryWindow` 标记：

```rust
fn read_primary_window(window: Single<&Window, With<PrimaryWindow>>) {
    println!("size = {:?}", window.resolution.size());
}
```

`WindowPlugin` 创建和维护窗口 Entity，`WinitPlugin` 将它与操作系统窗口同步。常见窗口消息包括：

- `WindowResized`：逻辑尺寸变化；
- `WindowFocused`：窗口获得或失去焦点；
- `WindowCloseRequested`：操作系统请求关闭窗口；
- `WindowClosed`：窗口已经关闭；
- `RequestRedraw`：请求事件循环尽快再次更新和重绘。

这些消息使用 `MessageReader<T>` 读取。窗口关闭后是否立即退出，还取决于 App 的窗口退出设置和系统对 `AppExit` 的处理。

## `AppExit` Message

`AppExit` 是 App 的退出消息。系统、窗口关闭处理或终端 Ctrl-C 处理器都可以写入它：

```rust
fn quit(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}
```

`TerminalCtrlCHandlerPlugin` 会在支持的平台上把终端 Ctrl-C 转换为退出请求。它不负责决定业务上的退出条件，真正的退出仍由 App 主循环处理。

## ScheduleRunnerPlugin 如何控制循环频率

这里需要区分“谁负责驱动 Schedule”和“时间 Resource 如何计算增量”。

### 无窗口 App：`ScheduleRunnerPlugin`

`ScheduleRunnerPlugin` 包含在 `MinimalPlugins` 中。当项目没有窗口功能时，可以这样配置：

```rust
App::new()
    .add_plugins(
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(100))),
    )
    .run();
```

- `run_once()`：只调用一次 `App::update()`；
- `run_loop(wait)`：重复调用 `App::update()`，并根据 `wait` 控制两次更新之间的等待时间；
- `wait` 是调度循环的目标间隔，不是严格的实时定时器，系统执行耗时和操作系统调度都会影响实际间隔。

当启用了窗口功能时，`DefaultPlugins` 不再加入 `ScheduleRunnerPlugin`，因为它会由 `WinitPlugin` 提供的操作系统事件循环驱动 App。

### 有窗口 App：`WinitSettings`

窗口应用通过 `WinitSettings` 的 `UpdateMode` 控制更新：

- `Continuous`：尽可能连续调用 Schedule，不设置固定帧率上限；
- `Reactive { wait, .. }`：在窗口事件、输入事件或等待时间到达时更新；`wait` 是近似的最短唤醒间隔；
- `reactive_low_power(wait)`：类似 `Reactive`，但忽略一部分不会直接作用于窗口的设备事件，以降低功耗。

`WinitSettings` 的等待时间只决定 Schedule 何时被再次驱动，不会把 `Time` 变成固定时间步。`TimeUpdateStrategy` 和 `Time<Fixed>` 影响的是时间值以及 `FixedUpdate` 的追赶次数，详细内容见 basic 011。

另外，更新频率和 VSync 不是同一件事：VSync 由窗口的 `PresentMode` 和渲染器控制，`WinitSettings` 控制的是 App Schedule 的唤醒方式。

## 示例说明

示例使用完整的 `DefaultPlugins`，并把 `WinitSettings` 设置为：窗口聚焦时约每 250ms 响应一次，失去焦点时约每 1s 进入低功耗响应。运行时会在终端打印：

- `FrameCount`、`Time` 和 `DiagnosticsStore`；
- 当前主窗口尺寸和实际 `UpdateMode`；
- `WindowResized`、`WindowFocused` 和 `WindowCloseRequested` 消息。

```bash
nix develop
just run basic 017
```

示例需要图形环境打开窗口。调整窗口大小或切换焦点观察窗口消息，按 `Escape` 写入 `AppExit::Success` 退出。
