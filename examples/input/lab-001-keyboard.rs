use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, keyboard_input)
        .run();
}

/// 按 Escape 退出，并在终端打印几个常用的键盘状态。
fn keyboard_input(keyboard: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Space) {
        println!("Space: just pressed");
    }

    if keyboard.pressed(KeyCode::ArrowUp) {
        println!("ArrowUp: pressed");
    }

    if keyboard.just_released(KeyCode::ArrowUp) {
        println!("ArrowUp: just released");
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
