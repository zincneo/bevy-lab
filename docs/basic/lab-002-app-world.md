# Lab 002：App、World 与 ECS 内容概览

## 学习目标

本节只建立一个够用的整体认识：`App` 负责组织和驱动应用，`World` 保存 ECS 数据，System 通过参数访问这些数据。

## `App` 是什么

`App` 是 Bevy 应用的入口和运行容器。常见配置方式如下：

```rust
fn main() {
    App::new()
        .add_plugins(MyPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}
```

常用API：

| 方法 | 用途 |
| --- | --- |
| `App::new()` | 创建 App 和它的主 `World`。 |
| `add_plugins(...)` | 添加插件，集中注册资源和 System。 |
| `add_systems(schedule, systems)` | 将 System 注册到某个 Schedule。 |
| `insert_resource(...)` / `init_resource::<T>()` | 准备全局 Resource。 |
| `add_message::<M>()` | 准备消息队列。 |
| `init_state::<S>()` | 准备类型化状态。 |
| `run()` | 交给 runner 驱动应用。 |

调用 `add_systems` 只是登记 System，真正执行要等 App 运行对应的 Schedule。

## `World` 是什么

`World` 是 ECS 数据的容器，最常用的内容是：

```text
World
├── Entity：实体身份
├── Component：附着在实体上的数据
├── Resource：全局唯一数据
├── State：由 State<S> / NextState<S> Resource 保存
└── Messages：由 Messages<M> Resource 保存
```

App 的 runner 驱动 Schedule，Schedule 调用 System，System 再通过参数读取或修改 World：

```text
App runner → Schedule → System → 访问 World
```

### Entity 和 Component

Entity 只有身份，业务数据放在 Component 中。一个实体可以拥有多个组件：

```rust
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Position { x: f32, y: f32 }

commands.spawn((Player, Position { x: 0.0, y: 0.0 }));
```

通常使用 `Query` 按组件组合查找实体，使用 `Commands` 创建实体或修改实体结构。

### Resource

Resource 是 World 中按类型唯一的全局数据，适合保存配置、计分和游戏设置：

```rust
#[derive(Resource, Default)]
struct Score(u32);

fn show_score(score: Res<Score>) {
    println!("{}", score.0);
}
```

使用 `Res<T>` 读取，使用 `ResMut<T>` 修改。当前 Bevy 中 Resource 底层也使用 Component 存储，但使用时仍应把它当作全局 Resource，通过 `Res` 或 `ResMut` 访问。

### State 和 Message

这两类内容在使用层面也可以先看作 World 中的 Resource：

- `State<S>` 表示当前模式，例如菜单、游戏中或暂停；`NextState<S>` 表示下一次切换请求，通常通过 `app.init_state::<S>()` 初始化。
- `Messages<M>` 保存消息队列，App 通过 `add_message::<M>()` 注册，System 使用 `MessageWriter<M>` 发送、`MessageReader<M>` 读取。

它们的具体状态切换和消息生命周期会在后续 lab 单独介绍。

## Plugin 的作用

Plugin 是组织配置的方式，不是 World 中的一种数据。它可以把资源和 System 的注册集中起来：

```rust
struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, update_player);
    }
}
```

大型项目通常按功能拆分 Plugin，再在 `main` 中添加这些 Plugin。

## 小结

编写 Bevy 程序时可以先记住：

```text
App      组织插件和执行流程
World    保存实体、组件和全局资源
System   读取或修改 World
Schedule 决定 System 什么时候运行
```
