use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
struct Counter(u32);

fn setup(mut counter: ResMut<Counter>) {
    counter.0 = 1;
    println!("Startup：初始化 Counter({})", counter.0);
}

fn update(counter: Res<Counter>, mut runs: Local<u32>) {
    *runs += 1;
    println!("Update：Counter({})，System 运行 {} 次", counter.0, *runs);
}

fn main() {
    App::new()
        .insert_resource(Counter::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}
