use atrium_api::agent::{
    Agent,
    atp_agent::{CredentialSession, store::MemorySessionStore},
};
use atrium_xrpc_client::reqwest::ReqwestClient;
use futures::future::join_all;
use std::sync::Arc;

struct User {
    name: String,
    handle: String,
    shared: Vec<usize>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CredentialSession::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );
    session
        .login("spuds.casa", include_str!("password.txt"))
        .await?;
    let agent = Agent::new(session);
    let actor: atrium_api::types::string::AtIdentifier = "spuds.casa".parse()?;
    let mut follows = agent
        .api
        .app
        .bsky
        .graph
        .get_follows(
            atrium_api::app::bsky::graph::get_follows::ParametersData {
                actor,
                cursor: None,
                limit: Some(100.try_into()?),
            }
            .into(),
        )
        .await?;
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
            tokio::spawn(async move {
                agent
                    .api
                    .app
                    .bsky
                    .graph
                    .get_follows(
                        atrium_api::app::bsky::graph::get_follows::ParametersData {
                            actor,
                            cursor: None,
                            limit: Some(100.try_into().unwrap()),
                        }
                        .into(),
                    )
                    .await
            })
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
    for user in users.iter().skip(1) {
        println!(
            "{} also follows {:#?}",
            &user.name,
            user.shared
                .iter()
                .map(|i| &users[*i].name)
                .collect::<Vec<_>>()
        )
    }
    Ok(())
}
