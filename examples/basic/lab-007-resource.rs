use bevy::prelude::*;

#[derive(Resource, Debug, Default, PartialEq)]
struct Counter(u32);

#[derive(Resource, Debug, Default)]
struct Settings {
    sound_enabled: bool,
}

#[derive(Resource, Debug, Default)]
struct OptionalLabel(String);

fn initialize(mut counter: ResMut<Counter>) {
    counter.set_if_neq(Counter(1));
    println!("Startup：初始化 Counter = {:?}", *counter);
}

fn advance(mut counter: ResMut<Counter>) {
    let next = counter.0 + 1;
    counter.set_if_neq(Counter(next));
}

fn inspect_counter(counter: Res<Counter>) {
    println!(
        "Res：Counter = {:?}, added={}, changed={}",
        *counter,
        counter.is_added(),
        counter.is_changed(),
    );
}

fn inspect_settings(settings: Res<Settings>) {
    println!(
        "init_resource：Settings {{ sound_enabled={} }}",
        settings.sound_enabled
    );
}

fn inspect_optional(label: Option<Res<OptionalLabel>>) {
    match label {
        Some(label) => println!("Option<Res>：label = {}", label.0),
        None => println!("Option<Res>：OptionalLabel 尚未插入"),
    }
}

fn settings_system(settings: Res<Settings>) {
    println!(
        "resource_exists：Settings 存在，System 可以运行（{:?}）",
        *settings
    );
}

fn main() {
    App::new()
        .insert_resource(Counter::default())
        .init_resource::<Settings>()
        .add_systems(Startup, initialize)
        .add_systems(
            Update,
            (
                advance,
                inspect_counter,
                inspect_settings,
                inspect_optional,
                settings_system.run_if(resource_exists::<Settings>),
            )
                .chain(),
        )
        .run();
}
