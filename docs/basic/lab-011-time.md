# Lab 011：Time

## 学习目标

理解 Bevy 中几种时间 Resource 的关系，知道普通 `Update` 和固定 `FixedUpdate` 分别读取什么时间，并使用 `TimeUpdateStrategy` 与 `Time<Fixed>` 控制测试中的模拟时间和固定步长。

## Time 资源从哪里来

`Time` 由 `TimePlugin` 管理。`DefaultPlugins` 和 `MinimalPlugins` 都包含 `TimePlugin`，会准备多个时间 Resource：

| Resource | 作用 |
| --- | --- |
| `Time<Real>` | 记录真实时钟的增量，不受暂停和时间缩放影响。 |
| `Time<Virtual>` | 在真实时间基础上应用暂停、相对速度和单次最大增量限制。 |
| `Time` | 默认的无标签时间，跟随 `Time<Virtual>`，普通 `Update` 通常读取它。 |
| `Time<Fixed>` | 为 `FixedUpdate` 保存固定步长和累计的固定时间。 |

可以把它们的依赖关系理解为：

```text
TimeUpdateStrategy
        ↓
Time<Real>：本次 App 更新应该增加多少原始时间
        ↓
Time<Virtual>：应用暂停、倍速和 max_delta 限制
        ↓
Time：普通 Update 使用的默认时间
        ↓
Time<Fixed>：按固定 timestep 消耗累计时间，驱动 FixedUpdate
```

普通 System 通常只需要：

```rust
fn update_game(time: Res<Time>) {
    println!("本次更新：{} 秒", time.delta_secs());
}
```

如果没有添加包含 `TimePlugin` 的插件，`Res<Time>` 就没有对应的 Resource。正常应用不需要自己初始化这些 Resource。

## `delta` 到底相对什么时间

`delta` 不是“同一个 System 上一次执行到这一次执行”的时间，而是对应时钟两次更新之间的时间增量。`TimePlugin` 在应用流程中先更新时钟，同一轮之后运行的 System 会读取同一个时钟值。

```text
TimePlugin 更新时钟
        ↓
记录该时钟本次与上次更新之间的间隔
        ↓
同一轮中的 System 读取相同的 delta
```

普通应用中，`Time` 通常在每个主循环更新一次，因此 `Res<Time>::delta()` 通常表示相邻两帧之间的虚拟时间。一个使用 `run_if` 被跳过的 System 不会让 Bevy 自动累加它被跳过的帧；如果需要统计该 System 自己两次执行之间的时间，应自行保存累计值。

第一次更新用于初始化真实时钟，可能得到 `0` 的 `delta`。这不表示 System 没有执行。

最常用的时间值如下：

| 方法 | 含义 | 常见用途 |
| --- | --- | --- |
| `delta()` | `Duration` 类型的本次时钟增量。 | 传给 `Timer::tick`。 |
| `delta_secs()` | 以 `f32` 秒表示的本次增量。 | 移动、旋转和插值。 |
| `delta_secs_f64()` | 以 `f64` 秒表示的本次增量。 | 需要更高精度的计算。 |
| `elapsed()` | 从该时钟开始后的累计 `Duration`。 | 传给其他时间 API。 |
| `elapsed_secs()` | 从该时钟开始后的累计秒数。 | 计算周期或动画相位。 |

按速度移动时应使用“速度 × 时间增量”：

```rust
fn move_object(mut position: ResMut<Position>, time: Res<Time>) {
    let speed_per_second = 100.0;
    position.x += speed_per_second * time.delta_secs();
}
```

这样移动距离取决于经过的时间，而不是 System 在一秒内被调用了多少次。

## `TimeUpdateStrategy`：每次 App 更新推进多少时间

`TimeUpdateStrategy` 影响 `TimePlugin` 在一次 `App::update()` 中如何更新 `Time<Real>`，进而影响虚拟时间和固定计划表。它不控制 runner 何时再次调用 `App::update()`，也不会让线程主动等待。

| 策略 | 作用 |
| --- | --- |
| `Automatic` | 使用两次时钟更新之间真实经过的时间，适合普通应用。 |
| `ManualDuration(duration)` | 每次更新人为增加固定时长，适合测试和可重复示例。 |
| `ManualInstant(instant)` | 使用调用方提供的时间点；要让时间继续前进，就要持续提供新的时间点。 |
| `FixedTimesteps(n)` | 按固定时间步推进，并让一次 `App::update()` 处理指定数量的固定步，适合测试固定流程。 |

主循环的频率由 runner 或窗口事件循环决定：

