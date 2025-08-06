use atrium_api::agent::{
    Agent,
    atp_agent::{CredentialSession, store::MemorySessionStore},
};
use atrium_xrpc_client::reqwest::ReqwestClient;
use futures::future::join_all;
use std::sync::Arc;

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
    let follows = agent
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
    for (follow, result) in follows.follows.iter().zip(results) {
        println!(
            "{} also follows {:#?}",
            follow.display_name.clone().unwrap(),
            result??
                .follows
                .iter()
                .filter(|follow| follows.follows.contains(follow))
                .map(|follow| follow.display_name.clone().unwrap())
                .collect::<Vec<_>>()
        )
    }
    Ok(())
}
