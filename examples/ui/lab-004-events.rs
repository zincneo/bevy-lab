use bevy::input_focus::AutoFocus;
use bevy::picking::events::{Click, Out, Over, Pointer, Press, Release};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

const POINTER_NORMAL: Color = Color::srgb(0.12, 0.18, 0.3);
const POINTER_HOVERED: Color = Color::srgb(0.18, 0.32, 0.5);
const POINTER_PRESSED: Color = Color::srgb(0.3, 0.2, 0.45);
const BUTTON_NORMAL: Color = Color::srgb(0.15, 0.15, 0.18);

#[derive(Component)]
struct PointerPanel;

#[derive(Component)]
struct PointerStatus;

#[derive(Component)]
struct KeyboardButtonLabel;

#[derive(Component)]
struct KeyboardStatus;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_shortcut)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                padding: px(32).all(),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(20),
                ..default()
            },
            BackgroundColor(Color::srgb(0.04, 0.06, 0.1)),
        ))
        .id();

    let pointer_panel = commands
        .spawn((
            PointerPanel,
            Hovered::default(),
            Node {
                width: px(380),
                height: px(160),
                padding: px(20).all(),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(10),
                border: px(3).all(),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(POINTER_NORMAL),
            BorderColor::all(Color::srgb(0.35, 0.55, 0.85)),
            children![
                (
                    Text::new("Pointer events"),
                    TextFont::from_font_size(24.0),
                    TextColor(Color::WHITE),
                ),
                (
                    PointerStatus,
                    Text::new("Move over this panel"),
                    TextFont::from_font_size(18.0),
                    TextColor(Color::srgb(0.8, 0.88, 1.0)),
                    TextLayout::justify(Justify::Center),
                ),
            ],
        ))
        .id();

    // 每个观察者只处理一种指针事件，便于观察事件的触发时机。
    commands
        .entity(pointer_panel)
        .observe(pointer_over)
        .observe(pointer_out)
        .observe(pointer_press)
        .observe(pointer_release)
        .observe(pointer_click);

    let keyboard_button = commands
        .spawn((
            Button,
            AutoFocus,
            Hovered::default(),
            Node {
                width: px(380),
                height: px(72),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: px(3).all(),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(BUTTON_NORMAL),
            BorderColor::all(Color::srgb(0.5, 0.5, 0.65)),
            children![(
                KeyboardButtonLabel,
                Text::new("Press Enter or Space"),
                TextFont::from_font_size(20.0),
                TextColor(Color::WHITE),
            )],
        ))
        .id();

    // ui_widgets::Button 会把鼠标点击和聚焦后的 Enter/Space 转换为 Activate。
    commands.entity(keyboard_button).observe(button_activated);

    let keyboard_status = commands
        .spawn((
            KeyboardStatus,
            Text::new("Global shortcut: press K or Escape"),
            TextFont::from_font_size(18.0),
            TextColor(Color::srgb(0.8, 0.85, 0.9)),
        ))
        .id();

    commands
        .entity(root)
        .add_children(&[pointer_panel, keyboard_button, keyboard_status]);
}

fn pointer_over(
    event: On<Pointer<Over>>,
    mut panels: Query<(&mut BackgroundColor, &mut BorderColor), With<PointerPanel>>,
    mut statuses: Query<&mut Text, With<PointerStatus>>,
) {
    info!("Pointer over {:?}", event.entity);
    if let Ok((mut background, mut border)) = panels.single_mut() {
        background.0 = POINTER_HOVERED;
        border.set_all(Color::srgb(0.45, 0.8, 1.0));
    }
    set_pointer_status(&mut statuses, "Hovered");
}

fn pointer_out(
    event: On<Pointer<Out>>,
    mut panels: Query<(&mut BackgroundColor, &mut BorderColor), With<PointerPanel>>,
    mut statuses: Query<&mut Text, With<PointerStatus>>,
) {
    info!("Pointer out {:?}", event.entity);
    if let Ok((mut background, mut border)) = panels.single_mut() {
        background.0 = POINTER_NORMAL;
        border.set_all(Color::srgb(0.35, 0.55, 0.85));
    }
    set_pointer_status(&mut statuses, "Pointer out");
}

fn pointer_press(
    event: On<Pointer<Press>>,
    mut panels: Query<&mut BackgroundColor, With<PointerPanel>>,
    mut statuses: Query<&mut Text, With<PointerStatus>>,
) {
    info!("Pointer pressed on {:?}", event.entity);
    if let Ok(mut background) = panels.single_mut() {
        background.0 = POINTER_PRESSED;
    }
    set_pointer_status(&mut statuses, "Pressed");
}

fn pointer_release(
    event: On<Pointer<Release>>,
    mut panels: Query<&mut BackgroundColor, With<PointerPanel>>,
    mut statuses: Query<&mut Text, With<PointerStatus>>,
) {
    info!("Pointer released on {:?}", event.entity);
    if let Ok(mut background) = panels.single_mut() {
        background.0 = POINTER_HOVERED;
    }
    set_pointer_status(&mut statuses, "Released");
}

fn pointer_click(event: On<Pointer<Click>>, mut statuses: Query<&mut Text, With<PointerStatus>>) {
    info!("Pointer clicked {:?}", event.entity);
    set_pointer_status(&mut statuses, "Clicked");
}

fn set_pointer_status(statuses: &mut Query<&mut Text, With<PointerStatus>>, value: &str) {
    if let Ok(mut status) = statuses.single_mut() {
        status.0 = value.to_string();
    }
}

fn button_activated(_event: On<Activate>, mut labels: Query<&mut Text, With<KeyboardButtonLabel>>) {
    info!("Button activated by pointer or keyboard");
    if let Ok(mut label) = labels.single_mut() {
        label.0 = "Activated: click, Enter, or Space".to_string();
    }
}

fn keyboard_shortcut(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut statuses: Query<&mut Text, With<KeyboardStatus>>,
) {
    let message = if keyboard.just_pressed(KeyCode::KeyK) {
        Some("Key K pressed")
    } else if keyboard.just_pressed(KeyCode::Escape) {
        Some("Escape pressed")
    } else {
        None
    };

    if let Some(message) = message
        && let Ok(mut status) = statuses.single_mut()
    {
        info!("{message}");
        status.0 = message.to_string();
    }
}
