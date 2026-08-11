use bevy::prelude::*;

#[derive(Resource, Default)]
struct Frame(u32);

#[derive(Resource, Default)]
struct Gate(bool);

fn advance(mut frame: ResMut<Frame>, mut gate: ResMut<Gate>) {
    frame.0 += 1;
    gate.0 = frame.0 % 2 == 0;
    println!("Update：第 {} 帧，Gate={}", frame.0, gate.0);
}

fn gate_open(gate: Res<Gate>) -> bool {
    gate.0
}

fn allowed_system() {
    println!("run_if(gate_open)：条件满足，System 执行");
}

fn blocked_system() {
    println!("run_if(not(gate_open))：条件不满足 gate_open，但反向条件满足");
}

fn group_system_a() {
    println!("(A, B).run_if：A 执行");
}

fn group_system_b() {
    println!("(A, B).run_if：B 执行");
}

fn run_once_system() {
    println!("run_once：只执行一次");
}

fn stop_after_four_frames(frame: Res<Frame>, mut exit: MessageWriter<AppExit>) {
    if frame.0 >= 4 {
        println!("完成四帧，退出示例");
        exit.write(AppExit::Success);
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .init_resource::<Frame>()
        .init_resource::<Gate>()
        .add_systems(
            Update,
            (
                advance,
                allowed_system.run_if(gate_open),
                blocked_system.run_if(not(gate_open)),
                (group_system_a, group_system_b).run_if(gate_open),
                run_once_system.run_if(run_once),
            )
                .chain(),
        )
        .add_systems(Last, stop_after_four_frames)
        .run();
}
