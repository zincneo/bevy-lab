# Lab 010：run_if

## 学习目标

`run_if` 是 Bevy 为 System 提供的运行条件接口。它会在 System 准备执行前调用一个返回 `bool` 的条件 System：条件为 `true` 时执行目标 System，条件为 `false` 时跳过目标 System。

```rust
fn update_score() {
    println!("更新分数");
}

fn can_update() -> bool {
    true
}

app.add_systems(Update, update_score.run_if(can_update));
```

`run_if` 控制的是“要不要执行整个 System”，不是在 System 内部自动生成一个 `if` 分支。如果只是逻辑中的一小段需要条件判断，直接在函数中使用 `if` 更合适；如果整个 System 在条件不满足时都没有工作，就使用 `run_if`。

## 条件 System

传给 `run_if` 的条件本身也是一个 System，但它必须返回 `bool`，并且只能通过只读的 System 参数观察 `World`：

```rust
#[derive(Resource)]
struct Permission {
    allowed: bool,
}

fn has_permission(permission: Res<Permission>) -> bool {
    permission.allowed
}

fn protected_system() {
    println!("权限满足，执行受保护逻辑");
}

app.insert_resource(Permission { allowed: true })
    .add_systems(Update, protected_system.run_if(has_permission));
```

条件可以使用 `Res<T>`、只读 `Query`、`State` 等只读参数，但不应该使用 `ResMut<T>`、`Commands` 或可变 Query 去修改 World。条件的职责是判断，不是执行修改。

条件会在每次对应 Schedule 运行时重新判断，因此资源、状态或实体发生变化后，System 是否运行也会随之变化。

## `run_if` 的基本写法

### 无参数条件

最简单的条件不需要访问 World：

```rust
fn is_enabled() -> bool {
    true
}

app.add_systems(Update, update_score.run_if(is_enabled));
```

也可以直接使用闭包：

```rust
app.add_systems(Update, update_score.run_if(|| true));
```

### 使用 Resource 的条件

条件可以读取 Resource 的值：

```rust
#[derive(Resource, PartialEq)]
struct Mode(u8);

fn is_playing(mode: Res<Mode>) -> bool {
    mode.0 == 1
}

app.add_systems(Update, update_score.run_if(is_playing));
```

如果条件需要的 Resource 可能不存在，应使用 `Option<Res<T>>`，而不是直接使用会要求 Resource 必须存在的 `Res<T>`：

```rust
fn has_mode(mode: Option<Res<Mode>>) -> bool {
    mode.is_some()
}
```

Bevy 的 `resource_exists::<T>` 就是这种模式的通用实现。

### 使用 Query 的条件

条件也可以根据实体是否存在来判断：

```rust
#[derive(Component)]
struct Target;

fn has_target(query: Query<(), With<Target>>) -> bool {
    !query.is_empty()
}

app.add_systems(Update, update_score.run_if(has_target));
```

如果只需要判断是否存在某类组件，可以直接使用内置的 `any_with_component::<Target>`。

## 常用内置条件

Bevy 在 `prelude` 中提供了一些日常可以直接使用的条件：

| 条件 | 作用 |
| --- | --- |
| `in_state(value)` | 只有当前状态等于 `value` 时运行。 |
| `resource_exists::<T>` | Resource `T` 存在时运行。 |
| `resource_added::<T>` | Resource `T` 在上次运行后刚被添加时运行。 |
| `resource_changed::<T>` | Resource `T` 在上次运行后被添加或修改时运行。 |
| `resource_equals(value)` | Resource 存在且等于指定值时运行。 |
| `on_message::<T>` | 有新的 `T` 消息可读取时运行。 |
| `any_with_component::<T>` | World 中至少有一个实体拥有组件 `T` 时运行。 |
| `run_once` | 只在第一次满足条件时运行一次。 |
| `not(condition)` | 反转另一个条件的结果。 |

这些条件分别对应前面介绍的 State、Resource、Message 和 Query 的常见场景：

```rust
app.add_systems(Update, playing_system.run_if(in_state(AppMode::Playing)))
    .add_systems(Update, load_system.run_if(resource_exists::<AssetsReady>))
    .add_systems(Update, refresh_system.run_if(on_message::<RefreshRequested>))
    .add_systems(Update, cleanup_system.run_if(not(any_with_component::<Target>)));
```

`resource_equals` 要求 Resource 已经存在；如果 Resource 可能缺少，可以使用 `resource_exists_and_equals(value)`。状态条件 `in_state` 在状态不存在时会返回 `false`。

## 条件组合

### 多次调用 `run_if`：全部满足

同一个 System 可以添加多个 `run_if`。这些条件需要全部为 `true`，System 才会运行：

