use bevy::input_focus::{
    InputFocus,
    tab_navigation::{TabGroup, TabIndex, TabNavigationPlugin},
};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui::{Checked, Pressed};
use bevy::ui_widgets::{
    Activate, Button, Checkbox, RadioButton, RadioGroup, ScrollArea, Slider, SliderRange,
    SliderStep, SliderThumb, SliderValue, TextInput, TrackClick, ValueChange, checkbox_self_update,
    radio_self_update,
};

const BACKGROUND: Color = Color::srgb(0.035, 0.05, 0.08);
const PANEL: Color = Color::srgb(0.08, 0.12, 0.19);
const TRACK: Color = Color::srgb(0.16, 0.22, 0.32);
const ACCENT: Color = Color::srgb(0.25, 0.65, 0.95);
const TEXT: Color = Color::srgb(0.9, 0.94, 1.0);
const MUTED_TEXT: Color = Color::srgb(0.65, 0.72, 0.82);

#[derive(Resource, Debug)]
struct WidgetState {
    button_activations: u32,
    checkbox_checked: bool,
    radio_choice: RadioChoice,
    slider_percent: f32,
    committed_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RadioChoice {
    First,
    Second,
    Third,
}

#[derive(Component)]
struct DemoButton;

#[derive(Component)]
struct DemoCheckbox;

#[derive(Component)]
struct CheckboxMark;

#[derive(Component)]
struct DemoRadio(RadioChoice);

#[derive(Component)]
struct RadioMark;

#[derive(Component)]
struct DemoSlider;

#[derive(Component)]
struct SliderThumbVisual;

#[derive(Component)]
struct DemoTextInput;

#[derive(Component)]
struct ButtonStatus;

#[derive(Component)]
struct CheckboxStatus;

#[derive(Component)]
struct RadioStatus;

#[derive(Component)]
struct SliderStatus;

#[derive(Component)]
struct InputPreview;

#[derive(Component)]
struct CommittedTextLabel;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TabNavigationPlugin))
        .insert_resource(WidgetState {
            button_activations: 0,
            checkbox_checked: false,
            radio_choice: RadioChoice::First,
            slider_percent: 50.0,
            committed_text: String::new(),
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_button_visual,
                update_check_visual,
                update_radio_visual,
                update_slider_thumb,
                commit_text_on_enter,
                show_widget_state,
                show_text_input_preview,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // The root is a vertical flex container. Its overflow setting makes the whole page
    // vertically scrollable when the window is shorter than the widget gallery.
    let root = commands
        .spawn((
            ScrollArea,
            Node {
                width: percent(100),
                height: percent(100),
                padding: px(28).all(),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(16),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(BACKGROUND),
            TabGroup::default(),
        ))
        .id();

    let title = commands
        .spawn((
            Text::new("Common UI widgets"),
            TextFont::from_font_size(30.0),
            TextColor(TEXT),
        ))
        .id();
    let description = commands
        .spawn((
            Text::new("Button, checkbox, radio group, slider, and text input"),
            TextFont::from_font_size(16.0),
            TextColor(MUTED_TEXT),
        ))
        .id();

    let button_panel = commands.spawn(panel()).id();
    let button_title = section_title(&mut commands, "Button");
    let button = commands
        .spawn((
            DemoButton,
            Button,
            Hovered::default(),
            TabIndex(0),
            Node {
                width: px(440),
                height: px(48),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: px(2).all(),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BorderColor::all(ACCENT),
            BackgroundColor(TRACK),
            children![(
                Text::new("Activate button"),
                TextFont::from_font_size(18.0),
                TextColor(TEXT),
            )],
        ))
        .id();
    let button_status = commands
        .spawn((
            ButtonStatus,
            Text::new("Activations: 0"),
            TextFont::from_font_size(16.0),
            TextColor(MUTED_TEXT),
        ))
        .id();
    commands.entity(button).observe(button_activated);
    commands
        .entity(button_panel)
        .add_children(&[button_title, button, button_status]);

    let checkbox_panel = commands.spawn(panel()).id();
    let checkbox_title = section_title(&mut commands, "Checkbox");
    let checkbox = commands
        .spawn((
            DemoCheckbox,
            Checkbox,
            Hovered::default(),
            TabIndex(1),
            Node {
                width: px(440),
                height: px(40),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(10),
                ..default()
            },
            Children::spawn((
                Spawn((
                    Node {
                        width: px(20),
                        height: px(20),
                        border: px(2).all(),
                        border_radius: BorderRadius::all(px(4)),
                        ..default()
                    },
                    BorderColor::all(ACCENT),
                    children![(
                        CheckboxMark,
                        Node {
                            width: px(10),
                            height: px(10),
                            position_type: PositionType::Absolute,
                            left: px(3),
                            top: px(3),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    )],
                )),
                Spawn((
                    Text::new("Enable feature"),
                    TextFont::from_font_size(18.0),
                    TextColor(TEXT),
                )),
            )),
        ))
        .id();
    let checkbox_status = commands
        .spawn((
            CheckboxStatus,
            Text::new("Checked: false"),
            TextFont::from_font_size(16.0),
            TextColor(MUTED_TEXT),
        ))
        .id();
    commands
        .entity(checkbox)
        .observe(checkbox_self_update)
        .observe(checkbox_changed);
    commands
        .entity(checkbox_panel)
        .add_children(&[checkbox_title, checkbox, checkbox_status]);

    let radio_panel = commands.spawn(panel()).id();
    let radio_title = section_title(&mut commands, "Radio group");
    let radio_group = commands
        .spawn((
            RadioGroup,
            TabIndex(2),
            Node {
                width: px(440),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
        ))
        .id();
    for (choice, caption) in [
        (RadioChoice::First, "First option"),
        (RadioChoice::Second, "Second option"),
        (RadioChoice::Third, "Third option"),
    ] {
        let mut radio = commands.spawn((
            DemoRadio(choice),
            RadioButton,
            Hovered::default(),
            Node {
                height: px(32),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(10),
                ..default()
            },
            Children::spawn((
                Spawn((
                    Node {
                        width: px(18),
                        height: px(18),
                        border: px(2).all(),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BorderColor::all(ACCENT),
                    children![(
                        RadioMark,
                        Node {
                            width: px(8),
                            height: px(8),
                            position_type: PositionType::Absolute,
                            left: px(3),
                            top: px(3),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                    )],
                )),
                Spawn((
                    Text::new(caption),
                    TextFont::from_font_size(18.0),
                    TextColor(TEXT),
                )),
            )),
        ));
        if choice == RadioChoice::First {
            radio.insert(Checked);
        }
        let radio = radio.id();
        commands.entity(radio_group).add_child(radio);
    }
    let radio_status = commands
        .spawn((
            RadioStatus,
            Text::new("Selected: First option"),
            TextFont::from_font_size(16.0),
            TextColor(MUTED_TEXT),
        ))
        .id();
    commands.entity(radio_group).observe(radio_self_update);
    commands.entity(radio_group).observe(radio_changed);
    commands
        .entity(radio_panel)
        .add_children(&[radio_title, radio_group, radio_status]);

    let slider_panel = commands.spawn(panel()).id();
    let slider_title = section_title(&mut commands, "Slider");
    let slider = commands
        .spawn((
            DemoSlider,
            Slider {
                track_click: TrackClick::Snap,
                ..default()
            },
            SliderValue(50.0),
            SliderRange::new(0.0, 100.0),
            SliderStep(1.0),
            TabIndex(3),
            Node {
                width: px(440),
                height: px(28),
                ..default()
            },
            Children::spawn((
                Spawn((
                    Node {
                        width: percent(100),
                        height: px(8),
                        margin: auto().all(),
                        border_radius: BorderRadius::all(px(4)),
                        ..default()
                    },
                    BackgroundColor(TRACK),
                )),
                Spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        right: px(16),
                        top: px(0),
                        bottom: px(0),
                        ..default()
                    },
                    Children::spawn((Spawn((
                        SliderThumb,
                        SliderThumbVisual,
                        Node {
                            width: px(16),
                            height: px(16),
                            position_type: PositionType::Absolute,
                            left: percent(50),
                            top: px(6),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(ACCENT),
                    )),)),
                )),
            )),
        ))
        .id();
    let slider_status = commands
        .spawn((
            SliderStatus,
            Text::new("Value: 50%"),
            TextFont::from_font_size(16.0),
            TextColor(MUTED_TEXT),
        ))
        .id();
    commands.entity(slider).observe(slider_value_changed);
    commands
        .entity(slider_panel)
        .add_children(&[slider_title, slider, slider_status]);

    let input_panel = commands.spawn(panel()).id();
    let input_title = section_title(&mut commands, "Text input");
    let text_input = commands
        .spawn((
            DemoTextInput,
            TextInput,
            EditableText::new(""),
            TextLayout::no_wrap(),
            TextFont::from_font_size(20.0),
            TextColor(TEXT),
            TextCursorStyle::default(),
            TabIndex(4),
            Node {
                width: px(440),
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
    let input_preview = commands
        .spawn((
            InputPreview,
            Text::new("Click the input to edit"),
            TextFont::from_font_size(16.0),
            TextColor(MUTED_TEXT),
        ))
        .id();
    let committed_label = commands
        .spawn((
            CommittedTextLabel,
            Text::new("Committed value: <none>"),
            TextFont::from_font_size(16.0),
            TextColor(TEXT),
        ))
        .id();
    commands.entity(input_panel).add_children(&[
        input_title,
        text_input,
        input_preview,
        committed_label,
    ]);

    commands.entity(root).add_children(&[
        title,
        description,
        button_panel,
        checkbox_panel,
        radio_panel,
        slider_panel,
        input_panel,
    ]);
}

fn panel() -> impl Bundle {
    (
        Node {
            width: px(520),
            padding: px(18).all(),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            border: px(1).all(),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BorderColor::all(Color::srgb(0.18, 0.28, 0.42)),
        BackgroundColor(PANEL),
    )
}

fn section_title(commands: &mut Commands, value: &str) -> Entity {
    commands
        .spawn((
            Text::new(value),
            TextFont::from_font_size(22.0),
            TextColor(TEXT),
        ))
        .id()
}

fn button_activated(activate: On<Activate>, mut state: ResMut<WidgetState>) {
    state.button_activations += 1;
    info!("Button activated on {:?}", activate.entity);
}

fn checkbox_changed(value_change: On<ValueChange<bool>>, mut state: ResMut<WidgetState>) {
    state.checkbox_checked = value_change.value;
}

fn radio_changed(
    value_change: On<ValueChange<Entity>>,
    radios: Query<&DemoRadio>,
    mut state: ResMut<WidgetState>,
) {
    if let Ok(radio) = radios.get(value_change.value) {
        state.radio_choice = radio.0;
    }
}

fn slider_value_changed(
    value_change: On<ValueChange<f32>>,
    mut state: ResMut<WidgetState>,
    mut commands: Commands,
) {
    state.slider_percent = value_change.value;
    commands
        .entity(value_change.source)
        .insert(SliderValue(value_change.value));
}

fn update_button_visual(
    mut buttons: Query<(&Hovered, Has<Pressed>, &mut BackgroundColor), With<DemoButton>>,
) {
    for (hovered, pressed, mut background) in &mut buttons {
        background.0 = if pressed {
            ACCENT
        } else if hovered.get() {
            Color::srgb(0.2, 0.32, 0.48)
        } else {
            TRACK
        };
    }
}

fn update_check_visual(
    checkboxes: Query<(Entity, Has<Checked>), With<DemoCheckbox>>,
    children: Query<&Children>,
    mut marks: Query<&mut BackgroundColor, With<CheckboxMark>>,
) {
    for (checkbox_entity, checked) in &checkboxes {
        for child in children.iter_descendants(checkbox_entity) {
            if let Ok(mut background) = marks.get_mut(child) {
                background.0 = if checked { ACCENT } else { Color::NONE };
            }
        }
    }
}

fn update_radio_visual(
    radios: Query<(Entity, Has<Checked>), With<DemoRadio>>,
    children: Query<&Children>,
    mut marks: Query<&mut BackgroundColor, With<RadioMark>>,
) {
    for (radio_entity, checked) in &radios {
        for child in children.iter_descendants(radio_entity) {
            if let Ok(mut background) = marks.get_mut(child) {
                background.0 = if checked { ACCENT } else { Color::NONE };
            }
        }
    }
}

fn update_slider_thumb(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange),
        (
            With<DemoSlider>,
            Or<(Changed<SliderValue>, Changed<SliderRange>)>,
        ),
    >,
    children: Query<&Children>,
    mut thumbs: Query<&mut Node, With<SliderThumbVisual>>,
) {
    for (slider_entity, value, range) in &sliders {
        let position = range.thumb_position(value.0) * 100.0;
        for child in children.iter_descendants(slider_entity) {
            if let Ok(mut node) = thumbs.get_mut(child) {
                node.left = percent(position);
            }
        }
    }
}

fn commit_text_on_enter(
    keyboard: Res<ButtonInput<KeyCode>>,
    input_focus: Res<InputFocus>,
    text_inputs: Query<(Entity, &EditableText), With<DemoTextInput>>,
    mut state: ResMut<WidgetState>,
) {
    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    let Some(focused_entity) = input_focus.get() else {
        return;
    };

    if let Ok((entity, editable_text)) = text_inputs.single()
        && entity == focused_entity
    {
        state.committed_text = editable_text.value().to_string();
        info!("Text input committed: {:?}", state.committed_text);
    }
}

fn show_widget_state(
    state: Res<WidgetState>,
    mut statuses: ParamSet<(
        Query<&mut Text, With<ButtonStatus>>,
        Query<&mut Text, With<CheckboxStatus>>,
        Query<&mut Text, With<RadioStatus>>,
        Query<&mut Text, With<SliderStatus>>,
        Query<&mut Text, With<CommittedTextLabel>>,
    )>,
) {
    if !state.is_changed() {
        return;
    }

    if let Ok(mut text) = statuses.p0().single_mut() {
        text.0 = format!("Activations: {}", state.button_activations);
    }
    if let Ok(mut text) = statuses.p1().single_mut() {
        text.0 = format!("Checked: {}", state.checkbox_checked);
    }
    if let Ok(mut text) = statuses.p2().single_mut() {
        text.0 = format!("Selected: {}", radio_choice_name(state.radio_choice));
    }
    if let Ok(mut text) = statuses.p3().single_mut() {
        text.0 = format!("Value: {:.0}%", state.slider_percent);
    }
    if let Ok(mut text) = statuses.p4().single_mut() {
        let value = if state.committed_text.is_empty() {
            "<none>"
        } else {
            &state.committed_text
        };
        text.0 = format!("Committed value: {value}");
    }
}

fn radio_choice_name(choice: RadioChoice) -> &'static str {
    match choice {
        RadioChoice::First => "First option",
        RadioChoice::Second => "Second option",
        RadioChoice::Third => "Third option",
    }
}

fn show_text_input_preview(
    input_focus: Res<InputFocus>,
    text_inputs: Query<(Entity, &EditableText), With<DemoTextInput>>,
    mut previews: Query<&mut Text, With<InputPreview>>,
) {
    let Ok((input_entity, editable_text)) = text_inputs.single() else {
        return;
    };

    let Ok(mut preview) = previews.single_mut() else {
        return;
    };

    if input_focus.get() == Some(input_entity) {
        preview.0 = format!("Editing: {}", editable_text.value());
    } else {
        preview.0 = "Click the input to edit".to_string();
    }
}
