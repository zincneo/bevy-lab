# Lab 002：Commands

## `Commands` 是什么

`Commands` 是 Bevy ECS 提供给系统使用的一个 **System Parameter（系统参数）**。它不是 `World` 本身，也不是一个实体容器，而是一个把“对 `World` 进行修改的请求”记录下来的接口。

系统可以通过 `Commands` 请求：

- 创建或删除实体
- 给实体添加、替换或移除组件
- 插入、初始化或移除 Resource
- 将自定义命令放入命令队列

这些请求通常不会在调用 `commands.spawn(...)` 或 `commands.entity(...).despawn()` 的那一行立刻直接修改 `World`。Bevy 会先把请求放入命令队列，在合适的调度边界应用它们。这种机制也叫 **deferred commands（延迟命令）**。

可以把它理解成：

```text
系统执行
    commands.spawn(ComponentA)
        ↓
记录一个“创建实体并添加 ComponentA”的请求
        ↓
调度器在安全时机应用命令
        ↓
World 真正发生结构变化
```

## 为什么使用延迟命令

实体创建、销毁和组件增删属于 ECS 的**结构性变化**。这些操作可能改变实体所在的 archetype、组件存储和查询索引。如果一个系统正在遍历组件数据时，另一个系统直接修改这些结构，就需要复杂的锁和借用协调，甚至可能使正在进行的遍历失效。

`Commands` 把结构性变化延迟到安全的时机，带来几个好处：

1. **保护正在进行的查询**：系统可以先完成当前对 `World` 数据的读取或修改，再应用实体和组件结构变化。
2. **更容易并行调度**：系统提交命令时不必直接取得整个 `World` 的独占结构修改权，调度器有更多机会安排不冲突的系统同时运行。
3. **统一修改入口**：创建、销毁、添加组件和移除组件都可以通过相同的命令接口表达。
4. **隔离系统逻辑与存储细节**：系统只描述“想让 ECS 发生什么变化”，不需要直接操作 archetype 或组件数组。

延迟也意味着一个重要事实：刚刚通过 `Commands` 创建的实体或组件，不一定能在同一个调度阶段的普通查询中立即看到。需要等命令被应用后，后续系统或后续阶段才能查询到最新结构。

## 为什么 `Commands` 可以作为系统参数

Bevy 的系统函数不是由 Bevy 特别硬编码某一个固定签名，而是会分析函数的每一个参数。只要参数类型实现了 Bevy 的 `SystemParam`，它就可以被 Bevy 在运行系统时自动构造和注入。

因此，下面的 `Commands` 之所以能使用，是因为 Bevy 知道如何从当前 `World` 和命令队列中构造它：

```rust
fn spawn_system(mut commands: Commands) {
    commands.spawn_empty();
}
```

`Commands` 可以和其他系统参数一起使用：

```rust
fn gameplay_system(
    time: Res<Time>,
    query: Query<&mut Position>,
    mut commands: Commands,
) {
    // 使用 time 读取资源
    // 使用 query 访问组件
    // 使用 commands 提交结构性修改
}
```

## 常用功能

### 创建实体

`spawn` 创建实体并添加一个组件、一个 tuple bundle 或一个自定义 bundle：

```rust
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Health(u32);

fn spawn_player(mut commands: Commands) {
    let player = commands.spawn((Player, Health(100))).id();
    // player 是预留的 Entity ID，可保存到 Resource 或其他组件中。
}
```

常见相关方法：

- `spawn_empty()`：创建没有组件的实体；
- `spawn(bundle)`：创建实体并一次添加组件；
- `spawn_batch(iterable)`：批量创建拥有相同组件组合的实体，适合大量实体初始化；
- `id()`：从返回的 `EntityCommands` 取得实体 ID。

### 修改指定实体

已知 `Entity` ID 后，通过 `commands.entity(entity)` 取得这个实体的命令对象：

```rust
#[derive(Component)]
struct Visible;

fn mark_visible(mut commands: Commands, player: Res<PlayerEntity>) {
    commands.entity(player.entity).insert(Visible);
}
```

常用的实体命令包括：

- `insert(component_or_bundle)`：添加或替换组件；
- `insert_if_new(component_or_bundle)`：只在组件不存在时添加；
- `remove::<Component>()`：移除组件；
- `despawn()`：销毁实体及其组件；
- `get_entity(entity)`：以可失败的方式取得实体命令对象，适合实体可能已经不存在的情况。

上例中的 `PlayerEntity` 只是为了表达“保存了一个 Entity ID 的 Resource”，完整定义可以是：

```rust
#[derive(Resource)]
struct PlayerEntity {
    entity: Entity,
}
```

### 管理 Resource

`Commands` 也可以延迟操作全局 Resource：

```rust
#[derive(Resource)]
struct Score(u32);

fn setup_score(mut commands: Commands) {
    commands.insert_resource(Score(0));
}

fn remove_score(mut commands: Commands) {
    commands.remove_resource::<Score>();
}
```

常见方法包括：

- `insert_resource(value)`：插入或替换 Resource；
- `init_resource::<T>()`：在没有该 Resource 时使用 `Default` 或 `FromWorld` 初始化；
- `remove_resource::<T>()`：移除 Resource；
- `insert_resource_if_neq(value)`：只有新旧 Resource 不同才写入。

### 添加自定义命令

除了内置方法，还可以通过 `commands.queue(...)` 放入自己的命令。自定义命令最终会在应用到 `World` 时执行：

```rust
struct IncreaseScore(u32);

impl Command for IncreaseScore {
    type Out = ();

    fn apply(self, world: &mut World) {
        if let Some(mut score) = world.get_resource_mut::<Score>() {
            score.0 += self.0;
        }
    }
}

fn add_score(mut commands: Commands) {
    commands.queue(IncreaseScore(10));
}
```

自定义命令适合把需要独占访问 `World` 的一小段操作封装起来，但初学阶段优先使用 `spawn`、`entity`、`insert_resource` 等内置方法即可。

## 常见误区

### 把 `Commands` 当成即时的 `World` 引用

`Commands` 记录的是未来要应用的操作，不是一个可以立即读取所有最新 ECS 数据的 `World` 引用。需要读取组件，应使用查询参数；需要直接、立即访问 `World`，则是另一类 exclusive system 的用法。

### 以为 `spawn` 后同一系统的查询一定能找到实体

```rust
fn confusing_system(mut commands: Commands, query: Query<Entity>) {
    commands.spawn(Player);

    // 这里的 query 不一定包含刚刚 spawn 的实体，
    // 因为 spawn 只是把命令加入队列，命令可能尚未应用。
}
```

如果后续逻辑必须看到新实体，应将逻辑放到命令应用之后的系统或调度阶段，或者采用适合该场景的即时 `World` 操作方式。

## 小结

`Commands` 可以概括为“由系统函数提交、稍后作用于 `World` 的 ECS 修改请求”。它是 Bevy 的系统参数，所以可以和 `Query`、`Res` 等参数共同出现在系统函数中；它没有必须位于第一个参数的规则。将它放在第一位主要是团队和教程常用的可读性约定。
