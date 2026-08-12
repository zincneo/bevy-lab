use bevy::feathers::{
    FeathersPlugins,
    dark_theme::create_dark_theme,
    theme::{ThemeBackgroundColor, ThemeBorderColor, ThemeProps, ThemeTextColor, UiTheme},
    tokens,
};
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

/// A reusable style scene, similar to a semantic CSS class for panels.
#[derive(SceneComponent, Clone, Default)]
struct PanelStyle;

impl PanelStyle {
    fn scene() -> impl Scene {
        bsn! {
            Node {
                width: px(500),
                padding: UiRect::all(px(18)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(8)),
            }
            ThemeBackgroundColor(tokens::PANE_BODY_BG)
            ThemeBorderColor(tokens::PANE_HEADER_BORDER)
        }
    }
}

/// A reusable primary-button style. Its children are supplied by the caller.
#[derive(SceneComponent, Clone, Default)]
struct PrimaryButtonStyle;

impl PrimaryButtonStyle {
    fn scene() -> impl Scene {
        bsn! {
            Button
            Node {
                width: px(210),
                min_height: px(46),
                padding: UiRect::all(px(10)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(6)),
            }
            ThemeBackgroundColor(tokens::BUTTON_PRIMARY_BG)
        }
    }
}

#[derive(Component, Clone, Default)]
struct ThemeStatus;

#[derive(Resource)]
struct ThemeMode {
    dark: bool,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FeathersPlugins))
        .insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(ThemeMode { dark: true })
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_theme)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(ui_root());
}

fn ui_root() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            padding: UiRect::all(px(28)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(16),
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            (
                Text::new("Reusable styles and a global theme")
                TextFont { font_size: FontSize::Px(28.0) }
                ThemeTextColor(tokens::TEXT_MAIN)
            ),
            (
                Text::new("Press Space to replace the UiTheme resource")
                TextFont { font_size: FontSize::Px(16.0) }
                ThemeTextColor(tokens::TEXT_DIM)
            ),
            (
                @PanelStyle
                Children [
                    (
                        Text::new("The same PanelStyle can be reused")
                        TextFont { font_size: FontSize::Px(20.0) }
                        ThemeTextColor(tokens::TEXT_MAIN)
                    ),
                    (
                        @PrimaryButtonStyle
                        on(button_activated)
                        Children [(
                            Text::new("Start")
                            TextFont { font_size: FontSize::Px(16.0) }
                            ThemeTextColor(tokens::BUTTON_PRIMARY_TEXT)
                        )]
                    ),
                    (
                        @PrimaryButtonStyle
                        on(button_activated)
                        Children [(
                            Text::new("Continue")
                            TextFont { font_size: FontSize::Px(16.0) }
                            ThemeTextColor(tokens::BUTTON_PRIMARY_TEXT)
                        )]
                    ),
                ]
            ),
            (
                ThemeStatus
                Text::new("Theme: dark | Click a button")
                TextFont { font_size: FontSize::Px(16.0) }
                ThemeTextColor(tokens::TEXT_DIM)
            ),
        ]
    }
}

fn toggle_theme(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<ThemeMode>,
    mut theme: ResMut<UiTheme>,
    mut status: Query<&mut Text, With<ThemeStatus>>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    mode.dark = !mode.dark;
    theme.0 = if mode.dark {
        create_dark_theme()
    } else {
        create_light_theme()
    };

    if let Ok(mut text) = status.single_mut() {
        text.0 = format!(
            "Theme: {} | Click a button",
            if mode.dark { "dark" } else { "light" }
        );
    }
}

fn button_activated(_event: On<Activate>, mut status: Query<&mut Text, With<ThemeStatus>>) {
    if let Ok(mut text) = status.single_mut() {
        text.0 = "A reusable styled button was activated".into();
    }
}

fn create_light_theme() -> ThemeProps {
    use bevy::feathers::tokens::semantic;

    let mut theme = create_dark_theme();
    theme
        .semantic_base
        .insert(semantic::SURFACE_WINDOW, Color::srgb(0.92, 0.94, 0.98));
    theme
        .semantic_base
        .insert(semantic::SURFACE_PANE_BODY, Color::srgb(0.98, 0.98, 1.0));
    theme
        .semantic_base
        .insert(semantic::BORDER_DEFAULT, Color::srgb(0.62, 0.67, 0.76));
    theme
        .semantic_base
        .insert(semantic::TEXT_DEFAULT, Color::srgb(0.10, 0.13, 0.20));
    theme
        .semantic_base
        .insert(semantic::TEXT_DIM, Color::srgb(0.30, 0.36, 0.46));
    theme
        .semantic_base
        .insert(semantic::FILL_ACCENT_DEFAULT, Color::srgb(0.12, 0.42, 0.78));
    theme
        .semantic_base
        .insert(semantic::FILL_ACCENT_HOVER, Color::srgb(0.16, 0.50, 0.88));
    theme
        .semantic_base
        .insert(semantic::FILL_ACCENT_PRESSED, Color::srgb(0.08, 0.32, 0.62));
    theme
}
