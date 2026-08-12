use bevy::input_focus::{InputFocus, tab_navigation::TabIndex};
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui_widgets::{Activate, Button, TextInput};

const BACKGROUND: Color = Color::srgb(0.035, 0.05, 0.08);
const PANEL: Color = Color::srgb(0.08, 0.12, 0.19);
const BUTTON: Color = Color::srgb(0.16, 0.28, 0.44);
const ACCENT: Color = Color::srgb(0.25, 0.65, 0.95);
const TEXT: Color = Color::srgb(0.9, 0.94, 1.0);
const MUTED_TEXT: Color = Color::srgb(0.65, 0.72, 0.82);

#[derive(Resource)]
struct DynamicUiState {
    input: Entity,
    button_panel: Entity,
    button_count: usize,
}

#[derive(Component)]
struct DynamicButton(usize);

#[derive(Component)]
struct InputStatus;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, regenerate_buttons_on_enter)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                padding: px(24).all(),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(18),
                ..default()
            },
            BackgroundColor(BACKGROUND),
        ))
        .id();

    let title = commands
        .spawn((
            Text::new("Dynamic UI: create buttons from input"),
            TextFont::from_font_size(28.0),
            TextColor(TEXT),
        ))
        .id();

    let description = commands
        .spawn((
            Text::new("Enter a number and press Enter to rebuild the panel"),
            TextFont::from_font_size(16.0),
            TextColor(MUTED_TEXT),
        ))
        .id();

    let input = commands
        .spawn((
            TextInput,
            EditableText::new("3"),
            TextLayout::no_wrap(),
            TextFont::from_font_size(22.0),
            TextColor(TEXT),
            TextCursorStyle::default(),
            TabIndex(0),
            Node {
                width: px(220),
                min_height: px(48),
                padding: px(10).all(),
                border: px(2).all(),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BorderColor::all(ACCENT),
            BackgroundColor(PANEL),
        ))
        .id();

    let status = commands
        .spawn((
            InputStatus,
            Text::new("Current buttons: 3"),
            TextFont::from_font_size(16.0),
            TextColor(MUTED_TEXT),
        ))
        .id();

    let button_panel = commands
        .spawn((
            Node {
                width: px(620),
                min_height: px(170),
                padding: px(18).all(),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::FlexStart,
                column_gap: px(10),
                row_gap: px(10),
                border: px(1).all(),
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BackgroundColor(PANEL),
            BorderColor::all(Color::srgb(0.25, 0.35, 0.52)),
        ))
        .id();

    commands
        .entity(root)
        .add_children(&[title, description, input, status, button_panel]);

    spawn_dynamic_buttons(&mut commands, button_panel, 3);
    commands.insert_resource(DynamicUiState {
        input,
        button_panel,
        button_count: 3,
    });
}

fn regenerate_buttons_on_enter(
    keyboard: Res<ButtonInput<KeyCode>>,
    input_focus: Res<InputFocus>,
    inputs: Query<&EditableText>,
    mut status_text: Query<&mut Text, With<InputStatus>>,
    mut state: ResMut<DynamicUiState>,
    mut commands: Commands,
) {
    if !keyboard.just_pressed(KeyCode::Enter) || input_focus.get() != Some(state.input) {
        return;
    }

    let Ok(editable_text) = inputs.get(state.input) else {
        return;
    };

    let value = editable_text.value().to_string();
    let requested_count = value.trim().parse::<usize>().unwrap_or_default().min(12);

    if requested_count == state.button_count {
        return;
    }

    commands.entity(state.button_panel).despawn_children();
    spawn_dynamic_buttons(&mut commands, state.button_panel, requested_count);
    state.button_count = requested_count;

    if let Ok(mut text) = status_text.single_mut() {
        text.0 = format!("Current buttons: {requested_count}");
    }
}

fn spawn_dynamic_buttons(commands: &mut Commands, panel: Entity, count: usize) {
    for index in 0..count {
        let button = commands
            .spawn((
                Button,
                DynamicButton(index + 1),
                ChildOf(panel),
                Node {
                    width: px(132),
                    height: px(48),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                BackgroundColor(BUTTON),
                children![(
                    Text::new(format!("Button {}", index + 1)),
                    TextFont::from_font_size(16.0),
                    TextColor(TEXT),
                )],
            ))
            .id();

        commands.entity(button).observe(dynamic_button_activated);
    }
}

fn dynamic_button_activated(
    event: On<Activate>,
    buttons: Query<&DynamicButton>,
    mut status_text: Query<&mut Text, With<InputStatus>>,
) {
    let Ok(button) = buttons.get(event.entity) else {
        return;
    };

    if let Ok(mut text) = status_text.single_mut() {
        text.0 = format!("Activated dynamic button {}", button.0);
    }
}