```rust
app.add_systems(
    Update,
    update_score
        .run_if(in_state(AppMode::Playing))
        .run_if(resource_exists::<Score>),
);
```

上例表示“当前处于 Playing 且 Score Resource 存在”时才执行。多个条件的结果相当于逻辑 AND。

### `and_then`：按顺序组合并短路

也可以显式组合条件：

```rust
let can_update = in_state(AppMode::Playing)
    .and_then(resource_exists::<Score>);

app.add_systems(Update, update_score.run_if(can_update));
```

`and_then` 在前一个条件为 `false` 时不会调用后一个条件，适合后一个条件依赖前一个条件已经成立的情况。

### `or_else`：满足任意一个

`or_else` 表示两个条件至少有一个为 `true`：

```rust
let can_show = in_state(AppMode::Menu)
    .or_else(in_state(AppMode::Paused));

app.add_systems(Update, show_overlay.run_if(can_show));
```

上例在 Menu 或 Paused 状态下都会执行 `show_overlay`。

### `not`：反转条件

```rust
app.add_systems(
    Update,
    idle_system.run_if(not(in_state(AppMode::Playing))),
);
```

只有不在 Playing 状态时，`idle_system` 才会运行。

## 多个 System 的条件范围

### 给单个 System 添加条件

最常见的方式是直接附加在函数上：

```rust
app.add_systems(Update, update_player.run_if(in_state(AppMode::Playing)));
```

### 给一组 System 添加共同条件

对一个 System 元组调用 `run_if` 时，条件作用于整组 System。条件为 `false` 时整组跳过，条件每次 Schedule 运行最多评估一次：

```rust
app.add_systems(
    Update,
    (update_player, update_camera).run_if(in_state(AppMode::Playing)),
);
```

这相当于把两个 System 放进同一个带条件的 System Set，适合一组逻辑必须一起启用或停用的场景。

### `distributive_run_if`

如果希望元组中的每个 System 分别评估条件，可以使用 `distributive_run_if`：

```rust
app.add_systems(
    Update,
    (update_player, update_camera).distributive_run_if(in_state(AppMode::Playing)),
);
```

这里每个 System 都有一份独立的条件。大多数情况下，普通的 `run_if` 更容易理解；只有需要每个 System 分别判断时才使用 `distributive_run_if`。

### 给 System Set 添加条件

System 数量较多时，可以先定义一个 Set，再统一配置条件：

```rust
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct GameplaySet;

app.configure_sets(
        Update,
        GameplaySet.run_if(in_state(AppMode::Playing)),
    )
    .add_systems(Update, (update_player, update_camera).in_set(GameplaySet));
```

Set 条件适合集中管理一组相关逻辑。与元组上的 `run_if` 一样，条件默认作为这组 System 的共同条件。

## `run_if` 与 System 内部 `if` 的区别

两种写法都可以表达条件逻辑，但作用范围不同：

```rust
fn update() {
    if should_update() {
        // 只有这一小段被跳过
    }
}

app.add_systems(Update, update.run_if(should_update));
```

- System 内部的 `if`：System 仍然会被调度和调用，只是函数内部跳过一部分代码；
- `run_if`：条件为 `false` 时目标 System 不会执行，适合整个 System 都不需要工作的情况。

条件判断本身也是一个只读 System，会参与调度器的访问分析。因此，条件应该保持简单，尽量只读取必要的状态，不要在条件中完成实际业务修改。

## 常见注意事项

1. `run_if` 是 System 配置方法，必须在注册 System 时使用，例如 `system.run_if(condition)`。
2. 条件返回 `false` 时，目标 System 不会运行，目标 System 的 `Local` 状态也不会推进。
3. 条件使用的 Resource、State 或 Message 必须先初始化；可能不存在时使用可选参数或对应的 `resource_exists`/`state_exists` 条件。
4. 同一个 Schedule 中，如果一个 System 会修改条件读取的 Resource，应使用 `.before()`、`.after()` 或 `.chain()` 明确执行顺序。
5. `run_if` 只决定是否执行 System，不会改变 System 所属的 Schedule；目标 System 仍然注册在原来的 `Startup`、`Update` 或其他 Schedule 中。

## 一个最小的使用流程

```text
准备一个返回 bool 的条件 System
        ↓
把条件附加到目标 System、System 元组或 System Set
        ↓
每次 Schedule 运行前评估条件
        ↓
true  → 执行目标 System
false → 跳过目标 System
```

日常使用时可以优先记住：状态用 `in_state`，资源用 `resource_*`，消息用 `on_message`，实体存在性用 `any_with_component`，多个条件用多个 `run_if` 或 `and_then`/`or_else` 组合。
