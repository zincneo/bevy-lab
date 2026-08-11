use bevy::prelude::*;

#[derive(Resource, Default)]
struct UpdateCount(u32);

fn first_system(mut local_count: Local<u32>) {
    *local_count += 1;
    println!("first_system 的 Local 状态：{}", *local_count);
}

fn second_system(mut local_count: Local<u32>) {
    *local_count += 1;
    println!("second_system 的 Local 状态：{}", *local_count);
}

fn count_updates(mut updates: ResMut<UpdateCount>) {
    updates.0 += 1;
    println!("共享 Resource 的 Update 次数：{}", updates.0);
}

fn stop_after_three_updates(updates: Res<UpdateCount>, mut exit: MessageWriter<AppExit>) {
    if updates.0 >= 3 {
        println!("Local 示例完成，退出应用");
        exit.write(AppExit::Success);
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .init_resource::<UpdateCount>()
        .add_systems(Update, (first_system, second_system, count_updates).chain())
        .add_systems(Last, stop_after_three_updates)
        .run();
}
