use bevy::prelude::*;
#[allow(unused_imports, clippy::single_component_path_imports)]
#[cfg(debug_assertions)]
use bevy_dylib;

mod game;
mod request;

fn main() -> AppExit {
    bevy::app::App::new()
        .register_type::<Stats>()
        .init_resource::<Stats>()
        .insert_resource(avian2d::prelude::Gravity(Vec2::ZERO))
        .add_plugins((
            bevy_web_asset::WebAssetPlugin,
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "skyweb".into(),
                    ..default()
                }),
                ..default()
            }),
            avian2d::PhysicsPlugins::default(),
            avian2d::picking::PhysicsPickingPlugin,
            bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
            bevy_inspector_egui::quick::ResourceInspectorPlugin::<Stats>::default(),
        ))
        .add_systems(Startup, game::spawn)
        .add_systems(Update, (game::attract, game::web, game::resize))
        .add_systems(Update, request::login.run_if(in_state(Game::Login)))
        .add_observer(game::link)
        .run()
}

const LIMIT: u8 = 100;
const HANDLE: &str = include_str!("handle.txt");
const PASSWORD: &str = include_str!("password.txt");

struct User {
    name: String,
    handle: String,
    avatar: String,
    shared: Vec<usize>,
}

#[derive(Component, Clone)]
struct UserComp {
    handle: String,
    shared: Vec<Entity>,
}

#[derive(Resource, Deref)]
struct Users(Vec<User>);

#[derive(States, Debug, Eq, PartialEq, Hash, Clone)]
enum Game {
    Ask,
    Login,
    Get,
    Connect,
    Attract,
}
