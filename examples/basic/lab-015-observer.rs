use bevy::prelude::*;

#[derive(Event)]
struct Greeting {
    message: String,
}

#[derive(EntityEvent)]
struct EntityGreeting {
    entity: Entity,
    message: String,
}

#[derive(Resource, Default)]
struct GreetingCount(u32);

#[derive(Resource, Default)]
struct EntityGreetingCount(u32);

fn observe_greeting(event: On<Greeting>, mut count: ResMut<GreetingCount>) {
    count.0 += 1;
    println!("Observer 收到第 {} 个事件：{}", count.0, event.message);
}

fn observe_entity_greeting(event: On<EntityGreeting>, mut count: ResMut<EntityGreetingCount>) {
    count.0 += 1;
    println!(
        "Entity {} 的 Observer 收到第 {} 个事件：{}",
        event.entity, count.0, event.message
    );
}

fn trigger_greetings(mut commands: Commands) {
    commands.trigger(Greeting {
        message: "第一次触发 Greeting".to_string(),
    });
    commands.trigger(Greeting {
        message: "第二次触发 Greeting".to_string(),
    });

    let listener = commands.spawn_empty().observe(observe_entity_greeting).id();

    commands.trigger(EntityGreeting {
        entity: listener,
        message: "第一次触发 EntityGreeting".to_string(),
    });
    commands.trigger(EntityGreeting {
        entity: listener,
        message: "第二次触发 EntityGreeting".to_string(),
    });
}

fn exit_after_observing(
    greeting_count: Res<GreetingCount>,
    entity_greeting_count: Res<EntityGreetingCount>,
    mut exit: MessageWriter<AppExit>,
) {
    if greeting_count.0 == 2 && entity_greeting_count.0 == 2 {
        println!("Observer 示例完成，退出应用");
        exit.write(AppExit::Success);
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_observer(observe_greeting)
        .init_resource::<GreetingCount>()
        .init_resource::<EntityGreetingCount>()
        .add_systems(Startup, trigger_greetings)
        .add_systems(Last, exit_after_observing)
        .run();
}
