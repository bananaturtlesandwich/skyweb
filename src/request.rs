use super::*;

use atrium_api::agent::Agent;
use atrium_api::app::bsky::graph::get_follows;

type Session = atrium_api::agent::atp_agent::CredentialSession<
    atrium_api::agent::atp_agent::store::MemorySessionStore,
    atrium_xrpc_client::reqwest::ReqwestClient,
>;

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
            |tokio: ResMut<bevy_tokio_tasks::TokioTasksRuntime>, users: Res<Users>| {
                for user in users.clone().into_iter() {
                    tokio.spawn_background_task(|ctx| connect(ctx, user));
                }
            },
        );
    }
}

struct Bsky {
    actor: atrium_api::types::string::AtIdentifier,
    // todo: remove radius and just allow moving the camera
    follows: usize,
    agent: Agent<Session>,
}

static BSKY: std::sync::OnceLock<Bsky> = std::sync::OnceLock::new();

fn bsky() -> &'static Bsky {
    BSKY.get().unwrap()
}

async fn login(mut ctx: bevy_tokio_tasks::TaskContext) {
    let session = Session::new(
        atrium_xrpc_client::reqwest::ReqwestClient::new("https://bsky.social"),
        atrium_api::agent::atp_agent::store::MemorySessionStore::default(),
    );
    let actor: atrium_api::types::string::AtIdentifier = std::env::args()
        .nth(1)
        .unwrap_or(HANDLE.into())
        .parse()
        .unwrap();
    loop {
        if session.login(HANDLE, PASSWORD).await.is_ok() {
            break;
        }
    }
    let agent = Agent::new(session);
    loop {
        if let Ok(profile) = agent
            .api
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
            BSKY.get_or_init(|| Bsky {
                actor: std::env::args()
                    .nth(1)
                    .unwrap_or(HANDLE.into())
                    .parse()
                    .unwrap(),
                follows: profile.follows_count.unwrap() as usize,
                agent,
            });
            ctx.run_on_main_thread(|bevy| {
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
        let pos = self.pos;
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
        pos
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
    window: Single<&Window>,
) {
    use avian2d::prelude::*;
    commands.spawn(Camera2d);
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
    let radius = (width * height / bsky().follows as f32 / std::f32::consts::PI).sqrt() / 2.0;
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
    tokio.spawn_background_task(get);
}

async fn get(mut ctx: bevy_tokio_tasks::TaskContext) {
    let bsky = bsky();
    let mut cursor = None;
    while let Ok(res) = bsky
        .agent
        .api
        .app
        .bsky
        .graph
        .get_follows(
            get_follows::ParametersData {
                actor: bsky.actor.clone(),
                cursor,
                limit: Some(LIMIT.try_into().unwrap()),
            }
            .into(),
        )
        .await
        && res.cursor.is_some()
    {
        cursor = res.cursor.clone();
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
    }
    ctx.run_on_main_thread(|bevy| {
        bevy.world
            .resource_mut::<NextState<Game>>()
            .set(Game::Connect)
    })
    .await;
}

async fn connect(mut ctx: bevy_tokio_tasks::TaskContext, (handle, ent): (String, Entity)) {
    let actor: atrium_api::types::string::AtIdentifier = handle.parse().unwrap();
    let bsky = bsky();
    let mut cursor = None;
    ctx.run_on_main_thread(move |bevy| {
        bevy.world.entity_mut(ent.clone()).insert(User {
            handle,
            shared: Vec::new(),
        });
    })
    .await;
    while let Ok(res) = bsky
        .agent
        .api
        .app
        .bsky
        .graph
        .get_follows(
            get_follows::ParametersData {
                actor: actor.clone(),
                cursor,
                limit: Some(LIMIT.try_into().unwrap()),
            }
            .into(),
        )
        .await
        && res.cursor.is_some()
    {
        cursor = res.cursor.clone();
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
    }
}
