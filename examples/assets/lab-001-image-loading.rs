use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
    ui_widgets::{Activate, Button},
};

const IMAGE_PATH: &str = "images/bevy-icon.png";
const SPRITE_IMAGE_PATH: &str = "images/bevy-bird-dark.png";
const PANEL: Color = Color::srgb(0.08, 0.12, 0.19);
// Keep the full-screen UI background translucent so the lower Sprite can be seen after loading.
const BACKGROUND: Color = Color::srgba(0.035, 0.05, 0.08, 0.82);
const TEXT: Color = Color::srgb(0.9, 0.94, 1.0);
const MUTED_TEXT: Color = Color::srgb(0.65, 0.72, 0.82);

#[derive(Resource, Default)]
struct SpriteLoadState {
    requested: bool,
}

#[derive(Component)]
struct OnDemandSprite;

#[derive(Component)]
struct OnDemandStatus;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SpriteLoadState>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_sprite_load_status)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // The Sprite camera renders first. No Sprite entity or Sprite image is loaded yet.
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        RenderLayers::layer(1),
    ));

    // The UI camera renders afterwards, so the UI stays above the Sprite.
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(0),
        IsDefaultUiCamera,
    ));

    let ui_root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(14),
                ..default()
            },
            BackgroundColor(BACKGROUND),
            children![
                (
                    Text::new("Image loading: BSN and Bundle UI"),
                    TextFont::from_font_size(26.0),
                    TextColor(TEXT),
                ),
                (
                    Text::new(
                        "UI images load at startup; the Sprite loads after button activation"
                    ),
                    TextFont::from_font_size(15.0),
                    TextColor(MUTED_TEXT),
                ),
                (
                    Node {
                        width: px(620),
                        padding: UiRect::all(px(18)),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        column_gap: px(18),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(10)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                    children![bundle_image_panel(&asset_server.load(IMAGE_PATH))],
                ),
                (
                    OnDemandStatus,
                    Text::new("Sprite: not created (click the button below)"),
                    TextFont::from_font_size(15.0),
                    TextColor(MUTED_TEXT),
                ),
            ],
        ))
        .id();

    // The BSN scene is attached to the same UI root as a child.
    // Its ImageNode receives the asset path directly and resolves it through AssetServer.
    commands.spawn_scene(bsn_image_panel(ui_root));
}

fn load_sprite_on_click(
    _event: On<Activate>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<SpriteLoadState>,
    mut status: Query<&mut Text, With<OnDemandStatus>>,
) {
    if state.requested {
        return;
    }

    state.requested = true;

    // This is the first load call for the Sprite image. AssetServer returns a handle
    // immediately and loads/decodes the file asynchronously in the asset pipeline.
    let sprite_image = asset_server.load(SPRITE_IMAGE_PATH);
    commands.spawn((
        OnDemandSprite,
        Sprite::from_image(sprite_image),
        Transform::from_xyz(0.0, -250.0, 0.0),
        RenderLayers::layer(1),
    ));

    if let Ok(mut text) = status.single_mut() {
        text.0 = "Sprite created; waiting for asynchronous image loading...".into();
    }
}

fn update_sprite_load_status(
    asset_server: Res<AssetServer>,
    state: Res<SpriteLoadState>,
    sprites: Query<&Sprite, With<OnDemandSprite>>,
    mut status: Query<&mut Text, With<OnDemandStatus>>,
) {
    if !state.requested {
        return;
    }

    let Ok(sprite) = sprites.single() else {
        return;
    };

    let message = if asset_server.is_loaded_with_dependencies(&sprite.image) {
        "Sprite image loaded asynchronously and is ready"
    } else {
        "Sprite created; waiting for asynchronous image loading..."
    };

    if let Ok(mut text) = status.single_mut() {
        text.0 = message.into();
    }
}

fn bundle_image_panel(image: &Handle<Image>) -> impl Bundle {
    (
        Node {
            width: px(250),
            height: px(250),
            padding: UiRect::all(px(12)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(8),
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.08, 0.13)),
        children![
            (
                ImageNode::new(image.clone()),
                Node {
                    width: px(170),
                    height: px(170),
                    ..default()
                },
            ),
            (
                Text::new("UI Bundle: ImageNode::new(handle)"),
                TextFont::from_font_size(13.0),
                TextColor(TEXT),
            ),
        ],
    )
}

fn bsn_image_panel(parent: Entity) -> impl Scene {
    bsn! {
        ChildOf(parent)
        Node {
            width: px(250),
            height: px(250),
            padding: UiRect::all(px(12)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(8),
        }
        BackgroundColor(Color::srgb(0.05, 0.08, 0.13))
        Children [
            (
                ImageNode { image: "images/bevy-icon.png" }
                Node { width: px(170), height: px(170) }
            ),
            (
                Text::new("UI BSN: ImageNode { image: path }")
                TextFont { font_size: FontSize::Px(13.0) }
                TextColor(TEXT)
            ),
            (
                Button
                Node {
                    width: px(250),
                    height: px(44),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(px(6)),
                }
                BackgroundColor(Color::srgb(0.16, 0.28, 0.44))
                on(load_sprite_on_click)
                Children [(
                    Text::new("Load Sprite image on demand")
                    TextFont { font_size: FontSize::Px(14.0) }
                    TextColor(TEXT)
                )]
            ),
        ]
    }
}
