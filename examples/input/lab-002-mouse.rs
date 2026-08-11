use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, mouse_input)
        .run();
}

/// 打印鼠标按钮状态、每帧累计的移动和滚轮输入。
fn mouse_input(
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        println!("Left mouse button: just pressed");
    }

    if mouse.pressed(MouseButton::Left) {
        println!("Left mouse button: pressed");
    }

    if mouse.just_released(MouseButton::Left) {
        println!("Left mouse button: just released");
    }

    if motion.delta != Vec2::ZERO {
        println!("Mouse moved: {:?}", motion.delta);
    }

    if scroll.delta != Vec2::ZERO {
        println!("Mouse scrolled ({:?}): {:?}", scroll.unit, scroll.delta);
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
