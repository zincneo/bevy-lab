# Lab 007：Resource

## 学习目标

`Resource` 用来保存一个 `World` 中按类型唯一的全局数据。本节介绍日常使用 Resource 时最常见的操作：定义、初始化、读取、修改、可选访问、变更检测，以及根据资源是否存在或发生变化来决定 System 是否运行。

## Resource 是什么

Resource 是附着在 `World` 上、同一类型只允许存在一份的数据。它适合表示不属于某个具体实体的状态，例如配置、计数器、时间累计值或当前选项。

```rust
use bevy::prelude::*;

#[derive(Resource, Debug)]
struct Counter {
    value: u32,
}
```

`#[derive(Resource)]` 为类型实现 Bevy 的 `Resource` 特征。当前 Bevy 的 `Resource` 特征建立在 `Component` 之上，因此 Resource 也使用 ECS 的组件存储机制；但使用层面仍然应把它看作 `World` 级别的单例数据，通过 Resource API 或 `Res`、`ResMut` 访问，而不是把它当作普通实体组件查询。

同一个 `World` 中，一个 Resource 类型最多有一个值。再次插入相同类型的 Resource 会替换原来的值：

```rust
app.insert_resource(Counter { value: 1 });
app.insert_resource(Counter { value: 2 }); // 替换为 value = 2
```

## 定义 Resource

Resource 没有规定字段形式，可以是单元结构体、元组结构体或普通结构体：

```rust
#[derive(Resource, Default)]
struct AppSettings {
    sound_enabled: bool,
}

#[derive(Resource, Default)]
struct PauseFlag;

#[derive(Resource, Debug, PartialEq)]
struct Score(u32);
```

是否派生 `Default`、`Debug` 或 `PartialEq` 取决于用途：

- `Default` 让 Resource 可以使用 `init_resource` 初始化；
- `Debug` 便于打印和观察；
- `PartialEq` 让 `set_if_neq` 或按值判断的条件可以使用。

## 初始化 Resource

### `insert_resource`：插入明确的初始值

已经知道初始值时，直接在 `App` 中插入：

```rust
App::new()
    .insert_resource(Score(100))
    .run();
```

如果同一类型已经存在，`insert_resource` 会替换它。

### `init_resource`：使用默认值初始化

Resource 实现 `Default` 时，可以让 Bevy 在不存在时创建它：

```rust
App::new()
    .init_resource::<AppSettings>()
    .run();
```

`init_resource::<T>()` 需要 `T` 实现 `FromWorld`。实现 `Default` 的类型会自动获得默认的 `FromWorld` 实现，因此通常只需派生 `Default`。重复调用 `init_resource` 不会覆盖已经存在的值。

### 自定义 `FromWorld`

当初始值需要根据 `World` 中已有的数据计算时，可以手动实现 `FromWorld`：

```rust
#[derive(Resource)]
struct Limits {
    max_items: usize,
}

impl FromWorld for Limits {
    fn from_world(_world: &mut World) -> Self {
        Self { max_items: 64 }
    }
}

App::new()
    .init_resource::<Limits>()
    .run();
```

`FromWorld::from_world` 会收到当前 `World` 的独占访问，可以读取其他初始化数据后构造 Resource。只需要固定默认值时，优先使用 `Default` 和 `init_resource`。

### 从 `World` 直接初始化

在需要直接操作 `World` 的代码中，也可以使用对应方法：

```rust
let mut world = World::new();
world.insert_resource(Score(10));
world.init_resource::<AppSettings>();
```

普通应用通常通过 `App` 初始化；直接使用 `World` 主要出现在测试、工具代码或 exclusive System 中。

## 在 System 中读取和修改

### `Res<T>`：只读访问

把 `Res<T>` 声明为 System 参数即可读取 Resource：

```rust
fn print_score(score: Res<Score>) {
    println!("当前分数：{}", score.0);
}
```

`Res<T>` 要求该 Resource 已经存在。如果 Resource 没有初始化，System 参数校验会失败；资源可能不存在时，应使用后面介绍的 `Option<Res<T>>` 或 `resource_exists`。

### `ResMut<T>`：可变访问

需要修改 Resource 时使用 `ResMut<T>`，并将参数声明为 `mut`：

```rust
fn increase_score(mut score: ResMut<Score>) {
    score.0 += 1;
}
```

在一个 System 中，对同一 Resource 不能同时取得冲突的读写访问。只读的 `Res<T>` 可以和其他只读参数一起使用；要写入时使用唯一的 `ResMut<T>`。

一个 System 也可以同时读取不同类型的 Resource：

```rust
fn show_status(score: Res<Score>, settings: Res<AppSettings>) {
    println!("score={}, sound={}", score.0, settings.sound_enabled);
}
```

### Resource 作为可选参数

如果某个 Resource 可能尚未插入，可以把参数写成 `Option<Res<T>>` 或 `Option<ResMut<T>>`。缺少 Resource 时 System 仍会运行，参数值为 `None`：

```rust
#[derive(Resource, Default)]
struct DebugLabel(String);

fn print_optional(label: Option<Res<DebugLabel>>) {
    match label {
        Some(label) => println!("调试标签：{}", label.0),
        None => println!("当前没有 DebugLabel Resource"),
    }
}

fn update_optional(mut label: Option<ResMut<DebugLabel>>) {
    if let Some(mut label) = label {
        label.0.push('!');
    }
}
```

这种写法适合确实允许缺少资源的 System。若资源缺失时整个 System 都不应该运行，使用 `resource_exists` 条件通常更清晰。

## 变更检测

Bevy 会为 Resource 记录添加和修改的 tick，System 可以检查本次运行以来 Resource 是否发生变化：

