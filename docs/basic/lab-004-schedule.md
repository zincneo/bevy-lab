# Lab 004：Schedule（计划表）

## Schedule 是什么

Schedule 是 Bevy 安排 System 执行时机的计划表：

```rust
app.add_systems(Update, update_player);
```

这行代码只是在配置阶段把 `update_player` 登记到 `Update`。App 运行时，runner 驱动 `Update`，Bevy 才会调用这个 System。

## 常用 Schedule

### `Startup`

只在 App 第一次更新时运行，适合创建初始实体、插入资源和读取配置：

```rust
app.add_systems(Startup, setup);
```

Bevy 还提供 `PreStartup` 和 `PostStartup`，但普通项目通常只需要 `Startup`。

### 主循环 Schedule

每次主循环大致经过：

```text
PreUpdate → RunFixedMainLoop → Update → PostUpdate → Last
```

| Schedule | 常见用途 |
| --- | --- |
| `PreUpdate` | 准备输入和外部数据。 |
| `Update` | 每帧一次的逻辑、输入响应、动画和界面更新。 |
| `PostUpdate` | 根据本帧结果做同步、相机或 Transform 后处理。 |
| `Last` | 清理、统计和调试输出。 |

`RunFixedMainLoop` 主要负责驱动固定时间步，普通逻辑通常不直接注册到这里。

### `FixedUpdate`

固定流程由 `RunFixedMainLoop` 驱动，最常用的注册点是 `FixedUpdate`：

```rust
app.add_systems(FixedUpdate, move_player);
```

它适合物理、碰撞、确定性移动等需要稳定时间步的逻辑。固定流程一帧内可能执行 0 次、1 次或多次，因此不要把它当作“每个渲染帧只执行一次”。

固定步长的长度、虚拟时间如何推进，以及一次主循环中为什么可能执行多次 `FixedUpdate`，统一见 Lab 011。本节只关注 Schedule 的职责和执行位置。

## 一次主循环的关系

可以先记住下面这个简化流程：

```text
第一次更新：Startup

每次更新：
PreUpdate
  ↓
固定流程（可能执行 0 次或多次）
  ↓
Update → PostUpdate → Last
```

一个常见的职责划分是：

```text
初始化实体和资源 → Startup
整理输入           → PreUpdate
物理和碰撞         → FixedUpdate
每帧逻辑和表现     → Update
同步相机或变换     → PostUpdate
调试统计和清理     → Last
```

## 同一 Schedule 内的顺序

同一 Schedule 中的 System 不应依赖源码书写顺序。需要明确顺序时使用：

```rust
app.add_systems(Update, (read_input, simulate, update_camera).chain());
```

也可以使用 `.before()`、`.after()`；只是把函数先写在前面，并不能保证它先运行。

## 小结

选择 Schedule 时，先掌握以下对应关系即可：

```text
Startup     一次性初始化
PreUpdate   准备输入
FixedUpdate 固定时间步模拟
Update      每帧逻辑和表现
PostUpdate  结果同步
Last        收尾和统计
```

本 lab 的示例使用 `MinimalPlugins`，通过 Resource 让 App 完整运行三次，并打印这些常用阶段的执行位置。
示例退出得很快，`FixedUpdate` 可能还没有累计到一个固定步长而不输出；固定步长和累计时间的配置见 Lab 011。