- `App::new()` 默认只执行一次更新。
- `MinimalPlugins` 提供 `ScheduleRunnerPlugin`，默认持续循环。
- `ScheduleRunnerPlugin::run_loop(duration)` 才是设置 runner 等待时间的方式。
- 窗口应用通常由窗口事件循环驱动。

例如 runner 每约 `16ms` 调用一次 `App::update()`，设置 `ManualDuration(100ms)` 后，真实循环间隔仍约为 `16ms`，只是模拟时间每轮前进 `100ms`。

## `Time<Virtual>`：普通时间的中间层

默认的 `Time` 跟随 `Time<Virtual>`。虚拟时间可以控制游戏逻辑是否暂停、运行速度以及单次允许推进的最大时间：

```rust
fn pause(mut virtual_time: ResMut<Time<Virtual>>) {
    virtual_time.pause();
}

fn resume(mut virtual_time: ResMut<Time<Virtual>>) {
    virtual_time.unpause();
}

fn slow_motion(mut virtual_time: ResMut<Time<Virtual>>) {
    virtual_time.set_relative_speed(0.5);
}
```

`Time<Virtual>` 的 `max_delta` 默认是 `250ms`。如果一次原始时间增量大于这个值，虚拟时间会截断多出的部分，以避免程序卡顿或系统挂起后突然模拟过长时间；普通 `Time` 和 `FixedUpdate` 也会使用截断后的虚拟增量。

测试较大的手动时间增量时，可以显式提高限制：

```rust
use bevy::time::Virtual;
use std::time::Duration;

app.insert_resource(Time::<Virtual>::from_max_delta(Duration::from_secs(1)));
```

## `Time<Fixed>` 与 `FixedUpdate`

`Time<Fixed>` 保存固定步长 `timestep`。`FixedUpdate` 不是一个独立线程，也不是按照墙上时钟自动唤醒的计划表；它位于每次 `App::update()` 的固定流程中，检查 `Time<Virtual>` 累计的时间可以消耗多少个固定步：

```text
TimePlugin 更新 Time<Virtual>
        ↓
RunFixedMainLoop 累积虚拟时间
        ↓
每累计够一个 timestep，执行一次 FixedUpdate
        ↓
剩余不足一个 timestep 的时间保留到下一轮
```

因此一次主循环可能执行零次、一次或多次 `FixedUpdate`。在 `FixedUpdate` 中读取的 `Time<Fixed>::delta()` 始终等于固定步长：

```rust
fn fixed_logic(time: Res<Time<Fixed>>) {
    println!("固定步长：{} 秒", time.delta_secs());
}

app.add_systems(FixedUpdate, fixed_logic)
    .insert_resource(Time::<Fixed>::from_hz(60.0));
```

`from_hz(60.0)` 表示一个固定步约为 `16.67ms`；`from_hz(10.0)` 表示一个固定步为 `100ms`。频率越高，单步越短，同样的虚拟时间内通常会执行更多次 `FixedUpdate`。

## 示例中的次数换算

011 示例使用：

```rust
TimeUpdateStrategy::ManualDuration(Duration::from_millis(250))
Time::<Fixed>::from_hz(10.0)
```

这表示每次时间更新最多给虚拟时钟增加 `250ms`，每个固定步长为 `100ms`。第一次更新只初始化时钟，`delta` 为 `0`；示例在第十次 `Update` 的 `Last` 阶段退出，所以真正提供非零时间的是后面的 9 次更新：

```text
9 × 250ms = 2250ms
2250ms ÷ 100ms = 22 次完整 FixedUpdate
剩余 50ms 留在固定时间累计值中
```

如果把 `ManualDuration` 改成 `500ms`，默认 `Time<Virtual>::max_delta` 会把每次虚拟增量截断为 `250ms`，因此仍然是 22 次。提高 `max_delta` 后，才会按 `500ms` 计算，此时理论上是 `9 × 500ms ÷ 100ms = 45` 次。

## 常见注意事项

1. 普通 `Update` 逻辑通常读取 `Res<Time>`；固定逻辑读取 `Res<Time<Fixed>>`。
2. 移动和旋转通常使用“速度 × `time.delta_secs()`”，不要每次 System 执行都增加固定距离。
3. `delta_secs()` 的单位是秒，不是毫秒。
4. `TimeUpdateStrategy` 控制模拟时间如何推进，不控制主循环的真实调用间隔。
5. `Time<Virtual>` 的暂停、倍速和 `max_delta` 会影响普通 `Time` 以及 `FixedUpdate`。
