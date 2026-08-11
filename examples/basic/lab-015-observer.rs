use bevy::prelude::*;

#[derive(Event)]
struct Greeting {
    message: String,
}

#[derive(Resource, Default)]
struct GreetingCount(u32);

fn observe_greeting(event: On<Greeting>, mut count: ResMut<GreetingCount>) {
    count.0 += 1;
    println!("Observer 收到第 {} 个事件：{}", count.0, event.message);
}

fn trigger_greetings(mut commands: Commands) {
    commands.trigger(Greeting {
        message: "第一次触发 Greeting".to_string(),
    });
    commands.trigger(Greeting {
        message: "第二次触发 Greeting".to_string(),
    });
}

fn exit_after_observing(count: Res<GreetingCount>, mut exit: MessageWriter<AppExit>) {
    if count.0 == 2 {
        println!("Observer 示例完成，退出应用");
        exit.write(AppExit::Success);
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_observer(observe_greeting)
        .init_resource::<GreetingCount>()
        .add_systems(Startup, trigger_greetings)
        .add_systems(Last, exit_after_observing)
        .run();
}
