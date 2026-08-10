use bevy::prelude::*;

#[derive(Component, Debug)]
struct Player;

#[derive(Component, Debug)]
struct Enemy;

#[derive(Component, Debug)]
struct Label(&'static str);

#[derive(Component, Debug)]
struct Position(Vec2);

#[derive(Component, Debug)]
struct Velocity(Vec2);

#[derive(Component, Debug, Default)]
struct Selected;

#[derive(Resource, Default)]
struct ImportantEntities {
    player: Option<Entity>,
    enemy: Option<Entity>,
}

fn setup(mut commands: Commands, mut ids: ResMut<ImportantEntities>) {
    let player = commands
        .spawn((
            Player,
            Label("player"),
            Position(Vec2::ZERO),
            Velocity(Vec2::X),
        ))
        .id();
    let enemy = commands
        .spawn((Enemy, Label("enemy"), Position(Vec2::new(4.0, 0.0))))
        .id();
    commands.spawn((Label("selected prop"), Position(Vec2::Y), Selected));

    ids.player = Some(player);
    ids.enemy = Some(enemy);
}

// 读取多个组件，并用 Option 处理不是所有实体都拥有的 Velocity。
fn inspect_all(query: Query<(Entity, &Label, &Position, Option<&Velocity>)>) {
    for (entity, label, position, velocity) in &query {
        let velocity = velocity.map(|velocity| velocity.0);
        println!(
            "基本查询：{entity:?} {} 位置={:?} 速度={velocity:?}",
            label.0, position.0
        );
    }
}

// &mut Query 只修改拥有 Player 标记的实体。
fn move_player(mut query: Query<&mut Position, With<Player>>) {
    for mut position in &mut query {
        position.0.x += 1.0;
        println!("可变查询：player 移动到 {:?}", position.0);
    }
}

fn inspect_filters(
    added: Query<&Label, Added<Velocity>>,
    changed: Query<&Label, Changed<Position>>,
    player_or_enemy: Query<&Label, Or<(With<Player>, With<Enemy>)>>,
    not_selected: Query<&Label, Without<Selected>>,
) {
    for label in &added {
        println!("Added：{} 最近添加了 Velocity", label.0);
    }
    for label in &changed {
        println!("Changed：{} 的 Position 发生变化", label.0);
    }
    for label in &player_or_enemy {
        println!("Or：{} 匹配 Player 或 Enemy", label.0);
    }
    for label in &not_selected {
        println!("Without：{} 没有 Selected", label.0);
    }
}

fn inspect_single_and_populated(
    player: Single<(&Label, &Position), With<Player>>,
    enemies: Populated<&Label, With<Enemy>>,
) {
    let (label, position) = player.into_inner();
    println!("Single：{} 的位置是 {:?}", label.0, position.0);

    for label in &enemies {
        println!("Populated：至少有一个 Enemy，当前是 {}", label.0);
    }
}

fn inspect_optional_data(query: Query<(&Label, Option<&Velocity>)>) {
    for (label, velocity) in &query {
        println!("Option：{} 速度={velocity:?}", label.0);
    }
}

fn inspect_by_entity(ids: Res<ImportantEntities>, query: Query<&Label>) {
    if let Some(player) = ids.player {
        match query.get(player) {
            Ok(label) => println!("get：Entity {player:?} 是 {}", label.0),
            Err(error) => println!("get：查询失败 {error:?}"),
        }
    }

    if let (Some(player), Some(enemy)) = (ids.player, ids.enemy)
        && let Ok([player_label, enemy_label]) = query.get_many([player, enemy])
    {
        println!(
            "get_many：{} 和 {} 都能通过 Entity ID 查询",
            player_label.0, enemy_label.0
        );
    }
}

fn inspect_combinations(query: Query<&Label>) {
    for [first, second] in query.iter_combinations() {
        println!("组合查询：{} 与 {} 是一组实体组合", first.0, second.0);
    }
}

fn main() {
    App::new()
        .insert_resource(ImportantEntities::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_player,
                inspect_all,
                inspect_filters,
                inspect_single_and_populated,
                inspect_optional_data,
                inspect_by_entity,
                inspect_combinations,
            )
                .chain(),
        )
        .run();
}
