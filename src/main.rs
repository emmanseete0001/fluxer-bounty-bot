use anyhow::Context;
use fluxer_neptunium::{
    client::{Client, ClientConfig},
    http::endpoints::channel::AllowedMentions,
};

use crate::{db::DbManager, event_handler::Handler};

mod db;
mod event_handler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    if let Err(e) = dotenvy::dotenv() {
        tracing::warn!(".env file not found: {e}");
    }

    let database_url =
        dotenvy::var("DATABASE_URL").context("Failed to load DATABASE_URL from env")?;
    let token = dotenvy::var("TOKEN").context("Failed to load TOKEN from env")?;
    let client_id = serde_json::from_str(
        &dotenvy::var("CLIENT_ID").context("Failed to load CLIENT_ID from env")?,
    )
    .context("Failed to parse CLIENT_ID")?;

    let db = DbManager::new(&database_url)
        .await
        .context("Failed to create database manager")?;

    let mut client = Client::new_with_config(
        token,
        ClientConfig::builder()
            .default_allowed_mentions(AllowedMentions {
                parse: Some(Vec::new()),
                users: Some(Vec::new()),
                roles: Some(Vec::new()),
                replied_user: false,
            })
            .build(),
    );
    client.register_event_handler(Handler::new(db, client_id));

    client.start().await.context("Fatal client error")?;

    Ok(())
}
