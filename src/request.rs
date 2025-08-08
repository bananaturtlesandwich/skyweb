use super::*;

use atrium_api::app::bsky::graph::get_follows;

pub struct Request;

impl Plugin for Request {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Game::Login),
            |tokio: ResMut<bevy_tokio_tasks::TokioTasksRuntime>| {
                tokio.spawn_background_task(login);
            },
        )
        .add_systems(OnEnter(Game::Get), place)
        .add_systems(
            OnEnter(Game::Connect),
            |tokio: ResMut<bevy_tokio_tasks::TokioTasksRuntime>,
             profile: Res<Profile>,
             users: Res<Users>| {
                let actor = profile.actor.as_ref();
                for user in users
                    .clone()
                    .into_iter()
                    .filter(|(handle, _)| handle != actor)
                {
                    tokio.spawn_background_task(|ctx| connect(ctx, user));
                }
            },
        );
    }
}

#[derive(Resource, Deref)]
struct Profile {
    actor: atrium_api::types::string::AtIdentifier,
    #[deref]
    profile: atrium_api::app::bsky::actor::defs::ProfileViewDetailedData,
}

static LIMIT: std::sync::OnceLock<Option<atrium_api::types::LimitedNonZeroU8<100>>> =
    std::sync::OnceLock::new();

fn limit() -> Option<atrium_api::types::LimitedNonZeroU8<100>> {
    *LIMIT.get_or_init(|| Some(10.try_into().unwrap()))
}

async fn login(mut ctx: bevy_tokio_tasks::TaskContext) {
    let actor: atrium_api::types::string::AtIdentifier = std::env::args()
        .nth(1)
        .unwrap_or("spuds.casa".into())
        .parse()
        .unwrap();
    let client = atrium_api::client::AtpServiceClient::new(
        atrium_xrpc_client::reqwest::ReqwestClient::new("https://public.api.bsky.app"),
    );
    loop {
        if let Ok(profile) = client
            .service
            .app
            .bsky
            .actor
            .get_profile(
                atrium_api::app::bsky::actor::get_profile::ParametersData {
                    actor: actor.clone(),
                }
                .into(),
            )
            .await
        {
            ctx.run_on_main_thread(|bevy| {
                bevy.world.insert_resource(Profile {
                    actor,
                    profile: profile.data,
                });
                bevy.world.resource_mut::<NextState<Game>>().set(Game::Get)
            })
            .await;
            break;
        }
    }
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

fn place(
    mut commands: Commands,
    tokio: ResMut<bevy_tokio_tasks::TokioTasksRuntime>,
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
        / 2.0;
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
    tokio.spawn_background_task(|ctx| get(ctx, actor));
}

async fn get(
    mut ctx: bevy_tokio_tasks::TaskContext,
    actor: atrium_api::types::string::AtIdentifier,
) {
    let mut cursor = None;
    loop {
        if let Ok(res) = client()
            .service
            .app
            .bsky
            .graph
            .get_follows(
                get_follows::ParametersData {
                    actor: actor.clone(),
                    cursor: cursor.clone(),
                    limit: limit(),
                }
                .into(),
            )
            .await
        {
            // lmao the number of scopes here is crazy
            #[rustfmt::skip]
            ctx.run_on_main_thread(|bevy| {
                bevy.world.resource_scope(|world, mut users: Mut<Users>| {
                    world.resource_scope(|world, mut placement: Mut<Placement>| {
                        world.resource_scope(|world, mut mats: Mut<Assets<ColorMaterial>>| {
                            world.resource_scope(|world, orb: Mut<Orb>| {
                                world.resource_scope(|world, server: Mut<AssetServer>| {
                                    let mut commands = world.commands();
                                    for follow in res.data.follows {
                                        users.insert(
                                            follow.handle.as_str().into(),
                                            commands
                                                .spawn((
                                                    orb.collider.clone(),
                                                    avian2d::prelude::RigidBody::Dynamic,
                                                    Mesh2d(orb.mesh.clone_weak()),
                                                    MeshMaterial2d(mats.add(ColorMaterial::from(
                                                        server.load_with_settings(
                                                            &follow.avatar.clone().unwrap_or_default(),
                                                            |s: &mut bevy::image::ImageLoaderSettings| {
                                                                s.format =
                                                                    bevy::image::ImageFormatSetting::Format(
                                                                        ImageFormat::Jpeg,
                                                                    )
                                                            },
                                                        ),
                                                    ))),
                                                    Transform::from_translation(placement.next()),
                                                ))
                                                .id(),
                                        );
                                    }
                                })
                            })
                        })
                    })
                });
            }).await;
            if res.data.cursor.is_none() {
                break;
            }
            cursor = res.data.cursor.clone();
        }
    }
    // round two for you!
    #[rustfmt::skip]
    ctx.run_on_main_thread(|bevy| {
        bevy.world.resource_scope(|world, mut users: Mut<Users>| {
            world.resource_scope(|world, mut mats: Mut<Assets<ColorMaterial>>| {
                world.resource_scope(|world, profile: Mut<Profile>| {
                    world.resource_scope(|world, orb: Mut<Orb>| {
                        world.resource_scope(|world, server: Mut<AssetServer>| {
                            let mut commands = world.commands();
                            let shared = users.values().cloned().collect();
                            users.insert(
                                profile.handle.to_string(),
                                commands
                                    .spawn((
                                        orb.collider.clone(),
                                        avian2d::prelude::RigidBody::Static,
                                        User {
                                            handle: profile.handle.to_string(),
                                            shared
                                        },
                                        Mesh2d(orb.mesh.clone_weak()),
                                        MeshMaterial2d(mats.add(ColorMaterial::from(
                                            server.load_with_settings(
                                                &profile.avatar.clone().unwrap_or_default(),
                                                |s: &mut bevy::image::ImageLoaderSettings| {
                                                    s.format =
                                                        bevy::image::ImageFormatSetting::Format(
                                                            ImageFormat::Jpeg,
                                                        )
                                                },
                                            ),
                                        ))),
                                    ))
                                    .id(),
                            );
                        })
                    })
                })
            })
        });
    }).await;
    ctx.run_on_main_thread(|bevy| {
        bevy.world
            .resource_mut::<NextState<Game>>()
            .set(Game::Connect)
    })
    .await;
}

async fn connect(mut ctx: bevy_tokio_tasks::TaskContext, (handle, ent): (String, Entity)) {
    let actor: atrium_api::types::string::AtIdentifier = handle.parse().unwrap();
    let mut cursor = None;
    ctx.run_on_main_thread(move |bevy| {
        bevy.world.entity_mut(ent.clone()).insert(User {
            handle,
            shared: Vec::new(),
        });
    })
    .await;
    loop {
        if let Ok(res) = client()
            .service
            .app
            .bsky
            .graph
            .get_follows(
                get_follows::ParametersData {
                    actor: actor.clone(),
                    cursor: cursor.clone(),
                    limit: limit(),
                }
                .into(),
            )
            .await
        {
            ctx.run_on_main_thread(move |bevy| {
                bevy.world.resource_scope(move |world, users: Mut<Users>| {
                    let mut ent = world.entity_mut(ent.clone());
                    let mut user = ent.get_mut::<User>().unwrap();
                    for follow in res.data.follows {
                        if let Some(ent) = users.get(follow.handle.as_str()) {
                            user.shared.push(ent.clone());
                        }
                    }
                })
            })
            .await;
            if res.data.cursor.is_none() {
                break;
            }
            cursor = res.data.cursor.clone();
        }
    }
}
