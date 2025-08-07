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
            Update,
            (
                login.run_if(in_state(Game::Login)),
                get.run_if(in_state(Game::Get)),
            ),
        )
        .add_systems(OnEnter(Game::Get), place);
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

// todo: don't block on this
pub fn login(
    mut next: ResMut<NextState<Game>>,
    mut session: Local<Option<Session>>,
    mut agent: Local<Option<Agent<Session>>>,
) {
    let actor: atrium_api::types::string::AtIdentifier = std::env::args()
        .nth(1)
        .unwrap_or(HANDLE.into())
        .parse()
        .unwrap();
    let agentref = match agent.as_ref() {
        Some(agent) => agent,
        None => {
            if bevy::tasks::block_on(session
                .get_or_insert_with(|| {
                    Session::new(
                        atrium_xrpc_client::reqwest::ReqwestClient::new("https://bsky.social"),
                        atrium_api::agent::atp_agent::store::MemorySessionStore::default(),
                    )
                })
                .login(HANDLE, PASSWORD)
        )
        .is_ok()
            // always true by this point
            && let Some(session) = session.take()
            {
                let _ = agent.insert(Agent::new(session));
                agent.as_ref().unwrap()
            } else {
                return;
            }
        }
    };
    if let Ok(profile) = bevy::tasks::block_on(
        agentref.api.app.bsky.actor.get_profile(atrium_api::app::bsky::actor::get_profile::ParametersData{
            actor: actor.clone(),
        }.into())
    )
        // always true by this point
        && let Some(agent) = agent.take()
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
        next.set(Game::Get);
    }
}

#[derive(Resource)]
struct Get {
    cursor: Option<String>,
    task: Option<
        bevy::tasks::Task<atrium_api::xrpc::Result<get_follows::Output, get_follows::Error>>,
    >,
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

fn place(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, window: Single<&Window>) {
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
}

fn get(
    mut commands: Commands,
    orb: Res<Orb>,
    server: Res<AssetServer>,
    mut tasks: ResMut<Get>,
    mut users: ResMut<Users>,
    mut placement: ResMut<Placement>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    mut next: ResMut<NextState<Game>>,
) {
    let pool = bevy::tasks::IoTaskPool::get();
    let bsky = bsky();
    if tasks.task.as_ref().is_some_and(|task| task.is_finished())
        && let Some(Ok(res)) = bevy::tasks::block_on(tasks.task.take().unwrap().cancel())
    {
        for follow in &res.follows {
            users.insert(
                follow.did.as_str().into(),
                commands
                    .spawn((
                        orb.collider.clone(),
                        avian2d::prelude::RigidBody::Dynamic,
                        Mesh2d(orb.mesh.clone_weak()),
                        MeshMaterial2d(mats.add(ColorMaterial::from(server.load_with_settings(
                            &follow.avatar.clone().unwrap_or_default(),
                            |s: &mut bevy::image::ImageLoaderSettings| {
                                s.format =
                                    bevy::image::ImageFormatSetting::Format(ImageFormat::Jpeg)
                            },
                        )))),
                        Transform::from_translation(placement.next()),
                    ))
                    .id(),
            );
        }
        match &res.cursor {
            Some(cursor) => tasks.cursor = Some(cursor.clone()),
            None => {
                commands.remove_resource::<Get>();
                next.set(Game::Connect);
                return;
            }
        }
    }
    let cursor = tasks.cursor.clone();
    tasks.task = Some(
        pool.spawn(
            bsky.agent.api.app.bsky.graph.get_follows(
                get_follows::ParametersData {
                    actor: bsky.actor.clone(),
                    cursor,
                    limit: Some(LIMIT.try_into().unwrap()),
                }
                .into(),
            ),
        ),
    )
}

/*
pub async fn old() -> Result<(), Box<dyn std::error::Error>> {
    let actor: atrium_api::types::string::AtIdentifier =
        std::env::args().nth(1).unwrap_or(HANDLE.into()).parse()?;
    let mut follows = agent
        .api
        .app
        .bsky
        .graph
        .get_follows(
            atrium_api::app::bsky::graph::get_follows::ParametersData {
                actor: actor.clone(),
                cursor: None,
                limit: Some(LIMIT.try_into().unwrap()),
            }
            .into(),
        )
        .await?;
    while follows.cursor.is_some() {
        let cursor = follows.cursor.clone();
        follows.follows.extend(
            agent
                .api
                .app
                .bsky
                .graph
                .get_follows(
                    atrium_api::app::bsky::graph::get_follows::ParametersData {
                        actor: actor.clone(),
                        cursor,
                        limit: Some(LIMIT.try_into().unwrap()),
                    }
                    .into(),
                )
                .await?
                .data
                .follows,
        );
    }
    // show your mutuals
    // currently does two requests for your followers :p
    let sub = follows.subject.clone();
    follows.follows.insert(0, sub);
    let mut users = Vec::with_capacity(follows.follows.len());
    let agent = Arc::new(agent);
    let handles = follows
        .follows
        .iter()
        .map(|actor| {
            let agent = Arc::clone(&agent);
            let actor = actor.handle.parse().unwrap();
            agent.api.app.bsky.graph.get_follows(
                atrium_api::app::bsky::graph::get_follows::ParametersData {
                    actor,
                    cursor: None,
                    limit: Some(LIMIT.try_into().unwrap()),
                }
                .into(),
            )
        })
        .collect::<Vec<_>>();
    let results = join_all(handles).await;
    std::thread::scope(|sc| {
        let threads = results
            .into_iter()
            .map(|result| -> std::thread::ScopedJoinHandle<_> {
                sc.spawn(|| {
                    let result = result.unwrap().unwrap();
                    User {
                        name: result.subject.display_name.clone().unwrap(),
                        handle: result.subject.handle.to_string(),
                        avatar: result.subject.avatar.clone().unwrap(),
                        shared: follows
                            .follows
                            .iter()
                            .enumerate()
                            .filter_map(|(i, follow)| result.follows.contains(follow).then_some(i))
                            .collect(),
                    }
                })
            });
        for thread in threads {
            users.push(thread.join().unwrap());
        }
    });
    users.sort_unstable_by(|user1, user2| user2.shared.len().cmp(&user1.shared.len()));
    Ok(())
}
*/
