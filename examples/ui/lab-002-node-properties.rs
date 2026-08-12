use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
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
            row_gap: px(18),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.04, 0.06, 0.1)),
        children![
            (
                Text::new("Node Properties"),
                TextFont::from_font_size(28.0),
                TextColor(Color::srgb(0.92, 0.95, 1.0)),
            ),
            (
                Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Stretch,
                    column_gap: px(16),
                    ..default()
                },
                children![
                    (
                        Node {
                            width: px(260),
                            height: px(190),
                            padding: px(16).all(),
                            margin: px(6).all(),
                            border: px(3).all(),
                            border_radius: BorderRadius::all(px(12)),
                            box_sizing: BoxSizing::BorderBox,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            row_gap: px(12),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.2, 0.32)),
                        BorderColor::all(Color::srgb(0.35, 0.65, 0.9)),
                        children![(
                            Text::new("Fixed Size\npx + margin + padding\nborder + radius"),
                            TextFont::from_font_size(18.0),
                            TextColor(Color::srgb(0.9, 0.95, 1.0)),
                        )],
                    ),
                    (
                        Node {
                            width: percent(30),
                            min_width: px(180),
                            max_width: px(360),
                            height: px(190),
                            flex_grow: 1.0,
                            flex_shrink: 1.0,
                            flex_basis: px(180),
                            padding: px(16).all(),
                            border: px(3).all(),
                            border_radius: BorderRadius::all(px(12)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            row_gap: px(12),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.16, 0.25, 0.2)),
                        BorderColor::all(Color::srgb(0.4, 0.8, 0.55)),
                        children![(
                            Text::new("Percent Size\nmin / max\nflex grow / shrink"),
                            TextFont::from_font_size(18.0),
                            TextColor(Color::srgb(0.9, 1.0, 0.92)),
                        )],
                    ),
                    (
                        Node {
                            width: percent(30),
                            height: px(190),
                            min_width: px(180),
                            max_width: px(360),
                            padding: px(16).all(),
                            border: px(3).all(),
                            border_radius: BorderRadius::all(px(12)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            row_gap: px(12),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.28, 0.18, 0.14)),
                        BorderColor::all(Color::srgb(0.95, 0.62, 0.35)),
                        children![(
                            Text::new("Percent + Constraints\nparent provides\nlayout context"),
                            TextFont::from_font_size(18.0),
                            TextColor(Color::srgb(1.0, 0.94, 0.86)),
                        )],
                    ),
                ],
            ),
            (
                Node {
                    position_type: PositionType::Absolute,
                    right: px(24),
                    bottom: px(18),
                    width: px(190),
                    height: px(42),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(px(8)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.2, 0.45)),
                children![(
                    Text::new("Absolute Position"),
                    TextFont::from_font_size(16.0),
                    TextColor(Color::srgb(0.95, 0.9, 1.0)),
                )],
            ),
        ],
    ));
}
