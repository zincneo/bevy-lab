# Lab 012：Timer

## 学习目标

`Timer` 是一个记录经过时间并判断是否到达目标时长的工具。它不会自己运行，必须在 System 中使用 `tick` 推进。最常见的做法是用 `Res<Time>` 提供时间增量，再用 `Timer` 判断一次性动作或重复动作何时发生。

## 创建 Timer

使用 `Timer::from_seconds` 创建一个按秒计时的 Timer，并指定计时模式：

```rust
let once = Timer::from_seconds(2.0, TimerMode::Once);
let repeating = Timer::from_seconds(1.0, TimerMode::Repeating);
```

两种常用模式的区别：

| 模式 | 完成后的行为 |
| --- | --- |
| `TimerMode::Once` | 到达时长后停止，保持完成状态，直到调用 `reset`。 |
| `TimerMode::Repeating` | 每次到达时长后重新从零开始，可持续触发。 |

Timer 只是一个普通 Rust 值，可以放在不同位置：

- 放在 `Resource` 中：整个 World 共用一个计时器；
- 放在实体 `Component` 中：每个实体拥有自己的计时器；
- 放在 `Local<Timer>` 中：只供某一个 System 保存内部计时状态。

例如，全局计时器可以定义为：

```rust
#[derive(Resource)]
struct Cooldown {
    timer: Timer,
}
```

## 使用 `tick` 推进计时

Timer 不会因为时间流逝自动变化。每次 System 运行时，都要把本次时间增量传给 `tick`：

```rust
fn tick_timer(time: Res<Time>, mut cooldown: ResMut<Cooldown>) {
    cooldown.timer.tick(time.delta());
}
```

`Time::delta()` 返回 `Duration`，正好可以传给 `Timer::tick`。如果 System 放在 `FixedUpdate` 中，则使用 `Res<Time<Fixed>>`：

```rust
fn tick_fixed_timer(time: Res<Time<Fixed>>, mut cooldown: ResMut<Cooldown>) {
    cooldown.timer.tick(time.delta());
}
```

Timer 应该和它所处的 Schedule 使用同一个时间来源：普通 `Update` 配合 `Time`，固定逻辑配合 `FixedUpdate` 和 `Time<Fixed>`。

时间策略、`Time<Virtual>` 的限制、`Time<Fixed>` 的步长以及一次主循环中执行多少次 `FixedUpdate`，统一见 Lab 011。本示例只关注 Timer 如何读取时间并推进自身状态。

## 判断 Timer 是否完成

### `just_finished`

`just_finished()` 只在本次 `tick` 到达目标时长的这一刻返回 `true`，适合触发一次动作：

```rust
fn play_effect(time: Res<Time>, mut timer: ResMut<Cooldown>) {
    timer.timer.tick(time.delta());

    if timer.timer.just_finished() {
        println!("冷却结束，执行一次动作");
    }
}
```

对于重复 Timer，每次周期结束时都会有一次 `just_finished()`；对于一次性 Timer，只会在第一次完成时返回 `true`。

### `is_finished`

`is_finished()` 表示 Timer 当前是否处于完成状态：

```rust
fn check_loading(time: Res<Time>, mut loading: ResMut<Cooldown>) {
    loading.timer.tick(time.delta());

    if loading.timer.is_finished() {
        println!("Timer 已完成");
    }
}
```

一次性 Timer 完成后会一直保持 `true`，直到 `reset()`；重复 Timer 的完成状态只对应当前周期，下一次 `tick` 会继续进入新的周期。因此，需要“刚刚完成时触发”时优先使用 `just_finished()`。

## 重置、暂停和查看进度

### 重置

```rust
timer.reset();
```

`reset()` 会清零已经过时间并清除完成状态。它不会改变 Timer 是一次性还是重复模式。

### 暂停和恢复

```rust
timer.pause();
timer.unpause();

if timer.is_paused() {
    println!("Timer 暂停中");
}
```

暂停后调用 `tick` 不会增加已经过时间。暂停 Timer 和暂停 `Time<Virtual>` 是两个不同层级的操作：前者只影响这个 Timer，后者会影响使用虚拟时间的应用逻辑。

### 查看进度

```rust
println!(
    "已完成比例={}，剩余={} 秒",
    timer.fraction(),
    timer.remaining_secs(),
);
```

常用查询方法如下：

| 方法 | 作用 |
| --- | --- |
| `elapsed_secs()` | 已经过的秒数。 |
| `remaining_secs()` | 距离完成剩余的秒数。 |
| `fraction()` | 已完成比例，范围通常是 `0.0..=1.0`。 |
| `fraction_remaining()` | 剩余比例。 |
| `duration()` | Timer 的目标时长。 |

这些值适合驱动进度条、淡入淡出或其他按时间插值的效果。

## Timer 与 `run_if`

如果只是希望某个简单 System 按固定间隔运行，可以使用 Bevy 提供的 `on_timer` 条件：

```rust
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

app.add_systems(
    Update,
    print_status.run_if(on_timer(Duration::from_secs(1))),
);
```

`on_timer` 内部管理了一个重复 Timer，适合低频状态刷新等简单场景。需要读取进度、暂停、重置，或者一次时间增量跨过多个周期时，应该自己保存 `Timer` 并调用 `tick`。

## 一个常见的冷却流程

```text
创建 TimerMode::Once 或 TimerMode::Repeating
        ↓
把 Timer 放进 Resource、Component 或 Local
        ↓
System 每次运行时调用 timer.tick(time.delta())
        ↓
just_finished() 判断本次是否刚完成
        ↓
一次性 Timer 可 reset 后再次使用
重复 Timer 自动进入下一周期
```

## 常见注意事项

1. Timer 不会自动推进；忘记调用 `tick` 时，`is_finished()` 和 `just_finished()` 都不会按预期变化。
2. 触发一次动作通常检查 `just_finished()`，不要只检查一次性 Timer 的 `is_finished()`，否则完成后每帧都可能重复执行。
3. Timer 的 `tick` 参数是 `Duration`，普通 Update 使用 `time.delta()`，不要直接传秒数浮点值。
4. 多个实体各自计时时，应把 Timer 放进 Component，而不是让所有实体共享一个 Resource。
5. 重复 Timer 在一次 `tick` 跨过多个周期时可能完成多次；需要统计次数时再使用 `times_finished_this_tick()`。
