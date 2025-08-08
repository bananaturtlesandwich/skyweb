use bevy::prelude::*;
#[allow(unused_imports, clippy::single_component_path_imports)]
#[cfg(debug_assertions)]
use bevy_dylib;

mod ask;
mod attraction;
mod request;

fn main() -> AppExit {
    bevy::app::App::new()
        .insert_resource(avian2d::prelude::Gravity(Vec2::ZERO))
        .add_plugins((
            bevy_web_asset::WebAssetPlugin,
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "skyweb".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: "".into(),
                    ..default()
                }),
            avian2d::PhysicsPlugins::default(),
            avian2d::picking::PhysicsPickingPlugin,
            bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
            bevy_tokio_tasks::TokioTasksPlugin::default(),
            ask::Ask,
            request::Request,
            attraction::Attraction,
        ))
        .init_state::<Game>()
        .run()
}

static CLIENT: std::sync::OnceLock<
    atrium_api::client::AtpServiceClient<atrium_xrpc_client::reqwest::ReqwestClient>,
> = std::sync::OnceLock::new();

fn client()
-> &'static atrium_api::client::AtpServiceClient<atrium_xrpc_client::reqwest::ReqwestClient> {
    CLIENT.get_or_init(|| {
        atrium_api::client::AtpServiceClient::new(atrium_xrpc_client::reqwest::ReqwestClient::new(
            "https://public.api.bsky.app",
        ))
    })
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
}

#[derive(Resource, Deref, DerefMut, Default)]
struct Users(std::collections::BTreeMap<String, Entity>);

#[derive(States, Default, Debug, Eq, PartialEq, Hash, Clone)]
#[states(scoped_entities)]
enum Game {
    #[default]
    Ask,
    Get,
    Connect,
}
