# Lab 014：Bundle

## 学习目标

了解 Bundle 如何把多个 Component 组合成一个方便创建实体的类型，以及创建后如何继续使用普通 Query 访问这些 Component。

## Bundle 是什么

Bundle 是一组 Component 的组合描述。使用 `#[derive(Bundle)]` 可以定义一个 Bundle：

```rust
#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    health: Health,
    speed: Speed,
}
```

Bundle 主要解决“某类实体通常需要同时添加哪些 Component”的重复问题。它不是一种新的运行时数据存储；实体创建后，里面的字段仍然是普通 Component。

## 使用 Bundle 创建实体

Bundle 最常见的用法是传给 `Commands::spawn`：

```rust
fn spawn_player(mut commands: Commands) {
    commands.spawn(PlayerBundle {
        player: Player,
        health: Health(100),
        speed: Speed(4.0),
    });
}
```

这相当于一次向实体添加 `Player`、`Health` 和 `Speed` 三个 Component。

## 创建后如何查询

Query 查询的是实体上的 Component，而不是 Bundle 类型：

```rust
fn inspect_player(query: Query<(Entity, &Health, &Speed), With<Player>>) {
    for (entity, health, speed) in query.iter() {
        println!("{:?}: {} {}", entity, health.0, speed.0);
    }
}
```

因此 Bundle 主要用于创建时的组合，Query、Commands 和 Component 的后续操作方式不变。

## 什么时候使用 Bundle

- 多个实体都需要相同的一组 Component 时；
- 创建实体的字段较多，不希望在每次 `spawn` 中重复书写时；
- 希望用一个有意义的类型名表达实体的初始组成时。

Bundle 不负责实体行为，也不会替代 Component。需要查询、修改或过滤实体时，仍然使用各个具体 Component。
