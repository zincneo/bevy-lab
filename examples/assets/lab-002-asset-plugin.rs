use bevy::{
    asset::{AssetPlugin, LoadState},
    prelude::*,
    window::WindowPlugin,
};

const IMAGE_PATH: &str = "images/bevy-icon.png";

#[derive(Resource)]
struct AssetDemo {
    image: Handle<Image>,
    updates: u32,
    reported_loading: bool,
}

fn main() {
    App::new()
        // Hide the window because this lab reports the loading states in the terminal.
        // DefaultPlugins still supplies AssetPlugin and the normal format-specific loaders.
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: "assets".into(),
                    ..default()
                }),
        )
        .add_systems(Startup, request_image)
        .add_systems(Update, observe_image_loading)
        .run();
}

fn request_image(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("AssetPlugin root: assets/");
    println!("A path string alone does not load anything: {IMAGE_PATH}");

    // load() starts an asynchronous request immediately. It does not wait for
    // disk I/O or decoding, and no entity needs to use the handle yet.
    let image = asset_server.load::<Image>(IMAGE_PATH);
    println!(
        "AssetServer::load({IMAGE_PATH:?}) returned a handle immediately; current state: {:?}",
        asset_server.load_state(image.id())
    );

    commands.insert_resource(AssetDemo {
        image,
        updates: 0,
        reported_loading: false,
    });
}

fn observe_image_loading(
    mut demo: ResMut<AssetDemo>,
    asset_server: Res<AssetServer>,
    mut exit: MessageWriter<AppExit>,
) {
    demo.updates += 1;
    let state = asset_server.load_state(demo.image.id());

    if !demo.reported_loading && matches!(state, LoadState::Loading) {
        demo.reported_loading = true;
        println!("The request is running asynchronously; no entity uses this handle yet.");
    }

    if asset_server.is_loaded_with_dependencies(&demo.image) {
        println!(
            "The image is now in memory after {} Update cycles; the handle can be used by ImageNode or Sprite.",
            demo.updates
        );
        exit.write(AppExit::Success);
    } else if demo.updates >= 30 {
        println!("The image did not finish loading; final state: {state:?}");
        exit.write(AppExit::error());
    }
}
