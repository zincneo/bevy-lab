use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2d);

    // 生成一个内存图片，避免示例依赖额外的外部资源文件。
    let image = images.add(Image::new_fill(
        Extent3d {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[35, 150, 210, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));

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
        BackgroundColor(Color::srgb(0.04, 0.06, 0.1)),
        children![
            (
                Node {
                    width: percent(100),
                    ..default()
                },
                Text::new("UI Visual Components"),
                TextFont::from_font_size(30.0),
                TextColor(Color::srgb(0.92, 0.95, 1.0)),
                TextLayout::justify(Justify::Center),
                TextShadow {
                    offset: Vec2::new(2.0, 2.0),
                    color: Color::srgba(0.0, 0.0, 0.0, 0.5),
                },
            ),
            (
                Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceEvenly,
                    column_gap: px(20),
                    ..default()
                },
                children![
                    (
                        Node {
                            width: px(220),
                            height: px(230),
                            padding: px(18).all(),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            row_gap: px(16),
                            border: px(3).all(),
                            border_radius: BorderRadius::all(px(12)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.3, 0.48)),
                        BorderColor::all(Color::srgb(0.4, 0.7, 1.0)),
                        BoxShadow::new(
                            Color::srgba(0.0, 0.0, 0.0, 0.35),
                            px(16),
                            px(16),
                            px(0),
                            px(32),
                        ),
                        children![(
                            Text::new("BackgroundColor"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::srgb(0.92, 0.96, 1.0)),
                        )],
                    ),
                    (
                        Node {
                            width: px(220),
                            height: px(230),
                            padding: px(18).all(),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            row_gap: px(16),
                            border: px(3).all(),
                            border_radius: BorderRadius::all(px(12)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.12, 0.18)),
                        BorderColor::all(Color::srgb(0.35, 0.45, 0.6)),
                        children![
                            (
                                ImageNode::new(image.clone()),
                                Node {
                                    width: px(180),
                                    height: px(110),
                                    ..default()
                                },
                            ),
                            (
                                Text::new("ImageNode"),
                                TextFont::from_font_size(20.0),
                                TextColor(Color::srgb(0.85, 0.9, 1.0)),
                            ),
                        ],
                    ),
                    (
                        Node {
                            width: px(220),
                            height: px(230),
                            padding: px(18).all(),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            row_gap: px(16),
                            border: px(3).all(),
                            border_radius: BorderRadius::all(px(12)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.25, 0.18, 0.3)),
                        BorderColor::all(Color::srgb(0.75, 0.5, 0.9)),
                        UiTransform {
                            translation: Val2::px(0, -8),
                            scale: Vec2::splat(0.92),
                            rotation: Rot2::radians(0.12),
                        },
                        children![(
                            Node {
                                width: percent(100),
                                ..default()
                            },
                            Text::new("Centered Text\nwith UiTransform"),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::srgb(1.0, 0.92, 1.0)),
                            TextLayout::justify(Justify::Center),
                        )],
                    ),
                ],
            ),
        ],
    ));
}
