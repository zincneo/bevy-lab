# Lab 016：Local System State

## 学习目标

了解 `Local<T>` 如何保存某一个 System 自己的状态，并区分它与共享的 Resource、实体上的 Component。

## `Local<T>` 是什么

`Local<T>` 是一种 System 参数。它为当前 System 保存一个独立的 `T` 值，并在该 System 的多次运行之间保留这个值：

```rust
fn count_runs(mut count: Local<u32>) {
    *count += 1;
    println!("System 已运行 {} 次", *count);
}
```

第一次运行时，`Local<u32>` 使用默认值初始化；之后每次运行都会继续使用上一次保存的值。

## 每个 System 都有自己的 Local

两个不同的 System 即使使用相同的 `Local<u32>` 类型，也不会共享同一个计数值：

```rust
fn first_system(mut count: Local<u32>) {
    *count += 1;
    println!("first: {}", *count);
}

fn second_system(mut count: Local<u32>) {
    *count += 1;
    println!("second: {}", *count);
}
```

它们各自维护自己的状态，都会输出 `1、2、3...`。

## 与 Resource、Component 的区别

| 存储方式 | 数据属于谁 | 常见用途 |
| --- | --- | --- |
| `Local<T>` | 一个 System | 该 System 的计数器、缓存和上次执行状态。 |
| `Resource` | 整个 World | 多个 System 需要共享的数据。 |
| `Component` | 某个 Entity | 每个实体各自拥有的数据。 |

选择方式可以简单判断为：

- 只有一个 System 需要，并且是它自己的内部状态：使用 `Local<T>`；
- 多个 System 需要读写：使用 `Resource`；
- 数据属于某个实体：使用 `Component`。

`Local<T>` 不需要手动插入 World，也不能直接被其他 System 查询。需要跨 System 共享时，应改用 Resource。
