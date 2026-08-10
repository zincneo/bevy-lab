# Lab 002：App、World 与 ECS 内容概览

## 学习目标

从整体上认识 Bevy 应用是如何组织这些 ECS 内容的需要了解

- `App` 负责什么
- `World` 保存什么
- `Schedule`、`System`、`Plugin` 分别处于什么位置；
- 实体、组件、资源、状态和消息如何放入并使用 `World`。

后续实验会分别详细介绍 Commands、System、Query、Resource、State 和 Message。本实验只建立它们之间的整体关系。

## `App` 是什么

`App` 是 Bevy 应用的组织和运行容器。它负责把 ECS 的 `World`、系统调度计划、插件和应用运行器组合起来，并在运行时不断执行已经注册的 Schedule。

一个最小的 App 可以这样配置：

```rust
fn main() {
    App::new()
        .add_plugins(MyPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}
```

这里的调用含义是：

| App 常用API | 作用 |
| --- | --- |
| `App::new()` | 创建一个新的应用容器和主 World。 |
| `add_plugins(...)` | 注册一组可复用的应用配置，包括资源、系统和其他插件。 |
| `add_systems(schedule, systems)` | 将 System 注册到指定的 Schedule。这里只是登记，不会立即执行。 |
| `insert_resource(...)` / `init_resource::<T>()` | 在 App 的 World 中准备全局 Resource。 |
| `add_message::<M>()` | 在 World 中准备一个 `Messages<M>` Resource，供系统传递消息。 |
| `init_state::<S>()` / `insert_state(...)` | 在 World 中准备 `State<S>` 和 `NextState<S>` Resource，并启用状态转换所需的调度。 |
| `run()` | 交给应用运行器，按照配置执行各个 Schedule。 |

App 还提供 `world()` 和 `world_mut()` 访问内部 World，主要用于应用配置、测试或特殊的独占操作。普通 System 通常通过 `Query`、`Res`、`Commands` 等系统参数访问 World，而不是直接持有整个 World。

## `World` 是什么

`World` 是 Bevy ECS 的数据容器。它保存实体、组件、资源、状态以及 ECS 运行所需的注册信息和存储结构。System 通过系统参数从 World 读取或修改数据。

可以把 App 和 World 的关系先理解为：

```text
App
├── 主 World：保存 ECS 数据
│   ├── Entities：实体及其 ID
│   ├── Components：实体上的组件数据
│   ├── Resources：全局唯一资源
│   ├── State：由 `State<S>` 和 `NextState<S>` Resource 保存
│   └── Messages：由 `Messages<M>` Resource 保存
├── Schedules：决定系统何时执行
├── Plugins：组织应用配置
└── Runner：驱动应用运行循环
```

`World` 只负责保存和提供 ECS 数据；“什么时候执行哪个 System”属于 Schedule 和 App 的职责。System 则是使用这些数据完成具体行为的代码。

World 本身不会主动执行逻辑。App 的 runner 按配置触发 Schedule，Schedule 调用其中的 System，System 再通过系统参数读取或修改 World；App 持续重复这些 Schedule，就形成应用的运行循环。

## `World` 中最常用的内容

### Entity

Entity 是 World 中对象的身份标识，通常只包含一个可复制的 ID。Entity 本身不保存业务字段，业务数据保存在挂载到它身上的 Component 中：

```rust
let player = commands.spawn((Player, Health(100))).id();
```

上例会创建一个 Entity，并为它添加 `Player` 和 `Health` 两个 Component。之后可以使用 Entity ID 定位它，或者通过 Query 按组件组合查找它。

### Component

Component 是附着在 Entity 上的数据类型。一个 Entity 可以拥有多个 Component，Entity 的具体类型由它拥有的组件组合决定：

```rust
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}
```

Component 适合保存与某个实体相关的数据。System 通常使用 `Query` 批量读取或修改 Component；需要改变实体结构时使用 `Commands`。

### Resource

