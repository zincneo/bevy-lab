use bevy::prelude::*;

#[derive(Resource, Default)]
struct LoopProgress {
    main_loops: u32,
    fixed_steps: u32,
}

fn startup() {
    println!("Startup    | 创建初始实体和资源");
}

fn pre_update(mut progress: ResMut<LoopProgress>) {
    progress.main_loops += 1;
    println!("PreUpdate  | 准备第 {} 次主循环的输入", progress.main_loops);
}

fn fixed_update(mut progress: ResMut<LoopProgress>) {
    progress.fixed_steps += 1;
    println!(
        "FixedUpdate| 执行第 {} 次固定模拟（主循环 {}）",
        progress.fixed_steps, progress.main_loops
    );
}

fn update(progress: Res<LoopProgress>) {
    println!("Update     | 执行第 {} 次主循环逻辑", progress.main_loops);
}

fn post_update(progress: Res<LoopProgress>) {
    println!("PostUpdate | 同步第 {} 次主循环结果", progress.main_loops);
}

fn last(progress: Res<LoopProgress>, mut exit: MessageWriter<AppExit>) {
    println!("Last       | 第 {} 次主循环结束", progress.main_loops);

    // 在 Last 退出，确保第三次循环已经完整经过 Update 和 PostUpdate。
    if progress.main_loops == 3 {
        println!("           | 已完成 3 次完整主循环，准备退出");
        exit.write(AppExit::Success);
    }
}

fn main() {
    App::new()
        // MinimalPlugins 提供无窗口运行所需的时间、Schedule 和 runner。
        .add_plugins(MinimalPlugins)
        .insert_resource(LoopProgress::default())
        .add_systems(Startup, startup)
        .add_systems(PreUpdate, pre_update)
        // 这个示例很快退出，FixedUpdate 可能尚未累计到一个固定步长。
        .add_systems(FixedUpdate, fixed_update)
        .add_systems(Update, update)
        .add_systems(PostUpdate, post_update)
        .add_systems(Last, last)
        .run();
}
