use bevy::{prelude::*, state::app::StatesPlugin};

#[derive(Component, Debug, Default)]
struct DemoEntity;

#[derive(Resource, Debug)]
struct DemoScore(u32);

#[derive(Message, Debug)]
struct DemoMessage;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum DemoState {
    #[default]
    Ready,
}

fn setup(mut commands: Commands, mut messages: MessageWriter<DemoMessage>) {
    let entity = commands.spawn(DemoEntity).id();
    println!("Entity：创建 {entity:?}，添加组件 {DemoEntity:?}");

    messages.write(DemoMessage);
}

fn inspect_world(
    query: Query<(Entity, &DemoEntity)>,
    score: Res<DemoScore>,
    state: Res<State<DemoState>>,
    mut messages: MessageReader<DemoMessage>,
) {
    for (entity, component) in &query {
        println!("Component：{entity:?} 拥有 {component:?}");
    }

    println!("Resource：DemoScore({})", score.0);
    println!("State：当前状态 {:?}", state.get());

    for _message in messages.read() {
        println!("Message：收到一条 DemoMessage");
    }
}

// Plugin 集中配置 App：准备 World 中的数据，并注册要运行的系统。
struct AppWorldPlugin;

impl Plugin for AppWorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DemoScore(100))
            .add_message::<DemoMessage>()
            .init_state::<DemoState>()
            .add_systems(Startup, setup)
            .add_systems(Update, inspect_world);
    }
}

fn main() {
    App::new()
        .add_plugins(StatesPlugin)
        .add_plugins(AppWorldPlugin)
        .run();
}
