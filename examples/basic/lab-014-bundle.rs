use bevy::prelude::*;

#[derive(Component, Debug)]
struct Player;

#[derive(Component, Debug)]
struct Health(u32);

#[derive(Component, Debug)]
struct Speed(f32);

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    health: Health,
    speed: Speed,
}

fn spawn_player(mut commands: Commands) {
    commands.spawn(PlayerBundle {
        player: Player,
        health: Health(100),
        speed: Speed(4.0),
    });
}

fn inspect_player(query: Query<(Entity, &Health, &Speed), With<Player>>) {
    for (entity, health, speed) in query.iter() {
        println!(
            "Bundle 创建实体 {:?}：health={}，speed={}",
            entity, health.0, speed.0
        );
    }
}

fn exit_after_inspection(mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_systems(Startup, spawn_player)
        .add_systems(Update, inspect_player)
        .add_systems(Last, exit_after_inspection)
        .run();
}
