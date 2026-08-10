use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy, Default)]
struct Marker;

#[allow(unused)]
#[derive(Component, Debug)]
struct Health(u32);

#[derive(Component, Debug, Clone, Copy, Default)]
struct Visible;

#[derive(Resource, Debug, Default, PartialEq)]
struct Score(u32);

// Command 也可以是一个自定义类型；它会在命令队列应用时访问 World。
// 只需要给自定义类型实现Command特征
struct PrintCommand(&'static str);

impl Command for PrintCommand {
    type Out = ();

    fn apply(self, _world: &mut World) {
        println!("{}", self.0);
    }
}

// 一个系统中集中演示 Commands 的常用操作。所有修改都会稍后应用到 World。
fn commands_demo(mut commands: Commands) {
    // 创建空实体、带组件的实体，以及一批拥有相同组件的实体。
    let temporary = commands.spawn_empty().id();
    let player = commands.spawn((Marker, Health(100))).id();
    commands.spawn_batch([Marker, Marker]);

    // 通过 EntityCommands 修改已有实体；get_entity 是可失败的版本。
    commands.entity(player).insert(Visible);
    if let Ok(mut entity) = commands.get_entity(player) {
        entity.insert_if_new(Health(999)); // 已有 Health 时不会覆盖 100。
        entity.remove::<Visible>();
    }

    // 删除临时实体和资源。
    commands.entity(temporary).despawn();
    commands.init_resource::<Score>();
    commands.insert_resource(Score(10));
    commands.insert_resource_if_neq(Score(20));
    commands.remove_resource::<Score>();

    // 提交一个自定义命令，在队列真正应用时打印消息。
    commands.queue(PrintCommand("自定义 Command 已应用"));
    println!("已提交 spawn、insert、remove、despawn、resource 和 queue 操作");
}

// 命令应用后，查询 World，观察仍然存在的实体及其组件。
fn inspect_world(query: Query<(Entity, &Marker, Option<&Health>, Option<&Visible>)>) {
    for (entity, marker, health, visible) in &query {
        println!("实体 {entity:?}：{marker:?}, Health={health:?}, Visible={visible:?}");
    }
}

struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, commands_demo)
            .add_systems(Update, inspect_world);
    }
}

fn main() {
    App::new().add_plugins(CommandsPlugin).run();
}
