use std::sync::Arc;

use anyhow::Context as _;
use chrono::{DateTime, Utc};
use fluxer_neptunium::{
    cached_payload::{CachedMessageCreate, CachedReady},
    create_embed,
    model::id::{Id, marker::UserMarker},
    prelude::*,
};

use crate::db::DbManager;

pub struct Handler {
    db: DbManager,
    client_id: Id<UserMarker>,
}

impl Handler {
    pub fn new(db: DbManager, client_id: Id<UserMarker>) -> Self {
        Self { db, client_id }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn on_ready(&self, _ctx: Context, ready: Arc<CachedReady>) -> Result<(), EventError> {
        tracing::info!(
            "Logged in as {}#{}",
            ready.user.username,
            ready.user.discriminator
        );
        Ok(())
    }

    async fn on_message_create(
        &self,
        ctx: Context,
        message: Arc<CachedMessageCreate>,
    ) -> Result<(), EventError> {
        if message.author.bot {
            return Ok(());
        }
        let Some(guild_id) = message.channel_id.get(&ctx).await?.guild_id else {
            return Ok(());
        };

        let guild_config = match self
            .db
            .get_guild_config_upsert(guild_id)
            .await
            .with_context(|| format!("Failed to get guild config for {guild_id}"))
        {
            Ok(guild_config) => guild_config,
            Err(e) => {
                tracing::error!("{e}");
                return Ok(());
            }
        };

        if let Some(command_channels) = &guild_config.command_channels
            && !command_channels.contains(&message.channel_id)
        {
            return Ok(());
        }

        let content = message.content.trim();

        let mention_prefix = format!("<@{}>", self.client_id);

        let full_command = if let Some(full_command) = content.strip_prefix(&mention_prefix) {
            full_command
        } else {
            let Some(full_command) = guild_config
                .command_prefixes
                .iter()
                .find_map(|prefix| content.strip_prefix(prefix))
            else {
                return Ok(());
            };
            full_command
        };

        let (command, args) = full_command.split_once(' ').unwrap_or((full_command, ""));

        match command {
            "ping" => {
                let latency = {
                    let now = Utc::now();
                    let created_at: DateTime<Utc> = message.timestamp.into();
                    now.signed_duration_since(created_at)
                };
                message
                    .reply(
                        &ctx,
                        create_embed!(
                            title: "Pong!",
                            description: format!("**Latency:** {} ms", latency.num_milliseconds()),
                        ),
                    )
                    .await?;
            }
            _ => return Ok(()),
        }

        Ok(())
    }
}
