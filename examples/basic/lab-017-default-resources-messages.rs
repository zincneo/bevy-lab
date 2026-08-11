use bevy::{
    diagnostic::{DiagnosticsStore, FrameCount},
    prelude::*,
    window::{PrimaryWindow, WindowCloseRequested, WindowFocused, WindowResized},
    winit::{UpdateMode, WinitSettings},
};
use std::time::Duration;

#[derive(Resource, Default)]
struct Updates(u32);

fn report_core_resources(
    time: Res<Time>,
    frame_count: Res<FrameCount>,
    diagnostics: Res<DiagnosticsStore>,
    winit_settings: Res<WinitSettings>,
    window: Single<&Window, With<PrimaryWindow>>,
    updates: Res<Updates>,
) {
    println!(
        "Update {}: frame={}, delta={:?}, diagnostics={}, window={:?}, mode={:?}",
        updates.0,
        frame_count.0,
        time.delta(),
        diagnostics.iter().count(),
        window.resolution.size(),
        winit_settings.update_mode(window.focused),
    );
}

fn print_window_messages(
    mut resized: MessageReader<WindowResized>,
    mut focused: MessageReader<WindowFocused>,
    mut close_requested: MessageReader<WindowCloseRequested>,
) {
    for event in resized.read() {
        println!("WindowResized: {} x {}", event.width, event.height);
    }
    for event in focused.read() {
        println!("WindowFocused: {}", event.focused);
    }
    for event in close_requested.read() {
        println!("WindowCloseRequested: {:?}", event.window);
    }
}

fn advance_and_exit(
    mut updates: ResMut<Updates>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
) {
    updates.0 += 1;
    if keyboard.just_pressed(KeyCode::Escape) {
        println!("AppExit message written; leaving the core resource lab");
        exit.write(AppExit::Success);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // WinitSettings 控制窗口事件循环何时再次驱动 App 的 Schedule。
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::reactive(Duration::from_millis(250)),
            unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs(1)),
        })
        .init_resource::<Updates>()
        .add_systems(
            Update,
            (
                report_core_resources,
                print_window_messages,
                advance_and_exit,
            )
                .chain(),
        )
        .run();
}
