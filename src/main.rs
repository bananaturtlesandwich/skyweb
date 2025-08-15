use bevy::prelude::*;
#[allow(unused_imports, clippy::single_component_path_imports)]
#[cfg(not(target_family = "wasm"))]
#[cfg(debug_assertions)]
use bevy_dylib;

mod ask;
mod compat;
use compat::*;

fn main() -> AppExit {
    bevy::app::App::new()
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
            bevy_egui::EguiPlugin::default(),
            ask::Stuff,
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

#[derive(Resource, Deref)]
struct Profile {
    actor: atrium_api::types::string::AtIdentifier,
    #[deref]
    profile: atrium_api::app::bsky::actor::defs::ProfileViewDetailedData,
}

#[derive(States, Default, Debug, Eq, PartialEq, Hash, Clone)]
#[states(scoped_entities)]
enum Game {
    #[default]
    Ask,
    Get,
    Connect,
}