Resource 是 World 中某一种类型的全局唯一数据。同一个 World 通常只能有一个同类型 Resource，它不属于某个特定 Entity：

```rust
#[derive(Resource, Default)]
struct GameScore(u32);
```

Resource 适合保存全局配置、计时器、游戏状态或统计数据。System 可以使用 `Res<GameScore>` 读取，使用 `ResMut<GameScore>` 修改；也可以通过 App 或 Commands 插入和初始化 Resource。

从 Bevy 0.19 开始，`Resource` 是 `Component` 的一种受约束形式：当前源码中的 `Resource` 特征继承自 `Component`，Resource 在底层也以单例实体上的 Component 形式存储。因此 `#[derive(Resource)]` 会同时完成 Component 所需的注册，但 Resource 仍然保持“每个 World 同一种类型只有一个实例”的语义，通常应该使用 `Res<T>` / `ResMut<T>` 访问，而不是把它当作普通实体组件使用。

### State

State 是由 App 管理的类型化状态，用来表示应用当前处于哪个阶段或模式，例如主菜单、游戏中或暂停。严格来说，World 中保存的是 `State<S>` 和 `NextState<S>` 两个 Resource：前者表示当前状态，后者保存下一次切换请求。State 还会参与 App 的调度，让 System 可以根据当前状态决定是否运行，状态进入或退出时也可以触发专门的 Schedule。

通常只需要在 App 中初始化状态：

```rust
app.init_state::<GameState>();
```

### Message

Message 是 System 之间传递短生命周期通知的一种方式。例如输入系统发送“玩家跳跃”，移动系统或音效系统分别读取这条消息。Message 不需要让发送方知道接收方是谁，因此可以减少系统之间的直接依赖。

严格来说，消息 payload 类型 `M` 本身不是 Resource；App 注册后，World 中保存的是负责排队和管理这些消息的 `Messages<M>` Resource。系统使用 `MessageReader<M>` 读取、使用 `MessageWriter<M>` 发送。具体的 Message 用法将在后续实验介绍。

## `Schedule`、`System` 与 `World` 的关系

这三者可以分别理解为：

| 内容 | 负责的问题 |
| --- | --- |
| `World` | 数据存在哪里？ |
| `System` | 对这些数据做什么？ |
| `Schedule` | 什么时候执行这些 System？ |

例如：

```rust
fn update_player(
    mut players: Query<&mut Position, With<Player>>,
    speed: Res<PlayerSpeed>,
) {
    // 从 World 读取 PlayerSpeed，查询并修改 Player 的 Position。
}

app.add_systems(Update, update_player);
```

`update_player` 是行为，`Position` 和 `PlayerSpeed` 是 World 中的数据，`Update` 是它所属的执行计划。调用 `add_systems` 时只会把行为登记到计划中，真正运行要等 App runner 执行 `Update` Schedule。

## Plugin 在整体结构中的位置

Plugin 是组织 App 配置的方式。它本身不是 Entity、Component 或 System，而是一个特征，实现该特征的类型可以添加到App中，用于集中注册这些内容的配置单元：

```rust
struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, update_player);
    }
}
```

当 App 调用 `add_plugins(PlayerPlugin)` 时，Bevy 会执行插件的 `build`，把插件需要的资源和系统注册到 App。这样可以把一个功能拆成独立、可复用的模块。

## 一次运行的大致流程

```text
创建 App
    ↓
添加插件、插入资源、消息、状态、System 和 Schedule 配置
    ↓
App runner 启动
    ↓
执行 Startup 等启动 Schedule
    ↓
进入主循环，反复执行 Update 等 Schedule
    ↓
System 通过参数访问 World 并产生数据修改
    ↓
应用 Commands 等延迟修改
```

在没有窗口、输入或持续运行插件的最小实验中，App 也可以只执行一次更新后退出。是否持续运行由 App runner 和插件配置决定，而不是由某一个普通 System 函数决定。
