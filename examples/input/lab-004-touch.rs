use bevy::{input::touch::Touches, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, (print_touch_input, quit_with_escape))
        .run();
}

/// Touches 是 World 中保存当前触摸点状态的 Resource。
fn print_touch_input(touches: Res<Touches>) {
    for touch in touches.iter_just_pressed() {
        println!(
            "touch {} just pressed at {:?}",
            touch.id(),
            touch.position()
        );
    }

    for touch in touches.iter() {
        if touch.delta() != Vec2::ZERO {
            println!("touch {} moved by {:?}", touch.id(), touch.delta());
        }
    }

    for touch in touches.iter_just_released() {
        println!("touch {} just released", touch.id());
    }

    for touch in touches.iter_just_canceled() {
        println!("touch {} canceled", touch.id());
    }
}

fn quit_with_escape(keyboard: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