```rust
fn inspect_score(score: Res<Score>) {
    if score.is_added() {
        println!("Score 刚被添加");
    }
    if score.is_changed() {
        println!("Score 自上次运行后发生了变化");
    }
}
```

- `is_added()`：Resource 在相关 System 上次运行后被添加；
- `is_changed()`：Resource 在相关 System 上次运行后被添加或修改。

通过 `ResMut<T>` 修改字段会标记 Resource 已改变，即使新值与旧值相同。如果类型实现了 `PartialEq`，可以使用 `set_if_neq`，只有值真的不同时才写入并标记变化：

```rust
fn set_score(mut score: ResMut<Score>) {
    let new_score = Score(100);
    if score.set_if_neq(new_score) {
        println!("分数确实发生了变化");
    }
}
```

变更检测只针对每个 System 自己记录的上次运行位置。一个 System 读取过 Resource 后，另一个 System 的修改不会让第一个 System 永久保持“已改变”；它会在下一次运行时重新按照 tick 判断。

## 用 Resource 控制 System 是否运行

Bevy 提供了几个常用的 Resource 条件，可以传给 `run_if`：

```rust
fn show_settings(settings: Res<AppSettings>) {
    println!("sound_enabled={}", settings.sound_enabled);
}

App::new()
    .init_resource::<AppSettings>()
    .add_systems(
        Update,
        show_settings.run_if(resource_exists::<AppSettings>),
    )
    .run();
```

常用条件如下：

| 条件 | 作用 |
| --- | --- |
| `resource_exists::<T>` | 只有 Resource `T` 存在时运行。 |
| `resource_added::<T>` | 只有 Resource `T` 在上次运行后被添加时运行。 |
| `resource_changed::<T>` | 只有 Resource `T` 在上次运行后被添加或修改时运行。 |
| `resource_equals(value)` | Resource 存在且等于给定值时运行，需要 `PartialEq`。 |
| `resource_exists_and_equals(value)` | Resource 存在且等于给定值时运行；不存在时不会运行。 |
| `resource_exists_and_changed::<T>` | Resource 存在且在上次运行后被添加或修改时运行。 |

`resource_exists` 特别适合保护带有 `Res<T>` 或 `ResMut<T>` 的 System，避免在 Resource 尚未准备好时访问它。`resource_added` 和 `resource_changed` 适合把初始化或响应更新的逻辑限制在真正需要的时机。

`resource_equals` 适用于能够确定 Resource 已存在的情况；如果 Resource 可能缺少，应改用 `resource_exists_and_equals`。同理，Resource 可能被移除时可使用 `resource_exists_and_changed`，让条件在缺少 Resource 时返回 `false`。

## 通过 `World` 读取、替换和移除

在普通 System 中优先使用 `Res` 和 `ResMut`。需要直接取得 `World` 时，可以使用 exclusive System：

```rust
fn inspect_world(world: &mut World) {
    if let Some(score) = world.get_resource::<Score>() {
        println!("读取：{}", score.0);
    }

    if let Some(mut score) = world.get_resource_mut::<Score>() {
        score.0 += 1;
    }
}
```

`World` 上最常用的 Resource 方法是：

| 方法 | 作用 |
| --- | --- |
| `insert_resource(value)` | 插入或替换指定类型的 Resource。 |
| `init_resource::<T>()` | Resource 不存在时使用 `FromWorld` 初始化。 |
| `contains_resource::<T>()` | 判断指定类型的 Resource 是否存在。 |
| `get_resource::<T>()` | 尝试取得只读引用，不存在时返回 `None`。 |
| `get_resource_mut::<T>()` | 尝试取得可变引用，不存在时返回 `None`。 |
| `remove_resource::<T>()` | 移除并返回 Resource，不存在时返回 `None`。 |

`resource::<T>()` 和 `resource_mut::<T>()` 是不带 `Option` 的快捷访问方式，Resource 不存在时会失败。只有能够确定 Resource 已初始化时才使用它们。

## 通过 `Commands` 延迟管理 Resource

系统中需要在命令应用时插入、初始化或移除 Resource，可以使用 `Commands`：

```rust
fn change_resources(mut commands: Commands) {
    commands.insert_resource(Score(0));
    commands.init_resource::<AppSettings>();
    commands.remove_resource::<DebugLabel>();
}
```

这些操作和实体结构修改一样会进入命令队列，在 Bevy 应用合适的时机应用。`Commands` 的延迟执行、实体操作和自定义命令见 Lab 005；本节只需记住 Resource 也有对应的 `insert_resource`、`init_resource` 和 `remove_resource` 命令。

Resource 实现 `PartialEq` 时，还可以使用 `insert_resource_if_neq(value)`，只有新旧值不同时才替换并标记变化。

## 一个最小的使用流程

Resource 的常见生命周期可以概括为：

```text
定义类型并派生 Resource
        ↓
App::insert_resource 或 App::init_resource
        ↓
System 用 Res 读取、用 ResMut 修改
        ↓
必要时用 Option、run_if 或变更检测处理可选和更新逻辑
        ↓
需要结构性变更时用 Commands 插入或移除
```

选择 API 时可以遵循下面的简单规则：

1. 有明确初始值，用 `insert_resource`；需要默认构造，用 `init_resource`。
2. 只读用 `Res<T>`，修改用 `ResMut<T>`。
3. Resource 可能不存在，用 `Option<Res<T>>`；不存在时整个 System 不运行，用 `resource_exists`。
4. 只想响应更新，用 `is_changed` 或 `resource_changed`。
5. 在 System 中改变 Resource 的存在性，用 `Commands`；在 exclusive System、测试或初始化代码中直接使用 `World` 方法。
