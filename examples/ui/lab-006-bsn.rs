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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RadioChoice {
    #[default]
    First,
    Second,
    Third,
}

#[derive(Component, Clone, Default)]
struct DemoButton;

#[derive(Component, Clone, Default)]
struct DemoCheckbox;

#[derive(Component, Clone, Default)]
struct CheckboxMark;

#[derive(Component, Clone, Default)]
struct DemoRadio(RadioChoice);

#[derive(Component, Clone, Default)]
struct RadioMark;

#[derive(Component, Clone, Default)]
struct DemoSlider;

#[derive(Component, Clone, Default)]
struct SliderThumbVisual;

#[derive(Component, Clone, Default)]
struct DemoTextInput;

#[derive(Component, Clone, Default)]
struct ButtonStatus;

#[derive(Component, Clone, Default)]
struct CheckboxStatus;

#[derive(Component, Clone, Default)]
struct RadioStatus;

#[derive(Component, Clone, Default)]
struct SliderStatus;

#[derive(Component, Clone, Default)]
struct InputPreview;

#[derive(Component, Clone, Default)]
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
        .add_systems(Startup, (setup, initialize_radio_selection).chain())
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
    // The entire static UI tree is declared by bsn! in demo_root.
    commands.spawn_scene(demo_root());
}

fn initialize_radio_selection(mut commands: Commands, radios: Query<(Entity, &DemoRadio)>) {
    for (entity, radio) in &radios {
        if radio.0 == RadioChoice::First {
            commands.entity(entity).insert(Checked);
        }
    }
}

fn demo_root() -> impl Scene {
    bsn! {
        ScrollArea
        Node {
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(28)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(16),
            overflow: Overflow::scroll_y(),
        }
        BackgroundColor(BACKGROUND)
        TabGroup
        Children [
            (
                Text::new("Common UI widgets (bsn!)")
                TextFont { font_size: FontSize::Px(30.0) }
                TextColor(TEXT)
            ),
            (
                Text::new("Static entity structure declared outside the setup system")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(MUTED_TEXT)
            ),
            button_panel(),
            checkbox_panel(),
            radio_panel(),
            slider_panel(),
            text_input_panel(),
        ]
    }
}

