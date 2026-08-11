use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use std::time::Duration;

#[derive(Resource)]
struct Timers {
    frame: u32,
    once: Timer,
    repeating: Timer,
}

impl Default for Timers {
    fn default() -> Self {
        Self {
            frame: 0,
            once: Timer::from_seconds(0.5, TimerMode::Once),
            repeating: Timer::from_seconds(0.75, TimerMode::Repeating),
        }
    }
}

fn tick_timers(time: Res<Time>, mut timers: ResMut<Timers>) {
    timers.frame += 1;
    timers.once.tick(time.delta());
    timers.repeating.tick(time.delta());

    println!(
        "第 {} 帧：once={:.2}, repeating={:.2}",
        timers.frame,
        timers.once.fraction(),
        timers.repeating.fraction(),
    );

    if timers.once.just_finished() {
        println!("TimerMode::Once：只触发一次");
    }

    if timers.repeating.just_finished() {
        println!("TimerMode::Repeating：周期结束，触发一次");
    }

    if timers.once.is_finished() {
        println!("Once Timer 当前保持完成状态");
    }
}

fn stop_after_ten_frames(timers: Res<Timers>, mut exit: MessageWriter<AppExit>) {
    if timers.frame >= 10 {
        println!("完成十帧计时，退出示例");
        exit.write(AppExit::Success);
    }
}

fn main() {
    App::new()
        // 使用真实时间推进 Timer；runner 每 100ms 调用一次 App::update()。
        // 时间策略、虚拟时间和 FixedUpdate 的配置集中在 Lab 011 介绍。
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(100))))
        .init_resource::<Timers>()
        .add_systems(Update, tick_timers)
        .add_systems(Last, stop_after_ten_frames)
        .run();
}
