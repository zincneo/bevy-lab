use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

const BACKGROUND: Color = Color::srgb(0.035, 0.05, 0.08);
const PANEL: Color = Color::srgb(0.08, 0.12, 0.19);
const BUTTON: Color = Color::srgb(0.16, 0.28, 0.44);
const TEXT: Color = Color::srgb(0.9, 0.94, 1.0);
const MUTED_TEXT: Color = Color::srgb(0.65, 0.72, 0.82);

/// The static BSN tree uses this component as the insertion point for runtime children.
#[derive(Component, Clone, Default)]
struct DynamicContent;

/// This text belongs to the static shell and reports actions from dynamic children.
#[derive(Component, Clone, Default)]
struct StatusLabel;

/// Runtime data can come from a configuration file, a server, or another game system.
#[derive(Resource)]
struct DynamicItems {
    labels: Vec<String>,
}

/// Each generated button keeps the data that it represents on its own entity.
#[derive(Component)]
struct DynamicAction(String);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(DynamicItems {
            labels: ["Inventory", "Quests", "Settings", "Credits"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, populate_dynamic_content)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    // Only the stable shell is described here. Its dynamic container starts empty.
    commands.spawn_scene(static_root());
}

/// The outer panel, title, status text, and insertion point are static BSN content.
fn static_root() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(28)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        BackgroundColor(BACKGROUND)
        Children [
            (
                Node {
                    width: px(620),
                    padding: UiRect::all(px(22)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(14),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(10)),
                }
                BackgroundColor(PANEL)
                BorderColor::all(Color::srgb(0.25, 0.35, 0.52))
                Children [
                    (
                        Text::new("Static BSN shell with dynamic content")
                        TextFont { font_size: FontSize::Px(25.0) }
                        TextColor(TEXT)
                    ),
                    (
                        Text::new("The buttons below are created after this scene is spawned")
                        TextFont { font_size: FontSize::Px(15.0) }
                        TextColor(MUTED_TEXT)
                    ),
                    (
                        DynamicContent
                        Node {
                            width: percent(100),
                            min_height: px(86),
                            padding: UiRect::all(px(12)),
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            align_content: AlignContent::FlexStart,
                            column_gap: px(10),
                            row_gap: px(10),
                        }
                        BackgroundColor(Color::srgb(0.05, 0.08, 0.13))
                    ),
                    (
                        StatusLabel
                        Text::new("Waiting for runtime data...")
                        TextFont { font_size: FontSize::Px(15.0) }
                        TextColor(MUTED_TEXT)
                    ),
                ]
            ),
        ]
    }
}

/// Populate the marked container once it has appeared in the world.
fn populate_dynamic_content(
    mut commands: Commands,
    containers: Query<Entity, Added<DynamicContent>>,
    items: Res<DynamicItems>,
    mut status: Query<&mut Text, With<StatusLabel>>,
) {
    for container in &containers {
        for label in &items.labels {
            let button = commands
                .spawn((
                    Button,
                    DynamicAction(label.clone()),
                    ChildOf(container),
                    Node {
                        width: px(132),
                        height: px(44),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    },
                    BackgroundColor(BUTTON),
                    children![(
                        Text::new(label.clone()),
                        TextFont::from_font_size(16.0),
                        TextColor(TEXT),
                    )],
                ))
                .id();

            commands.entity(button).observe(dynamic_button_activated);
        }

        if let Ok(mut text) = status.single_mut() {
            text.0 = format!(
                "{} dynamic buttons were added by Update",
                items.labels.len()
            );
        }
    }
}

fn dynamic_button_activated(
    event: On<Activate>,
    actions: Query<&DynamicAction>,
    mut status: Query<&mut Text, With<StatusLabel>>,
) {
    let Ok(action) = actions.get(event.entity) else {
        return;
    };

    if let Ok(mut text) = status.single_mut() {
        text.0 = format!("Activated dynamic item: {}", action.0);
    }
}
