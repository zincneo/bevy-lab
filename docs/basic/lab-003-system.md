# Lab 003：System 函数

## System 函数是什么

System 是一段可以被 App 调度执行的逻辑，最常见的写法就是普通 Rust 函数：

```rust
fn print_message() {
    println!("System 正在运行");
}

app.add_systems(Update, print_message);
```

能够注册为 System 的函数通常满足：

- 可以有 0 个、1 个或多个参数；
- 每个参数都实现 Bevy 的 `SystemParam`；
- 普通 System 通常返回 `()`。

使用时只需要在函数签名中声明需要的数据，不必关心 Bevy 内部如何生成这些参数。

## `add_systems` 的基本用法

第一个参数是 Schedule，第二个参数是 System：

```rust
app.add_systems(Startup, setup)
    .add_systems(Update, update);
```

也可以一次注册多个 System：

```rust
app.add_systems(Update, (read_input, update_player));
```

默认不要依赖函数的书写顺序；确实需要先后关系时，再使用 `.chain()`、`.before()` 或 `.after()`：

```rust
app.add_systems(Update, (read_input, update_player).chain());
```

Schedule 的选择方式见 Lab 004。

## 最常用的 System 参数

| 参数 | 用途 | 示例 |
| --- | --- | --- |
| `Res<T>` | 只读访问全局 Resource。 | `Res<Score>` |
| `ResMut<T>` | 修改全局 Resource。 | `ResMut<Score>` |
| `Query<D, F>` | 按组件查询实体，并读取或修改组件。 | `Query<(&mut Position, &Velocity)>` |
| `Commands` | 延迟创建实体、添加或删除组件。 | `Commands` |
| `Local<T>` | 保存当前 System 自己的状态。 | `Local<u32>` |

## 小结

System 就是被 Schedule 调度的函数。编写普通游戏逻辑时，优先掌握 `Res`、`ResMut`、`Query`、`Commands` 和 `Local`，然后把 System 注册到合适的 Schedule 即可。
