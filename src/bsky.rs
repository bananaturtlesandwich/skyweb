use super::*;

use atrium_api::app::bsky::graph::get_follows;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Game::Get), spawn).add_systems(
            Update,
            (
                get.run_if(in_state(Game::Get)),
                connect.run_if(in_state(Game::Connect)),
            ),
        );
    }
}

static LIMIT: std::sync::OnceLock<Option<atrium_api::types::LimitedNonZeroU8<100>>> =
    std::sync::OnceLock::new();

fn limit() -> Option<atrium_api::types::LimitedNonZeroU8<100>> {
    *LIMIT.get_or_init(|| Some(10.try_into().unwrap()))
}

#[derive(Component)]
struct Follow {
    actor: atrium_api::types::string::AtIdentifier,
    cursor: Option<String>,
    task: bevy::tasks::Task<atrium_api::xrpc::Result<get_follows::Output, get_follows::Error>>,
}

#[derive(Resource, Deref, DerefMut)]
struct You(Follow);

fn spawn(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, profile: Res<Profile>) {
    commands.insert_resource(Orb(meshes.add(Circle::new(6.0))));
    commands.init_resource::<Network>();
    let actor = profile.actor.clone();
    commands.insert_resource(You(Follow {
        actor: actor.clone(),
        cursor: None,
        task: bevy::tasks::IoTaskPool::get().spawn(Compat::new(
            CLIENT.service.app.bsky.graph.get_follows(
                get_follows::ParametersData {
                    actor: actor.clone(),
                    cursor: None,
                    limit: limit(),
                }
                .into(),
            ),
        )),
    }));
}

fn get(
    mut commands: Commands,
    orb: Res<Orb>,
    server: Res<AssetServer>,
    mut you: ResMut<You>,
    mut network: ResMut<Network>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    mut next: ResMut<NextState<Game>>,
) {
    let data = match bevy::tasks::block_on(bevy::tasks::poll_once(&mut you.task)) {
        Some(Ok(atrium_api::types::Object { data, .. })) => {
            let pool = bevy::tasks::IoTaskPool::get();
            for follow in &data.follows {
                let actor: atrium_api::types::string::AtIdentifier = follow.handle.parse().unwrap();
                let index = network.len();
                network.insert(
                    follow.handle.as_str().into(),
                    commands
                        .spawn((
                            Mesh2d(orb.clone_weak()),
                            User {
                                handle: follow.handle.to_string(),
                                shared: Vec::new(),
                                index,
                            },
                            Follow {
                                actor: actor.clone(),
                                cursor: None,
                                task: pool.spawn(Compat::new(
                                    CLIENT.service.app.bsky.graph.get_follows(
                                        get_follows::ParametersData {
                                            actor,
                                            cursor: None,
                                            limit: limit(),
                                        }
                                        .into(),
                                    ),
                                )),
                            },
                            MeshMaterial2d(mats.add(ColorMaterial::from(
                                server.load_with_settings(
                                    follow.avatar.clone().unwrap_or_default(),
                                    |s: &mut bevy::image::ImageLoaderSettings| {
                                        s.format = bevy::image::ImageFormatSetting::Guess
                                    },
                                ),
                            ))),
                            // Transform::from_translation(placement.next()),
                        ))
                        .id(),
                );
            }
            if data.cursor.is_some() {
                you.cursor = data.cursor;
                // duplicated code :/
                you.task = bevy::tasks::IoTaskPool::get().spawn(Compat::new(
                    CLIENT.service.app.bsky.graph.get_follows(
                        get_follows::ParametersData {
                            actor: you.actor.clone(),
                            cursor: you.cursor.clone(),
                            limit: limit(),
                        }
                        .into(),
                    ),
                ));
                return;
            }
            data
        }
        Some(Err(_)) => {
            // duplicated code :/
            you.task = bevy::tasks::IoTaskPool::get().spawn(Compat::new(
                CLIENT.service.app.bsky.graph.get_follows(
                    get_follows::ParametersData {
                        actor: you.actor.clone(),
                        cursor: you.cursor.clone(),
                        limit: limit(),
                    }
                    .into(),
                ),
            ));
            return;
        }
        None => return,
    };
    let shared: Vec<_> = network.values().cloned().collect();
    let index = network.len();
    network.insert(
        data.subject.handle.to_string(),
        commands
            .spawn((
                User {
                    handle: data.subject.handle.to_string(),
                    shared,
                    index,
                },
                Mesh2d(orb.clone_weak()),
                MeshMaterial2d(mats.add(ColorMaterial::from(server.load_with_settings(
                    data.subject.avatar.clone().unwrap_or_default(),
                    |s: &mut bevy::image::ImageLoaderSettings| {
                        s.format = bevy::image::ImageFormatSetting::Guess
                    },
                )))),
            ))
            .id(),
    );
    commands.remove_resource::<You>();
    next.set(Game::Connect)
}

fn connect(
    mut commands: Commands,
    mut network: ResMut<Network>,
    mut users: Query<(Entity, &mut User, &mut Follow)>,
) {
    for (ent, mut user, mut follow) in &mut users {
        match bevy::tasks::block_on(bevy::tasks::poll_once(&mut follow.task)) {
            Some(Ok(atrium_api::types::Object { data, .. })) => {
                for follow in data.follows {
                    if let Some(ent) = network.get(follow.handle.as_str()) {
                        user.shared.push(*ent);
                    }
                }
                if data.cursor.is_some() {
                    follow.cursor = data.cursor;
                    // duplicated code :/
                    follow.task = bevy::tasks::IoTaskPool::get().spawn(Compat::new(
                        CLIENT.service.app.bsky.graph.get_follows(
                            get_follows::ParametersData {
                                actor: follow.actor.clone(),
                                cursor: follow.cursor.clone(),
                                limit: limit(),
                            }
                            .into(),
                        ),
                    ));
                    return;
                }
                network.max = user.shared.len().max(network.max);
                commands.entity(ent).remove::<Follow>();
                commands.queue(move |world: &mut World| {
                    world.resource_scope(|world, mut sim: Mut<Sim>| {
                        let user = world.entity(ent).get::<User>().unwrap();
                        sim.links.extend(user.shared.iter().filter_map(|ent| {
                            Some((user.index, world.entity(*ent).get::<User>().unwrap().index))
                        }))
                    });
                    world.trigger(Rebuild);
                });
            }
            Some(Err(_)) => {
                // duplicated code :/
                follow.task = bevy::tasks::IoTaskPool::get().spawn(Compat::new(
                    CLIENT.service.app.bsky.graph.get_follows(
                        get_follows::ParametersData {
                            actor: follow.actor.clone(),
                            cursor: follow.cursor.clone(),
                            limit: limit(),
                        }
                        .into(),
                    ),
                ));
            }
            None => {}
        }
    }
}
