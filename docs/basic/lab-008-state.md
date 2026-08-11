# Lab 008：State

## 学习目标

`State` 用来表示应用在某个维度上的有限状态，例如菜单、运行中和暂停。Bevy 会在状态发生切换时运行对应的进入、退出和过渡 Schedule，也可以让普通 System 只在指定状态下运行。

## State 的组成

先定义一个实现 `States` 的枚举。通常还会派生 `Default`，用来指定初始状态：

```rust
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum AppMode {
    #[default]
    Menu,
    Playing,
    Paused,
}
```

这个枚举的每个变体就是一个状态。Bevy 会在 `World` 中保存与它对应的资源：

| Resource | 用途 |
| --- | --- |
| `State<AppMode>` | 当前已经生效的状态，只读访问。 |
| `NextState<AppMode>` | 下一次状态切换的请求，System 通过它提交目标状态。 |
| `PreviousState<AppMode>` | 最近一次切换前的状态，在发生过切换后才会出现。 |

日常代码通常只需要使用前两个 Resource。`State<S>` 不应该直接修改，应该通过 `NextState<S>` 请求切换，让 Bevy 在状态处理阶段统一应用。

## 一个枚举还是多个状态枚举

不需要为每个状态都注册一个枚举类型。互相排斥的模式通常放在同一个枚举中：

```rust
enum AppMode {
    Menu,
    Playing,
    Paused,
}
```

一个 `AppMode` 在同一时刻只有一个变体，这正好表达了“当前应用模式”。

当两个状态描述的是相互独立的维度时，可以定义多个状态枚举并分别注册。例如：

```rust
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum AppMode {
    #[default]
    Menu,
    Playing,
}

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum Connection {
    #[default]
    Offline,
    Online,
}
```

这里应用模式和网络连接状态可以同时存在，因此拆成两个状态维度更合适。这样可以表达 `Playing + Online`、`Playing + Offline` 等组合，而不必把所有组合都写成一个巨大枚举。

可以按下面的规则选择：

1. 变体之间互相排斥、只表示同一个流程阶段时，使用一个状态枚举。
2. 状态可以独立变化、同时成立时，使用多个状态枚举。
3. 每个状态类型只注册一次；每种状态类型在 `World` 中有自己独立的 `State<S>` 和 `NextState<S>`。

## 注册初始状态

使用状态需要 `StatesPlugin`。`DefaultPlugins` 已经包含它；使用 `App::new()` 和少量插件时，需要显式添加：

```rust
use bevy::{prelude::*, state::app::StatesPlugin};

App::new()
    .add_plugins(StatesPlugin)
    .init_state::<AppMode>()
    .run();
```

`init_state::<AppMode>()` 会使用 `AppMode::default()` 插入初始状态，并准备 `State<AppMode>`、`NextState<AppMode>` 以及状态过渡所需的 Schedule。

如果不想使用 `Default`，也可以直接指定初始值：

```rust
App::new()
    .add_plugins(StatesPlugin)
    .insert_state(AppMode::Playing)
    .run();
```

常规项目优先使用 `init_state`，因为初始值和枚举的 `Default` 实现放在一起更直观；需要由配置决定初始状态时使用 `insert_state`。

## 读取当前状态

在 System 中使用 `Res<State<S>>` 读取已经生效的状态：

```rust
fn print_mode(mode: Res<State<AppMode>>) {
    println!("当前状态：{:?}", mode.get());
}
```

`State<S>` 也实现了对枚举值的只读解引用，`mode.get()` 是更明确的写法。状态只有在状态处理阶段完成后才会更新；刚刚提交的 `NextState` 不会立刻改变同一个 System 中的 `State`。

## 请求状态切换

使用 `ResMut<NextState<S>>` 提交目标状态：

```rust
fn start_game(mut next_mode: ResMut<NextState<AppMode>>) {
    next_mode.set(AppMode::Playing);
}
```

`set` 只是记录一次待处理的切换请求。Bevy 会在 `StateTransition` Schedule 中读取它，并依次处理退出、过渡和进入逻辑。

