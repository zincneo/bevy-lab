use bevy::{
    input::gamepad::{GamepadAxis, GamepadButton, GamepadConnectionEvent},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Update,
            (
                print_gamepad_connections,
                print_gamepad_state,
                quit_with_escape,
            ),
        )
        .run();
}

/// 连接和断开是瞬时消息，适合用 MessageReader 读取。
fn print_gamepad_connections(mut events: MessageReader<GamepadConnectionEvent>) {
    for event in events.read() {
        println!("gamepad {:?}: {:?}", event.gamepad, event.connection);
    }
}

/// 每个已连接的手柄都是一个带 Gamepad 组件的 Entity。
fn print_gamepad_state(gamepads: Query<(Entity, &Gamepad)>) {
    for (entity, gamepad) in &gamepads {
        if gamepad.just_pressed(GamepadButton::South) {
            println!("{entity:?}: South just pressed");
        }

        if gamepad.just_released(GamepadButton::South) {
            println!("{entity:?}: South just released");
        }

        if let Some(trigger) = gamepad.get(GamepadButton::RightTrigger2)
            && trigger.abs() > 0.01
        {
            println!("{entity:?}: RightTrigger2 = {trigger:.2}");
        }

        if let Some(left_stick_x) = gamepad.get(GamepadAxis::LeftStickX)
            && left_stick_x.abs() > 0.01
        {
            println!("{entity:?}: LeftStickX = {left_stick_x:.2}");
        }
    }
}

fn quit_with_escape(keyboard: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
