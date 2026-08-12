use bevy::prelude::*;

#[derive(Component)]
struct AnimatedTransform;

#[derive(Component)]
struct AnimatedBackground;

#[derive(Component)]
struct AnimatedText;

#[derive(Component)]
struct AnimatedWidth;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, animate_ui)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            padding: px(24).all(),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(20),
            ..default()
        },
        BackgroundColor(Color::srgb(0.035, 0.05, 0.09)),
        children![
            (
                Text::new("UI animation examples"),
                TextFont::from_font_size(30.0),
                TextColor(Color::srgb(0.9, 0.95, 1.0)),
            ),
            (
                Node {
                    width: px(620),
                    padding: px(24).all(),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(20),
                    border: px(1).all(),
                    border_radius: BorderRadius::all(px(12)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.08, 0.11, 0.18)),
                BorderColor::all(Color::srgb(0.25, 0.35, 0.52)),
                children![transform_demo(), opacity_demo(), width_demo()],
            ),
        ],
    ));
}

fn transform_demo() -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: px(92),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: px(2).all(),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.16, 0.27, 0.43)),
        BorderColor::all(Color::srgb(0.35, 0.65, 1.0)),
        UiTransform::default(),
        AnimatedTransform,
        children![(
            Text::new("UiTransform: translate + scale + rotate"),
            TextFont::from_font_size(20.0),
            TextColor(Color::srgb(0.9, 0.96, 1.0)),
        )],
    )
}

fn opacity_demo() -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: px(92),
            padding: px(14).all(),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(10),
            ..default()
        },
        children![
            (
                Text::new("Opacity: animate each rendered component"),
                TextFont::from_font_size(18.0),
                TextColor(Color::srgba(0.9, 0.95, 1.0, 1.0)),
                AnimatedText,
            ),
            (
                Node {
                    width: px(360),
                    height: px(14),
                    border_radius: BorderRadius::all(px(7)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.7, 1.0, 1.0)),
                AnimatedBackground,
            ),
        ],
    )
}

fn width_demo() -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: px(92),
            padding: px(14).all(),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(10),
            ..default()
        },
        children![
            (
                Text::new("Node width: layout is recalculated each frame"),
                TextFont::from_font_size(18.0),
                TextColor(Color::srgb(0.9, 0.95, 1.0)),
            ),
            (
                Node {
                    width: px(120),
                    height: px(14),
                    border_radius: BorderRadius::all(px(7)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.95, 0.55, 0.2)),
                AnimatedWidth,
            ),
        ],
    )
}

fn animate_ui(
    time: Res<Time>,
    mut transforms: Query<&mut UiTransform, With<AnimatedTransform>>,
    mut backgrounds: Query<&mut BackgroundColor, With<AnimatedBackground>>,
    mut text_colors: Query<&mut TextColor, With<AnimatedText>>,
    mut widths: Query<&mut Node, With<AnimatedWidth>>,
) {
    let seconds = time.elapsed_secs();
    let pulse = 0.5 + 0.5 * (seconds * 2.0).sin();

    for mut transform in &mut transforms {
        transform.translation = Val2::px((seconds * 1.3).sin() * 70.0, 0.0);
        transform.scale = Vec2::splat(0.94 + pulse * 0.12);
        transform.rotation = Rot2::radians((seconds * 1.7).sin() * 0.12);
    }

    for mut background in &mut backgrounds {
        background.0 = Color::srgba(0.2, 0.7, 1.0, 0.25 + pulse * 0.75);
    }

    for mut text_color in &mut text_colors {
        text_color.0 = Color::srgba(0.9, 0.95, 1.0, 0.35 + pulse * 0.65);
    }

    for mut node in &mut widths {
        node.width = px(120.0 + pulse * 360.0);
    }
}
