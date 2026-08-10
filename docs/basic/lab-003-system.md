# Lab 003：System 函数

## System 函数是什么

在 Bevy 中，System（系统）是“读取 World 中的数据并执行一段逻辑”的可调度单元。最常见的写法是一个普通 Rust 函数：

```rust
fn update_score(mut query: Query<&mut Score>, round: Res<Round>) {
    for mut score in &mut query {
        score.0 += round.0;
    }
}
```

这个函数本身只是一个 Rust 函数值。传给 `App::add_systems` 后，Bevy 会根据它的函数签名把它转换成内部的 `System`，记录它需要访问的 World 数据，并把它加入指定的 Schedule：

```rust
app.add_systems(Update, update_score);
```

因此，System 不等同于“任何可以调用的函数”。它必须符合 Bevy 对系统输入、系统参数和返回值的约束，才能被 `add_systems` 接受。

## `add_systems` 可以注册什么

`add_systems` 的第一个参数是调度标签，例如 `Startup`、`Update` 或项目自定义的 Schedule；第二个参数可以是一个系统，也可以是多个系统组成的元组或带有调度配置的系统集合。使用时通常只需要关注下面这种调用方式，不需要记住内部泛型签名：

```rust
app.add_systems(Update, update_score);
```

### 普通函数

普通函数的每个参数都必须是 Bevy 支持的 `SystemParam`，返回值通常是 `()`：

```rust
fn no_parameter_system() {}

fn gameplay_system(
    time: Res<Time>,
    mut query: Query<&mut Position>,
    mut commands: Commands,
) {
    // 读取资源、查询组件，并提交延迟修改。
}

app.add_systems(Update, (no_parameter_system, gameplay_system));
```

参数个数可以不同，参数类型也可以不同，但每一种类型都必须实现 `SystemParam`。参数的顺序通常没有特殊要求；例如 `Commands` 不要求必须是第一个参数。

### 闭包

满足相同系统参数规则的闭包也可以注册：

```rust
app.add_systems(Update, |mut commands: Commands| {
    commands.spawn_empty();
});
```

闭包需要能够被 Bevy 长期保存和重复调用。捕获局部变量时，通常需要使用 `move`，并且被捕获的值需要满足相应的生命周期和线程安全要求：

```rust
let label = String::from("spawned");
app.add_systems(Update, move || println!("{label}"));
```

不能把只在当前函数栈帧中存在的引用直接捕获进将来才运行的系统。

### 独占 `World` 的系统

需要直接取得整个 `World` 可变访问权时，可以写成独占系统：

```rust
fn inspect_world(world: &mut World) {
    println!("实体数量：{}", world.entities().len());
}

app.add_systems(Update, inspect_world);
```

独占系统不是普通 `SystemParam` 系统。它会在执行时取得 `&mut World`，因此不能与其他访问 World 的系统并行执行；Bevy 会把它标记为 exclusive，并在合适的时机单独运行。独占函数的 `&mut World` 必须位于参数列表开头，后面只能跟允许用于独占系统的参数，例如 `Local`、`&mut QueryState` 或 `&mut SystemState`。

### 系统元组和配置

多个系统可以组成元组传给 `add_systems`：

```rust
app.add_systems(Update, (read_input, update_player, render_ui));
```

多个系统组成的元组还可以继续配置执行关系：

```rust
app.add_systems(
    Update,
    (read_input, update_player, render_ui)
        .chain()
        .run_if(in_state(GameState::Playing)),
);
```

默认情况下，调度器会根据系统的访问冲突决定哪些系统可以并行运行；`.chain()`、`.before()`、`.after()` 和 `.in_set()` 可以表达额外的顺序或分组要求。

### 返回值

通过 `add_systems` 加入普通 Schedule 的系统最终必须输出 `()`。当前 Bevy 也允许系统返回 `Result<(), BevyError>` 或 `Result<(), RunSystemError>`，由系统运行器统一处理错误：

```rust
fn fallible_system() -> Result<(), BevyError> {
    // 发生错误时返回 Err(...)
    Ok(())
}
```

直接返回任意业务值的函数不能作为普通 Schedule 系统：

```rust
fn returns_a_value() -> u32 {
    42
}

// 不能直接 app.add_systems(Update, returns_a_value);
```

需要传递系统输出时，应使用专门的系统组合方式，这属于后续内容。

## Schedule（计划表）与 System 执行

`Schedule` 可以理解为一张 System 的执行计划表。`add_systems` 只是把 System 登记到指定的计划表中，并不会在调用这一行时立即执行函数：

```rust
app.add_systems(Startup, setup);
app.add_systems(Update, update_player);
```

Bevy 的 App runner 会在应用运行过程中执行这些计划表。常见的执行关系是：

```text
应用启动
    ↓
Startup：启动系统，通常只执行一次
    ↓
主循环
    ├─ Update：每次更新执行一次
    ├─ FixedUpdate：按固定时间步执行（启用时）
    └─ 其他已配置的计划表
```

一个 Schedule 被执行时，大致遵循以下规则：

