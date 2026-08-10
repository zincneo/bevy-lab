# Lab 006：Query

## 学习目标

`Query` 是 System 访问实体和组件的主要方式。使用 Query 时，大多数实体处理都可以归纳为：

1. 找到拥有某些组件的实体；
2. 读取或修改这些组件；
3. 根据标记、状态或变更情况缩小匹配范围。

本节覆盖日常使用最常见的 Query 写法。

## `Query<D, F>` 的两个泛型参数

System 中通常这样声明 Query：

```rust
fn move_player(query: Query<(&mut Position, &Velocity), With<Player>>) {
    // ...
}
```

`Query` 的两个主要泛型参数是：

| 参数 | 名称 | 作用 |
| --- | --- | --- |
| `D` | Query Data | 每个匹配实体需要取出的数据。 |
| `F` | Query Filter | 实体必须额外满足的条件，默认是 `()`，表示不添加过滤。 |

`D` 决定“拿什么”，`F` 决定“哪些实体可以被拿到”。过滤器只参与匹配，不会出现在查询结果中。

## Query Data：查询结果

### Entity、只读组件和可变组件

| 写法 | 结果 | 用途 |
| --- | --- | --- |
| `Entity` | 实体 ID | 需要保存、比较或交给 Commands 时使用。 |
| `&T` | 只读组件引用 | 读取组件。多个只读 Query 可以并行。 |
| `&mut T` | 可变组件引用 | 修改组件；Query 参数需要声明为 `mut`。 |
| `()` | 不取数据 | 只关心哪些实体匹配过滤器。 |

```rust
fn read_positions(query: Query<(Entity, &Position)>) {
    for (entity, position) in &query {
        println!("{entity:?}: {:?}", position.0);
    }
}

fn move_positions(mut query: Query<&mut Position>) {
    for mut position in &mut query {
        position.0.x += 1.0;
    }
}
```

### 元组：一次读取多个组件

元组表示实体必须同时拥有其中的所有组件：

```rust
fn move_with_velocity(
    mut query: Query<(&mut Position, &Velocity)>,
) {
    for (mut position, velocity) in &mut query {
        position.0 += velocity.0;
    }
}
```

可变和只读访问可以混合，但同一个 Query 不能对同一组件同时产生冲突的 `&T` 和 `&mut T`。

### `Option<T>`：组件可能不存在

将组件写成 `Option<&T>` 或 `Option<&mut T>` 时，缺少该组件的实体仍然会匹配，只是结果为 `None`：

```rust
fn show_velocity(query: Query<(&Label, Option<&Velocity>)>) {
    for (label, velocity) in &query {
        println!("{}: {velocity:?}", label.0);
    }
}
```

这和把 `Velocity` 写进 `F` 不同：`Query<(&Label, With<Velocity>)>` 只会返回拥有 `Velocity` 的实体，而 `Option<&Velocity>` 会返回所有拥有 `Label` 的实体。

### `Has<T>`：只需要判断是否拥有组件

如果只需要一个布尔值，不需要读取组件数据，可以使用 `Has<T>`：

```rust
fn show_selected(query: Query<(&Label, Has<Selected>)>) {
    for (label, selected) in &query {
        println!("{} selected={selected}", label.0);
    }
}
```

`Has<T>` 不读取 `T` 的内容；它适合标记组件存在性判断。

## Query Filter：限制匹配范围

### `With<T>` 和 `Without<T>`

`With<T>` 要求实体拥有组件 `T`，`Without<T>` 要求实体没有组件 `T`：

```rust
fn move_players(mut query: Query<&mut Position, With<Player>>) {
    for mut position in &mut query {
        position.0.x += 1.0;
    }
}

fn find_unselected(query: Query<&Label, Without<Selected>>) {
    for label in &query {
        println!("未选中：{}", label.0);
    }
}
```

过滤器元组表示同时满足（逻辑 AND）：

```rust
Query<&mut Position, (With<Player>, Without<Dead>)>
```

### `Or<(...)>`

`Or` 表示满足其中任意一个过滤器（逻辑 OR）：

```rust
fn show_characters(query: Query<&Label, Or<(With<Player>, With<Enemy>)>>) {
    for label in &query {
        println!("角色：{}", label.0);
    }
}
```

`Or` 里面的每一项仍然可以是元组，例如 `Or<((With<Player>, Without<Dead>), With<Boss>)>`，但日常项目通常只需要一层简单组合。

### `Added<T>` 和 `Changed<T>`

这两个过滤器用于只处理最近发生变化的实体：

```rust
fn setup_new_entities(query: Query<&Label, Added<Velocity>>) {
    for label in &query {
        println!("最近添加了 Velocity：{}", label.0);
    }
}

fn update_changed_positions(query: Query<&Label, Changed<Position>>) {
    for label in &query {
        println!("Position 发生变化：{}", label.0);
    }
}
```

- `Added<T>`：组件 `T` 自上次运行该 System 后被添加；实体刚创建并带有 `T` 也会匹配。
- `Changed<T>`：组件 `T` 自上次运行该 System 后被修改或添加。对 `&mut T` 进行可变解引用也会被视为修改，不会比较修改前后的值。
- 两者第一次运行时，也可能匹配 System 启动前已经创建或修改过的组件；它们不是“只从程序启动后开始计时”。

