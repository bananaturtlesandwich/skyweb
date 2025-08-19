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

#[derive(Resource, Deref)]
struct Orb(Handle<Mesh>);

#[derive(Component)]
struct Follow {
    actor: atrium_api::types::string::AtIdentifier,
    cursor: Option<String>,
    task: bevy::tasks::Task<atrium_api::xrpc::Result<get_follows::Output, get_follows::Error>>,
}

#[derive(Resource, Deref, DerefMut)]
struct You(Follow);

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    profile: Res<Profile>,
    window: Single<&Window>,
) {
    let width = window.width();
    let height = window.height();
    let radius = (width * height / profile.follows_count.unwrap() as f32 / std::f32::consts::PI)
        .sqrt()
        / 2.5;
    commands.insert_resource(Orb(meshes.add(Circle::new(radius))));
    commands.init_resource::<Users>();
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
    mut users: ResMut<Users>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    mut next: ResMut<NextState<Game>>,
) {
    let data = match bevy::tasks::block_on(bevy::tasks::poll_once(&mut you.task)) {
        Some(Ok(atrium_api::types::Object { data, .. })) => {
            let pool = bevy::tasks::IoTaskPool::get();
            for follow in &data.follows {
                let actor: atrium_api::types::string::AtIdentifier = follow.handle.parse().unwrap();
                let index = users.len();
                users.insert(
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
    let shared = users.values().cloned().collect();
    let index = users.len();
    users.insert(
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
    next.set(Game::Connect)
}

fn connect(
    parallel: ParallelCommands,
    users: Res<Users>,
    mut user: Query<(Entity, &mut User, &mut Follow)>,
) {
    user.par_iter_mut().for_each(|(ent, mut user, mut follow)| {
        match bevy::tasks::block_on(bevy::tasks::poll_once(&mut follow.task)) {
            Some(Ok(atrium_api::types::Object { data, .. })) => {
                for follow in data.follows {
                    if let Some(ent) = users.get(follow.handle.as_str()) {
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
                parallel.command_scope(|mut commands| {
                    commands.entity(ent).remove::<Follow>();
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
    });
}
