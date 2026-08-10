use bevy::{prelude::*, state::app::StatesPlugin};

// Entity 只是身份，组件才保存实体的数据或标记。
#[derive(Component, Debug, Default)]
struct DemoEntity;

// Resource 是 World 中按类型唯一的全局数据。
#[derive(Resource, Debug)]
struct DemoScore(u32);

// Message 是系统之间传递的短消息；消息队列由 Bevy 放在 World 的 Messages<T> Resource 中。
#[derive(Message, Debug)]
struct DemoMessage;

// State 由 App 管理，当前值会以 State<T> Resource 放在 World 中。
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum DemoState {
    #[default]
    Ready,
}

// Startup 系统通过 Commands 创建 Entity，并向消息队列写入一条消息。
fn setup(mut commands: Commands, mut messages: MessageWriter<DemoMessage>) {
    let entity = commands.spawn(DemoEntity).id();
    println!("Entity：创建 {entity:?}，添加组件 {DemoEntity:?}");

    messages.write(DemoMessage);
}

// Update 系统用 Query、Resource、State 和 MessageReader 读取 World 中的数据。
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
    // 最小 App 不会自动安装 StatesPlugin，因此这里显式添加它，
    // 让 init_state 注册的状态转换 Schedule 可以正常工作。
    App::new()
        .add_plugins(StatesPlugin)
        .add_plugins(AppWorldPlugin)
        .run();
}
