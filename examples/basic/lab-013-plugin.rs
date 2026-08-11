use bevy::prelude::*;

#[derive(Resource, Default)]
struct PluginRuns(u32);

struct GreetingPlugin;

impl Plugin for GreetingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PluginRuns>()
            .add_systems(Update, greet_from_plugin);
    }
}

fn greet_from_plugin(mut runs: ResMut<PluginRuns>) {
    runs.0 += 1;
    println!("GreetingPlugin 注册的 System：第 {} 次运行", runs.0);
}

fn stop_after_three_updates(runs: Res<PluginRuns>, mut exit: MessageWriter<AppExit>) {
    if runs.0 >= 3 {
        println!("Plugin 示例完成，退出应用");
        exit.write(AppExit::Success);
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(GreetingPlugin)
        .add_systems(Last, stop_after_three_updates)
        .run();
}
