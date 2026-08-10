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
| `Update` | 每帧一次的游戏逻辑、输入响应、动画和 UI。 |
| `PostUpdate` | 根据本帧结果做同步、相机或 Transform 后处理。 |
| `Last` | 清理、统计和调试输出。 |

`RunFixedMainLoop` 主要负责驱动固定时间步，普通游戏逻辑通常不直接注册到这里。

### `FixedUpdate`

固定流程由 `RunFixedMainLoop` 驱动，最常用的注册点是 `FixedUpdate`：

```rust
app.add_systems(FixedUpdate, move_player);
```

它适合物理、碰撞、确定性移动等需要稳定时间步的逻辑。固定流程一帧内可能执行 0 次、1 次或多次，因此不要把它当作“每个渲染帧只执行一次”。

### 哪些资源决定固定计划表的执行间隔

固定计划表没有一个单独的“每隔多少秒执行”开关。Bevy 会把经过的虚拟时间累积起来，达到固定步长后执行 `FixedUpdate`。最直接影响这个过程的是两个资源类型：`Time<Fixed>` 和 `TimeUpdateStrategy`。

#### `Time<Fixed>`：一次固定步长有多长

`Time<Fixed>` 保存固定时间步的长度，常用的构造方式有两种：

下面比较 `hz` 或 `duration` 对执行间隔的影响时，假设 `TimeUpdateStrategy` 保持不变。

| 写法 | 含义 | 对执行间隔的影响 |
| --- | --- | --- |
| `Time::<Fixed>::from_hz(hz)` | 用频率设置步长，步长为 `1 / hz` 秒。 | `hz` 越大，步长越短，固定计划表执行得越频繁；`hz` 越小则越稀疏。 |
| `Time::<Fixed>::from_duration(duration)` | 直接用 `Duration` 设置步长。 | `duration` 越短，执行间隔越短；`duration` 越长，执行间隔越长。 |

例如：

```rust
Time::<Fixed>::from_hz(60.0); // 每个固定步约 16.67ms
Time::<Fixed>::from_duration(Duration::from_millis(20)); // 每个固定步 20ms
```

#### `TimeUpdateStrategy`：每次 App 更新推进多少时间

`TimeUpdateStrategy` 决定每次主循环更新时，Bevy 如何推进时间。不同类别的值会改变固定计划表在主循环中的触发频率：

| 值 | 时间推进方式 | 固定计划表的表现 |
| --- | --- | --- |
| `Automatic` | 使用真实经过的时间。 | 一帧经过时间小于固定步长时可能执行 0 次；经过时间较长时可能连续执行多次以追赶时间。 |
| `ManualDuration(duration)` | 每次更新都人为增加指定时长。 | `duration` 小于固定步长时，需要多次 App 更新才执行一次；等于固定步长时通常每次执行一次；大于固定步长时可能一次执行多次。 |
| `ManualInstant(instant)` | 使用外部提供的时间点，前后时间点的差值作为本次推进量。 | 执行间隔由传入的时间点差值决定，适合测试或外部时钟驱动。 |
| `FixedTimesteps(n)` | 每次更新按 `n × Time<Fixed>` 推进。 | 时间初始化完成后，通常每次 App 更新执行 `n` 次固定流程；`n` 越大，同一次主循环中的固定更新越多。 |

如果 `ManualDuration` 使用 `Duration::ZERO`，或者 `FixedTimesteps(0)`，虚拟时间不会向前推进，固定流程也不会被触发。

examples对应的示例代码使用：

```rust
.insert_resource(Time::<Fixed>::from_hz(60.0))
.insert_resource(TimeUpdateStrategy::FixedTimesteps(1))
```

也就是固定步长约 `16.67ms`，每次 App 更新推进一个固定步。这里的时间是模拟时间，不是让 `MinimalPlugins` 的 runner 睡眠 `16.67ms`；首次更新时间增量可能为 0，因此第一次主循环没有 `FixedUpdate` 是正常的。

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

对于平台类或俯视角 2D 游戏，常见分工是：

```text
初始化实体和资源 → Startup
整理输入           → PreUpdate
物理和碰撞         → FixedUpdate
游戏逻辑和表现     → Update
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

学习 2D 游戏开发时，先掌握以下选择即可：

```text
Startup     一次性初始化
PreUpdate   准备输入
FixedUpdate 固定时间步模拟
Update      每帧游戏逻辑和表现
PostUpdate  结果同步
Last        收尾和统计
```

本 lab 的示例使用 `MinimalPlugins`，通过 Resource 让 App 完整运行三次，并打印这些常用阶段的执行位置。
