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

#[derive(Resource)]
struct Placement {
    radius: f32,
    pos: Vec3,
    layer: u8,
    capacity: u8,
    angle: f32,
    counter: u8,
}

impl Placement {
    fn next(&mut self) -> Vec3 {
        self.counter += 1;
        self.pos = Quat::from_rotation_z(self.angle) * self.pos;
        if self.counter == self.capacity {
            self.counter = 0;
            self.layer += 1;
            // circumference of layer circle is 2*radius*layer*pi
            // orb capacity in each layer is circumference/radius = 2*layer*pi
            self.capacity = (2.0 * std::f32::consts::PI * self.layer as f32).floor() as u8;
            // angle to rotate by is 2*pi/capacity = 2*pi / 2*pi*layer = 1/layer
            self.angle = 1.0 / self.layer as f32;
            self.pos += Vec3::Y * self.radius * 2.5;
        }
        self.pos
    }
}

#[derive(Resource)]
struct Orb {
    mesh: Handle<Mesh>,
    collider: avian2d::prelude::Collider,
}

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
    use avian2d::prelude::*;
    let width = window.width();
    let height = window.height();
    // don't want our orbs escaping containment
    commands.spawn((
        Collider::half_space(Vec2::Y),
        RigidBody::Static,
        Transform::from_translation(Vec3::NEG_Y * height / 2.0),
    ));
    commands.spawn((
        Collider::half_space(Vec2::NEG_Y),
        RigidBody::Static,
        Transform::from_translation(Vec3::Y * height / 2.0),
    ));
    commands.spawn((
        Collider::half_space(Vec2::X),
        RigidBody::Static,
        Transform::from_translation(Vec3::NEG_X * width / 2.0),
    ));
    commands.spawn((
        Collider::half_space(Vec2::NEG_X),
        RigidBody::Static,
        Transform::from_translation(Vec3::X * width / 2.0),
    ));
    let radius = (width * height / profile.follows_count.unwrap() as f32 / std::f32::consts::PI)
        .sqrt()
        / 2.5;
    commands.insert_resource(Placement {
        radius,
        pos: Vec3::ZERO,
        layer: 0,
        capacity: 1,
        angle: 0.0,
        counter: 0,
    });
    commands.insert_resource(Orb {
        mesh: meshes.add(Circle::new(radius)),
        collider: Collider::circle(radius),
    });
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
    mut placement: ResMut<Placement>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    mut next: ResMut<NextState<Game>>,
) {
    let data = match bevy::tasks::block_on(bevy::tasks::poll_once(&mut you.task)) {
        Some(Ok(atrium_api::types::Object { data, .. })) => {
            let pool = bevy::tasks::IoTaskPool::get();
            for follow in &data.follows {
                let actor: atrium_api::types::string::AtIdentifier = follow.handle.parse().unwrap();
                users.insert(
                    follow.handle.as_str().into(),
                    commands
                        .spawn((
                            orb.collider.clone(),
                            avian2d::prelude::RigidBody::Dynamic,
                            Mesh2d(orb.mesh.clone_weak()),
                            User {
                                handle: follow.handle.to_string(),
                                shared: Vec::new(),
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
                                    &follow.avatar.clone().unwrap_or_default(),
                                    |s: &mut bevy::image::ImageLoaderSettings| {
                                        s.format = bevy::image::ImageFormatSetting::Guess
                                    },
                                ),
                            ))),
                            Transform::from_translation(placement.next()),
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
    users.insert(
        data.subject.handle.to_string(),
        commands
            .spawn((
                orb.collider.clone(),
                avian2d::prelude::RigidBody::Static,
                User {
                    handle: data.subject.handle.to_string(),
                    shared,
                },
                Mesh2d(orb.mesh.clone_weak()),
                MeshMaterial2d(mats.add(ColorMaterial::from(server.load_with_settings(
                    &data.subject.avatar.clone().unwrap_or_default(),
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
                        user.shared.push(ent.clone());
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
