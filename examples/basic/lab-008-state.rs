use bevy::{prelude::*, state::app::StatesPlugin};

#[derive(Resource, Default)]
struct Frame(u32);

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum AppMode {
    #[default]
    Menu,
    Playing,
    Paused,
}

fn enter_menu() {
    println!("OnEnter(Menu)：进入菜单");
}

fn exit_menu() {
    println!("OnExit(Menu)：离开菜单");
}

fn enter_playing() {
    println!("OnEnter(Playing)：开始运行");
}

fn exit_playing() {
    println!("OnExit(Playing)：停止运行");
}

fn enter_paused() {
    println!("OnEnter(Paused)：进入暂停");
}

fn menu_to_playing() {
    println!("OnTransition：Menu -> Playing");
}

fn count_frame(mut frame: ResMut<Frame>) {
    frame.0 += 1;
}

fn request_next_state(
    frame: Res<Frame>,
    mode: Res<State<AppMode>>,
    mut next_mode: ResMut<NextState<AppMode>>,
) {
    match (frame.0, mode.get()) {
        (1, AppMode::Menu) => next_mode.set(AppMode::Playing),
        (2, AppMode::Playing) => next_mode.set_if_different(AppMode::Paused),
        _ => {}
    }
}

fn playing_update() {
    println!("Update.run_if：当前正在执行 Playing 逻辑");
}

fn stop_after_three_frames(frame: Res<Frame>, mut exit: MessageWriter<AppExit>) {
    if frame.0 >= 3 {
        println!("完成三次循环，退出示例");
        exit.write(AppExit::Success);
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_resource::<Frame>()
        .init_state::<AppMode>()
        .add_systems(OnEnter(AppMode::Menu), enter_menu)
        .add_systems(OnExit(AppMode::Menu), exit_menu)
        .add_systems(OnEnter(AppMode::Playing), enter_playing)
        .add_systems(OnExit(AppMode::Playing), exit_playing)
        .add_systems(OnEnter(AppMode::Paused), enter_paused)
        .add_systems(
            OnTransition {
                exited: AppMode::Menu,
                entered: AppMode::Playing,
            },
            menu_to_playing,
        )
        .add_systems(Update, (count_frame, request_next_state).chain())
        .add_systems(Update, playing_update.run_if(in_state(AppMode::Playing)))
        .add_systems(Last, stop_after_three_frames)
        .run();
}