1. 检查 System 的运行条件；条件不满足的 System 会被跳过。
2. 根据 System 声明的参数访问关系安排执行。互不冲突的 System 可以同时执行，存在读写冲突的 System 需要错开。
3. 遵守 `.chain()`、`.before()`、`.after()` 和 SystemSet 配置的明确顺序。
4. 在调度边界应用 `Commands` 等延迟修改，之后的 System 才能看到已经应用的结构变化。

因此，System 在 `add_systems` 中的书写顺序不等于实际执行顺序。如果两个 System 没有冲突，也没有使用顺序配置，通常不应依赖它们的先后关系：

```rust
app.add_systems(Update, (read_input, update_player).chain());
```

上面的 `.chain()` 才明确表示 `read_input` 先执行，`update_player` 后执行。计划表本身负责“什么时候运行哪些 System”，System 函数则负责“运行时要做什么”。

## 参数数量和类型

一个可以注册到 Schedule 的普通 System 函数可以有：

- 0 个参数，例如 `fn update() {}`；
- 1 个参数，例如 `fn update(time: Res<Time>) {}`；
- 多个参数，例如 `fn update(time: Res<Time>, query: Query<Entity>, commands: Commands) {}`。

这些参数不需要是同一种类型。它们只需要分别实现 Bevy 的 `SystemParam` 特征，也就是 Bevy 知道如何在运行系统时为它们准备对应的数据。常用的 `Query`、`Res`、`ResMut`、`Commands` 和 `Local` 都已经实现了这个特征。

因此，编写系统时只需要在函数签名中声明需要的参数：

```rust
fn update_player(
    time: Res<Time>,
    mut players: Query<&mut Position, With<Player>>,
    mut commands: Commands,
) {
    // Bevy 会在系统运行时提供这三个参数。
}
```

不需要自己创建这些参数，也不需要关心 Bevy 内部如何支持不同的参数数量。参数顺序通常也没有特殊要求；`Commands` 不需要放在第一个位置。

需要注意的是，`&mut World` 是独占系统的特殊写法，不属于普通 `SystemParam` 参数。它会让整个 World 由当前系统独占访问，因此不能和其他系统并行运行。

系统参数还会帮助 Bevy 判断系统之间是否存在数据访问冲突，但学习和编写应用时只需要正确选择参数的读写形式即可：`Res` 和只读 `Query` 表示读取，`ResMut` 和带有可变组件的 `Query` 表示写入。

## 常用 System 参数

下面列出日常编写 Bevy 系统时最常见的参数。它们都可以作为普通系统函数的参数，除非特别说明。

| 参数 | 作用 | 常见写法 |
| --- | --- | --- |
| `Query<D, F>` | 按组件数据 `D` 查询实体；过滤器 `F` 用于限制匹配范围。`&T` 只读，`&mut T` 可写。 | `Query<(&Transform, &mut Velocity), With<Player>>` |
| `Single<D, F>` | 查询必须恰好匹配一个实体；适合场景中确定只有一个的对象。 | `Single<&mut Camera>` |
| `Populated<D, F>` | 查询至少要匹配一个实体；没有匹配实体时系统会跳过。 | `Populated<&Enemy>` |
| `Res<T>` | 读取一个全局 Resource；资源必须已经存在。 | `Res<Time>` |
| `ResMut<T>` | 可变访问一个全局 Resource；会形成写访问并参与变更检测。 | `ResMut<Score>` |
| `Commands` | 提交实体、组件和 Resource 的延迟修改；命令在安全边界应用。 | `Commands` |
| `Local<T>` | 保存该系统私有、跨多次运行保持的状态；不会成为 World 中的共享 Resource。 | `Local<usize>` |

### Query类型的泛型参数用法

`Query` 在系统参数中通常写成 `Query<D, F>`。它的完整类型还包含由 Bevy 管理的生命周期参数：

```rust
Query<'world, 'state, D, F = ()>
```

编写普通系统时不需要手动填写 `'world` 和 `'state`，只需要关注 `D` 和 `F`：

- `D`（Query Data）说明查询要从每个实体中取出什么数据；
- `F`（Query Filter）说明实体还必须满足哪些条件，默认值 `()` 表示没有额外过滤。

#### `D`：查询数据

`D` 中的类型决定了查询结果的内容。常见写法如下：

| `D` 写法 | 查询结果和用途 |
| --- | --- |
| `Entity` | 只获取实体 ID，不读取组件。 |
| `&Component` | 以只读方式访问组件。多个系统可以同时读取。 |
| `&mut Component` | 以可变方式访问组件，用于修改数据。会产生写访问。 |
| `(&A, &B)` | 同时读取多个组件；实体必须同时拥有这些组件。 |
| `(Entity, &A)` | 同时获取实体 ID 和组件。 |
| `Option<&A>` | 可选组件；缺少该组件的实体仍然可以匹配，结果为 `None`。 |
| `(&A, Option<&B>)` | 必需组件和可选组件组合使用。 |

例如，下面的查询会获取实体 ID、必需的 `Transform`，以及可能存在的 `Health`：

```rust
fn inspect_players(
    query: Query<(Entity, &Transform, Option<&Health>)>,
) {
    for (entity, transform, health) in &query {
        println!("{entity:?}: {transform:?}, health={health:?}");
    }
}
```

