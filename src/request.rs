use super::*;

use atrium_api::agent::Agent;
use atrium_api::app::bsky::graph::get_follows;

type Session = atrium_api::agent::atp_agent::CredentialSession<
    atrium_api::agent::atp_agent::store::MemorySessionStore,
    atrium_xrpc_client::reqwest::ReqwestClient,
>;

pub struct Request;

impl Plugin for Request {}

struct Bsky {
    actor: atrium_api::types::string::AtIdentifier,
    agent: Agent<Session>,
}

static BSKY: std::sync::OnceLock<Bsky> = std::sync::OnceLock::new();

fn bsky() -> &'static Bsky {
    BSKY.get().unwrap()
}

// todo: don't block on this
pub fn login(mut next: ResMut<NextState<Game>>, mut session: Local<Option<Session>>) {
    if bevy::tasks::block_on(async {
        session
            .get_or_insert_with(|| {
                Session::new(
                    atrium_xrpc_client::reqwest::ReqwestClient::new("https://bsky.social"),
                    atrium_api::agent::atp_agent::store::MemorySessionStore::default(),
                )
            })
            .login(HANDLE, PASSWORD)
            .await
    })
    .is_ok()
        // always true by this point
        && let Some(session) = session.take()
    {
        BSKY.get_or_init(|| Bsky {
            actor: std::env::args()
                .nth(1)
                .unwrap_or(HANDLE.into())
                .parse()
                .unwrap(),
            agent: Agent::new(session),
        });
        next.set(Game::Get);
    }
}

#[derive(Resource, Deref, DerefMut)]
struct Get(
    Vec<bevy::tasks::Task<atrium_api::xrpc::Result<get_follows::Output, get_follows::Error>>>,
);

pub fn begin_get(mut commands: Commands) {
    let pool = bevy::tasks::IoTaskPool::get();
    let bsky = bsky();
    let mut tasks = Get(Vec::new());
    tasks.push(pool.spawn(async {
        bsky.agent
            .api
            .app
            .bsky
            .graph
            .get_follows(
                get_follows::ParametersData {
                    actor: bsky.actor.clone(),
                    cursor: None,
                    limit: Some(LIMIT.try_into().unwrap()),
                }
                .into(),
            )
            .await
    }));
    commands.insert_resource(tasks);
}

pub fn get(commands: Commands, mut tasks: ResMut<Get>, mut next: ResMut<NextState<Game>>) {
    let pool = bevy::tasks::IoTaskPool::get();
    let bsky = bsky();
    for i in (0..tasks.len()).rev() {
        if tasks[i].is_finished() {
            match bevy::tasks::block_on(tasks.remove(i).cancel()) {
                Some(Ok(res)) => {
                    for follow in &res.follows {
                        // todo: spawn
                    }
                }
                _ => {
                    tasks.push(pool.spawn(async {
                        bsky.agent
                            .api
                            .app
                            .bsky
                            .graph
                            .get_follows(
                                get_follows::ParametersData {
                                    actor: bsky.actor.clone(),
                                    cursor: None,
                                    limit: Some(LIMIT.try_into().unwrap()),
                                }
                                .into(),
                            )
                            .await
                    }));
                }
            }
        }
    }
    if tasks.is_empty() {
        next.set(Game::Connect)
    }
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