fn panel() -> impl Scene {
    bsn! {
        Node {
            width: px(520),
            padding: UiRect::all(px(18)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
        }
        BorderColor::all(Color::srgb(0.18, 0.28, 0.42))
        BackgroundColor(PANEL)
    }
}

fn section_title(value: &'static str) -> impl Scene {
    bsn! {
        Text::new(value)
        TextFont { font_size: FontSize::Px(22.0) }
        TextColor(TEXT)
    }
}

fn button_panel() -> impl Scene {
    bsn! {
        panel()
        Children [
            section_title("Button"),
            (
                DemoButton
                Button
                Hovered
                TabIndex(0)
                on(button_activated)
                Node {
                    width: px(440),
                    height: px(48),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                }
                BorderColor::all(ACCENT)
                BackgroundColor(TRACK)
                Children [(
                    Text::new("Activate button")
                    TextFont { font_size: FontSize::Px(18.0) }
                    TextColor(TEXT)
                )]
            ),
            (
                ButtonStatus
                Text::new("Activations: 0")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(MUTED_TEXT)
            ),
        ]
    }
}

fn checkbox_panel() -> impl Scene {
    bsn! {
        panel()
        Children [
            section_title("Checkbox"),
            (
                DemoCheckbox
                Checkbox
                Hovered
                TabIndex(1)
                on(checkbox_self_update)
                on(checkbox_changed)
                Node {
                    width: px(440),
                    height: px(40),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(10),
                }
                Children [
                    (
                        Node {
                            width: px(20),
                            height: px(20),
                            border: UiRect::all(px(2)),
                            border_radius: BorderRadius::all(px(4)),
                        }
                        BorderColor::all(ACCENT)
                        Children [(
                            CheckboxMark
                            Node {
                                width: px(10),
                                height: px(10),
                                position_type: PositionType::Absolute,
                                left: px(3),
                                top: px(3),
                            }
                            BackgroundColor(Color::NONE)
                        )]
                    ),
                    (
                        Text::new("Enable feature")
                        TextFont { font_size: FontSize::Px(18.0) }
                        TextColor(TEXT)
                    ),
                ]
            ),
            (
                CheckboxStatus
                Text::new("Checked: false")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(MUTED_TEXT)
            ),
        ]
    }
}

fn radio_panel() -> impl Scene {
    bsn! {
        panel()
        Children [
            section_title("Radio group"),
            (
                RadioGroup
                TabIndex(2)
                on(radio_self_update)
                on(radio_changed)
                Node {
                    width: px(440),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                }
                Children [
                    radio_option(RadioChoice::First, "First option"),
                    radio_option(RadioChoice::Second, "Second option"),
                    radio_option(RadioChoice::Third, "Third option"),
                ]
            ),
            (
                RadioStatus
                Text::new("Selected: First option")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(MUTED_TEXT)
            ),
        ]
    }
}

fn radio_option(choice: RadioChoice, caption: &'static str) -> impl Scene {
    bsn! {
        (
            DemoRadio(choice)
            RadioButton
            Hovered
            Node {
                height: px(32),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(10),
            }
            Children [
                (
                    Node {
                        width: px(18),
                        height: px(18),
                        border: UiRect::all(px(2)),
                        border_radius: BorderRadius::MAX,
                    }
                    BorderColor::all(ACCENT)
                    Children [(
                        RadioMark
                        Node {
                            width: px(8),
                            height: px(8),
                            position_type: PositionType::Absolute,
                            left: px(3),
                            top: px(3),
                            border_radius: BorderRadius::MAX,
                        }
                        BackgroundColor(Color::NONE)
                    )]
                ),
                (
                    Text::new(caption)
                    TextFont { font_size: FontSize::Px(18.0) }
                    TextColor(TEXT)
                ),
            ]
        )
    }
}

fn slider_panel() -> impl Scene {
    bsn! {
        panel()
        Children [
            section_title("Slider"),
            (
                DemoSlider
                Slider {
                    track_click: TrackClick::Snap,
                }
                SliderValue(50.0)
                SliderRange::new(0.0, 100.0)
                SliderStep(1.0)
                TabIndex(3)
                on(slider_value_changed)
                Node {
                    width: px(440),
                    height: px(28),
                }
                Children [
                    (
                        Node {
                            width: percent(100),
                            height: px(8),
                            margin: UiRect::all(auto()),
                            border_radius: BorderRadius::all(px(4)),
                        }
                        BackgroundColor(TRACK)
                    ),
                    (
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            right: px(16),
                            top: px(0),
                            bottom: px(0),
                        }
                        Children [(
                            SliderThumb
                            SliderThumbVisual
                            Node {
                                width: px(16),
                                height: px(16),
                                position_type: PositionType::Absolute,
                                left: percent(50),
                                top: px(6),
                                border_radius: BorderRadius::MAX,
                            }
                            BackgroundColor(ACCENT)
                        )]
                    ),
                ]
            ),
            (
                SliderStatus
                Text::new("Value: 50%")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(MUTED_TEXT)
            ),
        ]
    }
}

fn text_input_panel() -> impl Scene {
    bsn! {
        panel()
        Children [
            section_title("Text input"),
            (
                DemoTextInput
                TextInput
                EditableText::new("")
                TextLayout::no_wrap()
                TextFont { font_size: FontSize::Px(20.0) }
                TextColor(TEXT)
                TextCursorStyle::default()
                TabIndex(4)
                Node {
                    width: px(440),
                    min_height: px(48),
                    padding: UiRect::all(px(10)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                }
                BorderColor::all(ACCENT)
                BackgroundColor(PANEL)
            ),
            (
                InputPreview
                Text::new("Click the input to edit")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(MUTED_TEXT)
            ),
            (
                CommittedTextLabel
                Text::new("Committed value: <none>")
                TextFont { font_size: FontSize::Px(16.0) }
                TextColor(TEXT)
            ),
        ]
    }
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
