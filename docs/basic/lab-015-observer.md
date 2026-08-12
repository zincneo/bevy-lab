# Lab 015：Observer

## 学习目标

了解 Bevy Observer 的基本用途：定义一个事件类型，注册 Observer，并在事件被触发时立即执行响应逻辑。

## Observer 是什么

Observer 是监听事件的 System。它不会按照 `Update` 每帧运行，而是在匹配的事件被触发时运行。

事件类型使用 `#[derive(Event)]`：

```rust
#[derive(Event)]
struct Greeting {
    message: String,
}
```

Observer 使用 `On<EventType>` 作为参数读取事件：

```rust
fn observe_greeting(event: On<Greeting>) {
    println!("{}", event.message);
}
```

## 注册全局 Observer

使用 `App::add_observer` 注册一个监听整个 World 的 Observer：

```rust
App::new()
    .add_plugins(MinimalPlugins)
    .add_observer(observe_greeting)
    .run();
```

Observer 仍然可以使用普通 System 参数，例如 `ResMut`、`Query` 和 `Commands`。

## 触发事件

可以通过 `Commands::trigger` 触发事件：

```rust
fn trigger_greeting(mut commands: Commands) {
    commands.trigger(Greeting {
        message: "Hello".to_string(),
    });
}
```

触发命令被应用后，匹配的 Observer 会执行，并读取这次事件携带的数据。本示例在 `Startup` 中触发两个事件，Observer 分别打印两次消息。

## 给 Entity 添加 Observer

除了使用 `App::add_observer` 注册全局 Observer，还可以把 Observer 绑定到某一个
Entity。绑定时使用 `EntityCommands::observe`：

```rust
let listener = commands
    .spawn_empty()
    .observe(observe_entity_greeting)
    .id();
```

这个 Observer 不会监听所有 Entity 的同类事件，只有事件的目标是 `listener` 时才会
执行。

## 触发 Entity 上的 Observer

要让事件拥有明确的目标，需要使用 `#[derive(EntityEvent)]` 定义事件。对于包含名为
`entity` 的 `Entity` 字段的结构体，Bevy 会自动把这个字段当作事件目标：

```rust
#[derive(EntityEvent)]
struct EntityGreeting {
    entity: Entity,
    message: String,
}
```

触发时通过 `Commands::trigger` 传入目标 Entity：

```rust
commands.trigger(EntityGreeting {
    entity: listener,
    message: "Hello entity".to_string(),
});
```

Entity 上的 Observer 使用同样的 `On<EventType>` 参数读取事件：

```rust
fn observe_entity_greeting(event: On<EntityGreeting>) {
    println!("{} received: {}", event.entity, event.message);
}
```

`EntityEvent` 仍然属于 Bevy 的 `Event`，只是比普通 `Event` 多了一个明确的目标。
触发一个 EntityEvent 时，全局注册的同类型 Observer 和目标 Entity 上注册的
Observer 都有机会收到它；本示例只给目标 Entity 添加监听，因此只有该 Entity 的
Observer 响应。

完整流程可以概括为：

```text
创建 Entity
    ↓
EntityCommands::observe 添加监听
    ↓
EntityEvent 携带目标 Entity
    ↓
Commands::trigger 触发事件
    ↓
目标 Entity 上的 Observer 执行
```

## Observer 和普通 System 的区别

| 类型 | 运行方式 |
| --- | --- |
| 普通 System | 注册到 Schedule，在计划表运行时执行。 |
| Observer | 注册后等待匹配事件，事件触发时执行。 |

当行为是“每次更新都检查”时使用普通 System；当行为是“某件事发生后立即响应”时，可以使用 Observer。

本 lab 介绍全局 Observer、Entity Observer、普通 `Event`、`EntityEvent` 和
`Commands::trigger`。事件沿父子层级传播等更复杂用法暂不展开。
