use bevy::prelude::*;
#[allow(unused_imports, clippy::single_component_path_imports)]
#[cfg(not(target_family = "wasm"))]
#[cfg(debug_assertions)]
use bevy_dylib;

mod ask;
mod avatar;
mod bsky;
mod compat;
use compat::*;
mod config;
mod connect;

fn main() -> AppExit {
    bevy::app::App::new()
        .register_asset_source(
            "https",
            bevy::asset::io::AssetSource::build().with_reader(|| Box::new(avatar::AvatarReader)),
        )
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "skyweb".into(),
                        canvas: Some("#bevy".into()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: "./".into(),
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                }),
            MeshPickingPlugin,
            bevy_egui::EguiPlugin::default(),
            ask::Stuff,
            bsky::Stuff,
            connect::Stuff,
            config::Stuff,
        ))
        .init_state::<Game>()
        .add_systems(
            Startup,
            (compat::alive, |mut commands: Commands| {
                commands.spawn(Camera2d);
            }),
        )
        .run()
}

static CLIENT: std::sync::LazyLock<
    atrium_api::client::AtpServiceClient<atrium_xrpc_client::reqwest::ReqwestClient>,
> = std::sync::LazyLock::new(|| {
    atrium_api::client::AtpServiceClient::new(atrium_xrpc_client::reqwest::ReqwestClient::new(
        "https://public.api.bsky.app",
    ))
});

#[derive(Resource, Reflect)]
struct Config {
    iter: usize,
    charge: f64,
    distance: f64,
    link: Option<f64>,
    centre: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            iter: 5,
            charge: -30.0,
            distance: 30.0,
            link: None,
            centre: 1.0,
        }
    }
}

#[derive(Event)]
struct Rebuild;

#[derive(Component)]
struct Lines;

#[derive(Resource, Deref, DerefMut)]
struct Sim {
    #[deref]
    sim: fjadra::Simulation,
    nodes: Vec<fjadra::Node>,
    links: Vec<(usize, usize)>,
}

#[derive(Resource, Deref)]
struct Profile {
    actor: atrium_api::types::string::AtIdentifier,
    #[deref]
    profile: atrium_api::app::bsky::actor::defs::ProfileViewDetailedData,
}

#[derive(Component)]
struct User {
    handle: String,
    shared: Vec<Entity>,
    index: usize,
}

#[derive(Resource, Deref, DerefMut, Default)]
struct Network(std::collections::BTreeMap<String, Entity>);

#[derive(States, Default, Debug, Eq, PartialEq, Hash, Clone)]
#[states(scoped_entities)]
enum Game {
    #[default]
    Ask,
    Get,
    Connect,
}
