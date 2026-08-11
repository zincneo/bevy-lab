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

## Observer 和普通 System 的区别

| 类型 | 运行方式 |
| --- | --- |
| 普通 System | 注册到 Schedule，在计划表运行时执行。 |
| Observer | 注册后等待匹配事件，事件触发时执行。 |

当行为是“每次更新都检查”时使用普通 System；当行为是“某件事发生后立即响应”时，可以使用 Observer。

本 lab 只介绍全局 Observer、普通 `Event` 和 `Commands::trigger`。实体定向事件、事件传播等内容暂不展开。
