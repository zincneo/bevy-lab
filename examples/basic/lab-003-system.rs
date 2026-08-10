use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
struct Counter(u32);

// Startup System：只在应用启动阶段运行一次。
fn setup(mut counter: ResMut<Counter>) {
    counter.0 = 1;
    println!("Startup：初始化 Counter({})", counter.0);
}

// Update System：通过 Res 读取 Resource，通过 Local 保存系统自己的状态。
fn update(counter: Res<Counter>, mut runs: Local<u32>) {
    *runs += 1;
    println!("Update：Counter({})，System 运行 {} 次", counter.0, *runs);
}

struct SystemOverviewPlugin;

impl Plugin for SystemOverviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, update);
    }
}

fn main() {
    // App runner 会按照 Startup、Update 等 Schedule 驱动已注册的 System。
    App::new()
        .insert_resource(Counter::default())
        .add_plugins(SystemOverviewPlugin)
        .run();
}
