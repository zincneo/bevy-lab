# Lab 003：System 函数

## System 函数是什么

在 Bevy 中，System 是一段可以被 App 调度执行的逻辑。最常见的 System 就是一个普通 Rust 函数：

```rust
fn print_message() {
    println!("System 正在运行");
}
```

将它注册到 App 后，Bevy 会在对应的 Schedule 中调用它：

```rust
app.add_systems(Update, print_message);
```

普通函数能否注册为 System，主要取决于它的参数和返回值：

- 参数可以有 0 个、1 个或多个；
- 参数类型可以不同，但每个参数都必须实现 Bevy 的 `SystemParam` 特征；
- 普通 Schedule System 通常返回 `()`。

因此，编写 System 时只需要在函数签名中声明需要的数据，Bevy 会在运行时准备这些参数。参数数量和顺序没有固定要求，`Commands` 也不要求必须放在第一个参数位置。

```rust
fn update_score(score: Res<Score>, mut runs: Local<u32>) {
    *runs += 1;
    println!("分数：{}，运行次数：{}", score.0, *runs);
}
```

## `add_systems` 可以注册什么

`add_systems` 的第一个参数是 Schedule 标签，第二个参数是要加入该 Schedule 的 System：

```rust
app.add_systems(Startup, setup)
    .add_systems(Update, update);
```

除了普通函数，也可以注册符合相同参数规则的闭包，或者把多个 System 组成元组：

```rust
app.add_systems(Update, (read_input, update_player));
app.add_systems(Update, || println!("闭包 System"));
```

本实验只关注最常见的普通函数；System 参数的具体实现细节属于 Bevy 内部机制，不需要在使用层面展开理解。

## Schedule 与 System 的关系

Schedule 是 System 的执行计划表，负责决定一组 System 什么时候运行。App runner 会按照应用配置执行这些计划：

```text
App 启动
    ↓
Startup：启动阶段，通常只执行一次
    ↓
主循环
    ├─ Update：每次更新执行
    ├─ FixedUpdate：按固定时间步执行（启用时）
    └─ 其他自定义 Schedule
```

调用 `add_systems` 只是把 System 登记到计划表，并不会在这一行立即执行。没有窗口或其他持续运行插件的最小 App，默认 runner 可能只执行一次更新，因此示例中的 `Update` System 也可能只打印一次。

同一个 Schedule 中的 System 默认不等于按注册顺序执行。Bevy 会根据 System 的数据访问关系安排执行；没有冲突的 System 可以被并行调度，有冲突的访问会被错开。如果业务逻辑要求明确顺序，可以使用 `.chain()`、`.before()` 或 `.after()`：

```rust
app.add_systems(Update, (read_input, update_player).chain());
```

`chain()` 表示先执行 `read_input`，再执行 `update_player`。具体的并行调度规则会在后续实验逐步展开。

## 常用 System 参数

以下参数是编写普通 System 时最常见的几类：

| 参数 | 作用 | 常见写法 |
| --- | --- | --- |
| `Res<T>` | 只读访问 World 中的全局 Resource。 | `Res<Score>` |
| `ResMut<T>` | 可变访问全局 Resource。 | `ResMut<Score>` |
| `Commands` | 提交创建实体、修改组件或 Resource 等延迟操作。详细用法见 Lab 004。 | `Commands` |
| `Local<T>` | 保存当前 System 私有、跨多次运行保留的状态，不是 World 中的共享 Resource。 | `Local<u32>` |
| `MessageReader<M>` | 读取已注册的 `Messages<M>` Resource。 | `MessageReader<GameMessage>` |
| `MessageWriter<M>` | 向已注册的 `Messages<M>` Resource 写入消息。 | `MessageWriter<GameMessage>` |

## 最小示例中的参数组合

下面的示例只使用 `Res`、`ResMut` 和 `Local`，展示 System 如何通过参数访问 World 中的 Resource，以及 App 如何按照 Startup 和 Update 执行它们：

```rust
use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
struct Counter(u32);

fn setup(mut counter: ResMut<Counter>) {
    counter.0 = 1;
    println!("Startup：初始化 Counter({})", counter.0);
}

fn update(counter: Res<Counter>, mut runs: Local<u32>) {
    *runs += 1;
    println!("Update：Counter({})，System 运行 {} 次", counter.0, *runs);
}

struct SystemOverviewPlugin;

impl Plugin for SystemOverviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, update);
    }
}

fn main() {
    App::new()
        .insert_resource(Counter::default())
        .add_plugins(SystemOverviewPlugin)
        .run();
}
```

这个示例中，`setup` 和 `update` 都是 System；`Startup` 和 `Update` 是 Schedule；`ResMut<Counter>`、`Res<Counter>` 和 `Local<u32>` 是由 Bevy 自动准备的 System 参数。

## 小结

System 是读取或修改 World 数据的可调度逻辑，Schedule 负责安排它的执行时机，App runner 负责驱动这些 Schedule。编写普通 System 时，只需要声明实现了 `SystemParam` 的参数，并把函数注册到合适的 Schedule；参数的数量和类型不需要固定。