## 遍历匹配实体

### 普通遍历

```rust
fn inspect(query: Query<&Position>) {
    for position in &query {
        println!("{:?}", position.0);
    }
}

fn update(mut query: Query<&mut Position>) {
    for mut position in &mut query {
        position.0.y += 1.0;
    }
}
```

`&query` 使用只读迭代，`&mut query` 使用可变迭代。也可以显式写成 `query.iter()` 和 `query.iter_mut()`。查询结果的顺序不保证，不要把它当作稳定排序。

常见的辅助方法还有：

| 方法 | 作用 |
| --- | --- |
| `iter().count()` | 统计匹配数量。 |
| `is_empty()` | 判断是否没有匹配实体。 |
| `get(entity)` / `get_mut(entity)` | 按一个 Entity ID 查询。 |
| `contains(entity)` | 判断一个 Entity 是否匹配该 Query。 |

### 按 Entity ID 查询

当系统已经保存了某个实体的 ID 时，使用 `get` 或 `get_mut`，而不是遍历所有实体：

```rust
fn inspect_entity(entity: Res<TrackedEntity>, query: Query<&Position>) {
    if let Ok(position) = query.get(entity.0) {
        println!("实体位置：{:?}", position.0);
    }
}
```

如果一次需要几个已知实体，可以使用 `get_many` 或 `get_many_mut`：

```rust
fn inspect_pair(ids: Res<TwoEntities>, query: Query<&Label>) {
    if let Ok([first, second]) = query.get_many([ids.first, ids.second]) {
        println!("{} 和 {}", first.0, second.0);
    }
}
```

可变版本要求这些 Entity ID 不重复，否则 Bevy 无法保证两个可变引用不指向同一个组件。

如果 Entity 列表是动态迭代器，也可以使用 `iter_many` 或 `iter_many_mut`；它们按给定的 Entity 列表读取匹配结果。对于固定数量的少量 ID，`get_many` 更直接。

### 单个实体和至少一个实体

普通 Query 的 `single`、`single_mut` 适合需要在运行时检查数量的情况：

```rust
fn singleton(query: Query<&Transform, With<Singleton>>) {
    if let Ok(transform) = query.single() {
        println!("唯一实体的位置：{:?}", transform.translation);
    }
}
```

`single()` 或 `single_mut()` 在没有匹配或匹配多个实体时返回错误。Bevy 还提供两个专用 System 参数：

```rust
fn player(mut player: Single<&mut Position, With<Player>>) {
    player.0.x += 1.0;
}

fn enemies(enemies: Populated<&Position, With<Enemy>>) {
    for position in &enemies {
        println!("Enemy：{:?}", position.0);
    }
}
```

- `Single<D, F>` 要求恰好一个匹配实体；数量不对时，整个 System 会被跳过。
- `Populated<D, F>` 要求至少一个匹配实体；没有匹配实体时，整个 System 会被跳过。
- 如果对象可能不存在，但存在时最多一个，可以使用 `Option<Single<D, F>>`。

### 组合遍历

`iter_combinations()` 会返回匹配实体的两两组合，适合简单的实体间检测：

```rust
fn check_pairs(query: Query<&Position>) {
    for [first, second] in query.iter_combinations() {
        // 比较 first 和 second，例如检测它们之间的关系。
    }
}
```

组合数量会随实体数快速增加，只适合实体数量较少或已经经过过滤的 Query。大规模组合检查通常需要专门的数据结构或算法优化。

需要修改组合中的组件时使用 `iter_combinations_mut()`，但要特别注意组合数量和可变引用的借用范围。

## 同一 System 中的多个 Query

如果一个 System 需要多个 Query，Bevy 会检查它们的读写是否冲突。访问不同组件的 Query 通常可以直接写在参数中：

```rust
fn update(mut positions: Query<&mut Position>, velocities: Query<&Velocity>) {
    // ...
}
```

如果多个 Query 之间存在无法同时满足的可变访问，先将它们拆分到不同 System 中，分别表达各自的职责。

## 常见注意事项

- `Query` 只会返回满足所有必需组件和过滤器的实体；组件可能不存在时使用 `Option`。
- 读取组件使用 `&T`，修改组件使用 `&mut T`，不要为了方便把所有组件都声明为可变。
- `Commands` 创建或删除的实体通常要等延迟命令应用后，后续 Schedule 才能通过 Query 看到变化。
- Query 遍历顺序不稳定；如果需要排序，应把结果收集后自行排序。
- `Added`、`Changed` 依赖 System 的运行记录；同一个组件在多个 System 中被修改时，应明确哪个 System 负责响应。

## 小结

日常开发可以按这个顺序写 Query：

```text
Query<D, F>
  ↓
D：Entity、&T、&mut T、元组、Option、Has
  ↓
F：With、Without、Or、Added、Changed
  ↓
遍历：iter / iter_mut
定位：get / get_mut / single
特殊数量：Single / Populated
```

掌握这些写法后，就足以覆盖大多数实体和组件的数据访问需求。
