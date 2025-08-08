use bevy::prelude::*;
#[allow(unused_imports, clippy::single_component_path_imports)]
#[cfg(debug_assertions)]
use bevy_dylib;

mod ask;
mod bsky;
mod config;
mod connect;

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
                    file_path: "./".into(),
                    ..default()
                }),
            avian2d::PhysicsPlugins::default(),
            avian2d::picking::PhysicsPickingPlugin,
            bevy_tokio_tasks::TokioTasksPlugin::default(),
            bevy_simple_text_input::TextInputPlugin,
            ask::Stuff,
            bsky::Stuff,
            connect::Stuff,
            config::Stuff,
        ))
        .init_resource::<Grape>()
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

#[derive(Resource, Reflect)]
struct Config {
    attraction: f32,
    repulsion: f32,
    gravity: f32,
    tick: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            attraction: 400.0,
            repulsion: 300.0,
            gravity: 0.5,
            tick: 0.5,
        }
    }
}

#[derive(Resource, Deref)]
struct Grape(Handle<Font>);

impl FromWorld for Grape {
    fn from_world(world: &mut World) -> Self {
        Self(world.resource::<AssetServer>().load("grapesoda.ttf"))
    }
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
