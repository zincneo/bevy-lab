use bevy::feathers::{
    FeathersPlugins,
    controls::{
        FeathersButton, FeathersCheckbox, FeathersRadio, FeathersSlider, FeathersTextInput,
        FeathersTextInputContainer,
    },
    dark_theme::create_dark_theme,
    display::{caption, label, label_dim},
    theme::{ThemeBackgroundColor, UiTheme},
    tokens,
};
use bevy::input_focus::{InputFocus, tab_navigation::TabIndex};
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::Checked;
use bevy::ui_widgets::{
    Activate, RadioGroup, ScrollArea, SliderStep, SliderValue, ValueChange, checkbox_self_update,
    radio_self_update, slider_self_update,
};

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
struct DemoRadio(RadioChoice);

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
        .add_plugins((DefaultPlugins, FeathersPlugins))
        .insert_resource(UiTheme(create_dark_theme()))
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
                commit_text_on_enter,
                show_widget_state,
                show_text_input_preview,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(demo_root());
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
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            label("Common UI widgets (Feathers)"),
            label_dim("The Feathers theme supplies the visual control structure"),
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
        BackgroundColor(Color::srgba(0.12, 0.14, 0.18, 0.8))
        BorderColor::all(Color::srgba(0.3, 0.34, 0.42, 0.8))
    }
}

fn button_panel() -> impl Scene {
    bsn! {
        panel()
        Children [
            label("Button"),
            (
                @FeathersButton {
                    @caption: bsn! { caption("Activate button") }
                }
                TabIndex(0)
                on(button_activated)
                Node { width: px(440) }
            ),
            (
                ButtonStatus
                label_dim("Activations: 0")
            ),
        ]
    }
}

fn checkbox_panel() -> impl Scene {
    bsn! {
        panel()
        Children [
            label("Checkbox"),
            (
                @FeathersCheckbox {
                    @caption: bsn! { caption("Enable feature") }
                }
                TabIndex(1)
                on(checkbox_self_update)
                on(checkbox_changed)
            ),
            (
                CheckboxStatus
                label_dim("Checked: false")
            ),
        ]
    }
}

fn radio_panel() -> impl Scene {
    bsn! {
        panel()
        Children [
            label("Radio group"),
            (
                Node {
                    width: px(440),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(6),
                }
                RadioGroup
                on(radio_self_update)
                on(radio_changed)
                Children [
                    (
                        @FeathersRadio {
                            @caption: bsn! { caption("First option") }
                        }
                        DemoRadio(RadioChoice::First)
                        Checked
                    ),
                    (
                        @FeathersRadio {
                            @caption: bsn! { caption("Second option") }
                        }
                        DemoRadio(RadioChoice::Second)
                    ),
                    (
                        @FeathersRadio {
                            @caption: bsn! { caption("Third option") }
                        }
                        DemoRadio(RadioChoice::Third)
                    ),
                ]
            ),
            (
                RadioStatus
                label_dim("Selected: First option")
            ),
        ]
    }
}

fn slider_panel() -> impl Scene {
    bsn! {
        panel()
        Children [
            label("Slider"),
            (
                @FeathersSlider {
                    @min: 0.0,
                    @max: 100.0,
                }
                SliderValue(50.0)
                SliderStep(1.0)
                on(slider_self_update)
                on(slider_value_changed)
                Node { width: px(440) }
            ),
            (
                SliderStatus
                label_dim("Value: 50%")
            ),
        ]
    }
}

fn text_input_panel() -> impl Scene {
    bsn! {
        panel()
        Children [
            label("Text input"),
            (
                @FeathersTextInputContainer
                Node { width: px(440), flex_grow: 0.0 }
                Children [(
                    @FeathersTextInput
                    DemoTextInput
                    TabIndex(4)
                )]
            ),
            (
                InputPreview
                label_dim("Click the input to edit")
            ),
            (
                CommittedTextLabel
                label("Committed value: <none>")
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

fn slider_value_changed(value_change: On<ValueChange<f32>>, mut state: ResMut<WidgetState>) {
    state.slider_percent = value_change.value;
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
