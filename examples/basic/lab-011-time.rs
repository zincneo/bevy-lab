use bevy::{
    prelude::*,
    time::{Real, TimeUpdateStrategy, Virtual},
};
use std::time::Duration;

#[derive(Resource, Default)]
struct Counters {
    updates: u32,
    fixed_updates: u32,
}

#[derive(Resource, Default)]
struct Position(f32);

fn update_frame(
    time: Res<Time>,
    real_time: Res<Time<Real>>,
    virtual_time: Res<Time<Virtual>>,
    mut counters: ResMut<Counters>,
    mut position: ResMut<Position>,
) {
    counters.updates += 1;
    position.0 += 10.0 * time.delta_secs();

    println!(
        "Update：第 {} 次，Time delta={}，Real delta={}，Virtual delta={}，fixed_count={}，position={}",
        counters.updates,
        time.delta_secs(),
        real_time.delta_secs(),
        virtual_time.delta_secs(),
        counters.fixed_updates,
        position.0,
    );
}

fn fixed_update(time: Res<Time<Fixed>>, mut counters: ResMut<Counters>) {
    counters.fixed_updates += 1;
    println!(
        "FixedUpdate：第 {} 次，固定 delta={} 秒",
        counters.fixed_updates,
        time.delta_secs(),
    );
}

fn stop_after_ten_updates(counters: Res<Counters>, mut exit: MessageWriter<AppExit>) {
    if counters.updates >= 10 {
        println!(
            "完成十次 Update，共执行 {} 次 FixedUpdate，退出示例",
            counters.fixed_updates
        );
        exit.write(AppExit::Success);
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        // TimeUpdateStrategy 控制每次 App::update() 推进多少模拟时间，
        // 不控制 runner 调用 App::update() 的真实间隔。
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            250,
        )))
        // 10Hz 表示每个 FixedUpdate 步长为 100ms。
        // Time<Virtual> 默认最多接受单次 250ms 的虚拟时间增量。
        .insert_resource(Time::<Fixed>::from_hz(10.0))
        .init_resource::<Counters>()
        .init_resource::<Position>()
        .add_systems(Update, update_frame)
        .add_systems(FixedUpdate, fixed_update)
        .add_systems(Last, stop_after_ten_updates)
        .run();
}