只需要读取时使用 `&A`；需要修改时才使用 `&mut A`。例如：

```rust
fn move_players(mut query: Query<&mut Transform>) {
    for mut transform in &mut query {
        transform.translation.x += 1.0;
    }
}
```

#### `F`：查询过滤器

`F` 不会出现在查询结果中，只负责决定哪些实体能够进入查询：

| `F` 写法 | 匹配条件 |
| --- | --- |
| `()` | 不添加额外条件，匹配所有拥有 `D` 所需数据的实体。 |
| `With<A>` | 实体必须拥有组件 `A`。 |
| `Without<A>` | 实体不能拥有组件 `A`。 |
| `Added<A>` | 组件 `A` 必须是最近添加的。 |
| `Changed<A>` | 组件 `A` 必须自该系统上次运行以来发生变化。 |
| `Spawned` | 实体必须是最近创建的。 |
| `Or<(With<A>, With<B>)>` | 满足其中任意一个过滤条件即可。 |
| `Allow<A>` | 绕过 World 的 `DefaultQueryFilters` 对组件 `A` 的默认排除；属于特殊用途。 |

多个过滤器可以写成元组，表示同时满足所有条件（AND）：

```rust
fn update_players(
    query: Query<&mut Transform, (With<Player>, Without<Disabled>)>,
) {
    // 只处理 Player 且没有 Disabled 的实体。
    for mut transform in query {
        transform.translation.x += 1.0;
    }
}
```

`Or<(...)>` 可以把多个过滤器组合成“满足任意一个条件”（OR）：

```rust
fn inspect_targets(
    query: Query<Entity, Or<(With<Player>, With<Enemy>)>>,
) {
    // Player 或 Enemy 都会被查询到。
    for entity in &query {
        println!("目标：{entity:?}");
    }
}
```

`Spawned` 用于只处理最近创建的实体，适合执行一次性的初始化逻辑：

```rust
fn initialize_new_entities(query: Query<Entity, Spawned>) {
    for entity in &query {
        println!("初始化新实体：{entity:?}");
    }
}
```

### 自定义和特殊过滤器

如果多个系统反复使用同一组条件，可以使用 `#[derive(QueryFilter)]` 定义一个可复用的过滤器：

```rust
#[derive(QueryFilter)]
struct ActivePlayer {
    player: With<Player>,
    not_disabled: Without<Disabled>,
}

fn update_active_players(query: Query<Entity, ActivePlayer>) {
    // 只匹配 Player 且没有 Disabled 的实体。
    for entity in &query {
        println!("活动玩家：{entity:?}");
    }
}
```

`Allow<A>` 不是普通 gameplay 过滤器，而是和 Bevy 的默认实体过滤机制配合使用的特殊类型。例如 World 配置了默认排除 `Disabled` 实体时，`Allow<Disabled>` 可以让某个查询重新包含这些实体。初学阶段通常不需要使用它。

`Has<A>` 和 `AnyOf<...>` 看起来也像“判断组件是否存在”，但它们属于 `D` 查询数据，而不是 `F` 过滤器：前者把存在性作为查询结果的一部分，后者允许从多个可选组件中取结果。

#### 查询结果的常用访问方式

- `for item in &query` 或 `query.iter()`：遍历只读结果；
- `for item in &mut query` 或 `query.iter_mut()`：遍历可变结果；
- `query.get(entity)`：根据实体 ID 获取一个只读结果；
- `query.get_mut(entity)`：根据实体 ID 获取一个可变结果；
- `query.single()` / `query.single_mut()`：要求恰好匹配一个实体，否则返回错误。

`Single<D, F>` 和 `Populated<D, F>` 是 Query 的相关系统参数：前者在参数获取时就要求恰好一个匹配实体，后者要求至少有一个匹配实体。它们使用的 `D` 和 `F` 含义与 `Query` 相同。


### `Commands` 与读取参数的区别

`Commands` 用来提交延迟修改，不能代替 `Query`、`Res` 等读取参数：

```rust
fn system(mut commands: Commands, query: Query<Entity>, score: Res<Score>) {
    // query 和 score 读取当前系统获得的 World 数据。
    // commands.spawn_empty() 只提交未来创建实体的请求。
    let _ = (&query, &score);
    commands.spawn_empty();
}
```

参数顺序不决定执行顺序，也不会让 `Commands` 变成即时的 World 引用。系统之间的执行顺序应通过 Schedule 配置、SystemSet 或明确的依赖表达。

## 小结

可以被 `App::add_systems` 注册的，不是任意函数，而是符合 Bevy System 规则的函数、闭包、独占函数或系统组合。普通系统的参数必须实现 `SystemParam`，返回值最终必须是 `()` 或受支持的错误结果。

普通 System 可以拥有数量不同、类型不同的参数，关键条件是每个参数都实现了 `SystemParam`。Bevy 会在系统运行时准备这些参数；学习和编写应用时不需要关注其内部实现。参数的读写形式还会帮助调度器进行冲突检测和多处理器并行调度。