如果目标状态可能与当前状态相同，可以使用 `set_if_different`：

```rust
fn pause(mut next_mode: ResMut<NextState<AppMode>>) {
    next_mode.set_if_different(AppMode::Paused);
}
```

这个方法在目标状态已经是当前状态时不会再次触发状态过渡。一个主循环中如果多个 System 都提交请求，最终应用的是状态处理阶段读取到的请求，因此需要按项目逻辑安排这些 System 的执行顺序。

## 状态相关的 Schedule

注册状态后，Bevy 会准备以下常用 Schedule：

| Schedule | 运行时机 |
| --- | --- |
| `StateTransition` | 负责读取 `NextState` 并应用状态切换。默认在每次主循环的 `PreUpdate` 之后运行。 |
| `OnExit(value)` | 离开指定状态时运行一次。 |
| `OnTransition { exited, entered }` | 从指定旧状态切换到指定新状态时运行一次。 |
| `OnEnter(value)` | 进入指定状态时运行一次。 |

在主循环中的简化顺序是：

```text
PreUpdate
    ↓
StateTransition
    ├── OnExit(旧状态)
    ├── OnTransition { exited, entered }
    └── OnEnter(新状态)
    ↓
Update
```

应用第一次启动时，状态处理也会先应用初始状态并运行对应的 `OnEnter`，然后才进入启动阶段。

### `OnEnter`：进入状态时初始化

```rust
fn setup_playing() {
    println!("进入 Playing，准备运行所需内容");
}

app.add_systems(OnEnter(AppMode::Playing), setup_playing);
```

`OnEnter` 适合创建或准备只在该状态使用的内容。

### `OnExit`：离开状态时清理

```rust
fn cleanup_playing() {
    println!("离开 Playing，清理运行内容");
}

app.add_systems(OnExit(AppMode::Playing), cleanup_playing);
```

`OnExit` 适合释放或重置该状态独有的内容。

### `OnTransition`：处理特定的状态跳转

```rust
fn menu_to_playing() {
    println!("从 Menu 进入 Playing");
}

app.add_systems(
    OnTransition {
        exited: AppMode::Menu,
        entered: AppMode::Playing,
    },
    menu_to_playing,
);
```

只有旧状态和新状态都匹配时，这个 Schedule 才会运行。通常只有需要区分具体跳转路径时才使用它；一般初始化和清理分别使用 `OnEnter`、`OnExit` 即可。

## 只在指定状态运行普通 System

`in_state` 是最常用的状态运行条件：

```rust
fn playing_update() {
    println!("当前正在执行 Playing 逻辑");
}

app.add_systems(Update, playing_update.run_if(in_state(AppMode::Playing)));
```

System 每次执行前都会检查当前 `State<AppMode>`。不满足条件时，System 会被跳过；状态切换后它会自动开始或停止运行。

多个状态条件可以组合，但入门阶段通常为每个状态直接注册一个清晰的运行条件：

```rust
app.add_systems(
    Update,
    (
        menu_update.run_if(in_state(AppMode::Menu)),
        playing_update.run_if(in_state(AppMode::Playing)),
    ),
);
```

## 状态运行流程

一个最小的状态使用流程可以概括为：

```text
定义 #[derive(States)] 的枚举
        ↓
添加 StatesPlugin
        ↓
init_state 或 insert_state
        ↓
Res<State<S>> 读取当前状态
ResMut<NextState<S>> 提交切换请求
        ↓
StateTransition 应用请求
        ↓
OnExit → OnTransition → OnEnter
        ↓
Update 中通过 in_state 运行对应逻辑
```

需要记住的几个要点：

1. `NextState` 是“请求”，`State` 是“当前已经生效的值”；两者不是同一个 Resource。
2. 状态切换不会在调用 `set` 的那一行立即完成，而是在 `StateTransition` 阶段应用。
3. `OnEnter` 和 `OnExit` 只在切换发生时运行一次，持续逻辑应放在 `Update.run_if(in_state(...))` 中。
4. `App::new()` 不会自动添加 `StatesPlugin`；使用 `DefaultPlugins` 时则不需要单独添加。
