use bevy::prelude::*;

#[derive(Component, Debug)]
struct Player;

#[derive(Component, Debug)]
struct Camera;

#[derive(Component, Debug)]
struct Disabled;

#[derive(Component, Debug)]
struct Position(i32);

#[derive(Component, Debug)]
struct Velocity(i32);

#[derive(Component, Debug)]
struct Health(u32);

#[derive(Resource, Debug)]
struct Step(i32);

#[derive(Resource, Debug, Default)]
struct Report {
    moved_players: u32,
}

// Commands 在 Startup 中创建后续系统需要的实体和 Resource。
fn setup(mut commands: Commands) {
    commands.insert_resource(Step(1));
    commands.insert_resource(Report::default());

    commands.spawn((Player, Position(0), Velocity(2), Health(100)));
    commands.spawn((Camera, Position(10)));
    println!("Startup：已通过 Commands 创建 Player 和 Camera");
}

// Query 的 D 使用元组，同时读取 Entity、可变组件和可选组件；F 使用元组过滤器。
fn update_players(
    mut players: Query<
        (Entity, &mut Position, Option<&Velocity>, Option<&Health>),
        (With<Player>, Without<Disabled>),
    >,
    step: Res<Step>,
    mut report: ResMut<Report>,
) {
    for (entity, mut position, velocity, health) in &mut players {
        let speed = velocity.map_or(0, |value| value.0);
        let health_value = health.map(|value| value.0);
        position.0 += speed * step.0;
        report.moved_players += 1;
        println!("Query：{entity:?} 移动到 {position:?}，Health={health_value:?}");
    }
}

// Single 要求恰好找到一个带 Camera 的实体；Res 只读访问全局资源。
fn update_camera(mut camera: Single<&mut Position, With<Camera>>, step: Res<Step>) {
    camera.0 += step.0;
    println!("Single：Camera 已移动");
}

// Populated 要求至少找到一个 Player；这里读取 ResMut 修改后的 Report。
fn report_players(players: Populated<&Position, With<Player>>, report: Res<Report>) {
    println!(
        "Populated：找到 {} 个 Player，移动次数 {}",
        players.iter().count(),
        report.moved_players
    );
}

// Local 属于当前系统自己，跨多次运行保留，不是 World 中共享的 Resource。
fn count_runs(mut runs: Local<u32>) {
    *runs += 1;
    println!("Local：当前系统已运行 {} 次", *runs);
}

struct SystemParameterPlugin;

impl Plugin for SystemParameterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (update_players, update_camera, report_players, count_runs).chain(),
        );
    }
}

fn main() {
    App::new().add_plugins(SystemParameterPlugin).run();
}
