use std::sync::Arc;

use anyhow::Context;
use fluxer_neptunium::model::id::{Id, marker::GuildMarker};
use moka::sync::Cache;
use sqlx::PgPool;

use crate::db::schema::{BountySubmissionFormat, GuildConfig, GuildConfigSchema};

#[derive(Clone)]
pub struct DbManager {
    pool: PgPool,
    cached_guild_config: Arc<moka::sync::Cache<Id<GuildMarker>, Arc<GuildConfig>>>,
}

impl DbManager {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(url).await.context("Failed to connect")?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("Failed to run migrations")?;
        Ok(Self {
            pool,
            cached_guild_config: Arc::new(Cache::new(128)),
        })
    }

    pub async fn get_guild_config_upsert(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> anyhow::Result<Arc<GuildConfig>> {
        if let Some(config) = self.cached_guild_config.get(&guild_id) {
            return Ok(config);
        }
        let raw = sqlx::query_as!(
            GuildConfigSchema,
            "INSERT INTO guilds (guild_id, bounty_submission_format)
            VALUES ($1, $2)
            ON CONFLICT (guild_id) DO UPDATE
            SET guild_id = EXCLUDED.guild_id
            RETURNING *",
            guild_id.into_inner().cast_signed(),
            serde_json::to_value(&BountySubmissionFormat {})?,
        )
        .fetch_one(&self.pool)
        .await?;
        let guild_config = Arc::new(raw.try_into()?);
        self.cached_guild_config
            .insert(guild_id, Arc::clone(&guild_config));
        Ok(guild_config)
    }
}

pub mod schema {
    use fluxer_neptunium::model::id::{
        Id,
        marker::{ChannelMarker, GuildMarker},
    };

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct BountySubmissionFormat {}

    pub struct GuildConfig {
        pub guild_id: Id<GuildMarker>,
        pub bounty_submission_channel: Option<Id<ChannelMarker>>,
        pub approval_queue_channel: Option<Id<ChannelMarker>>,
        pub claimed_bounties_channel: Option<Id<ChannelMarker>>,
        pub completed_bounties_channel: Option<Id<ChannelMarker>>,
        pub denied_bounties_channel: Option<Id<ChannelMarker>>,
        pub command_prefixes: Vec<String>,
        pub bounty_submission_format: BountySubmissionFormat,
        pub command_channels: Option<Vec<Id<ChannelMarker>>>,
    }

    impl TryFrom<GuildConfigSchema> for GuildConfig {
        type Error = anyhow::Error;
        fn try_from(value: GuildConfigSchema) -> Result<Self, Self::Error> {
            Ok(Self {
                guild_id: value.guild_id.cast_unsigned().into(),
                bounty_submission_channel: value
                    .bounty_submission_channel
                    .map(|id| id.cast_unsigned().into()),
                approval_queue_channel: value
                    .approval_queue_channel
                    .map(|id| id.cast_unsigned().into()),
                claimed_bounties_channel: value
                    .claimed_bounties_channel
                    .map(|id| id.cast_unsigned().into()),
                completed_bounties_channel: value
                    .completed_bounties_channel
                    .map(|id| id.cast_unsigned().into()),
                denied_bounties_channel: value
                    .denied_bounties_channel
                    .map(|id| id.cast_unsigned().into()),
                command_prefixes: value.command_prefixes,
                bounty_submission_format: serde_json::from_value(value.bounty_submission_format)?,
                command_channels: value.command_channels.map(|command_channels| {
                    command_channels
                        .into_iter()
                        .map(|id| id.cast_unsigned().into())
                        .collect()
                }),
            })
        }
    }

    #[derive(sqlx::FromRow)]
    pub(super) struct GuildConfigSchema {
        pub guild_id: i64,
        pub bounty_submission_channel: Option<i64>,
        pub approval_queue_channel: Option<i64>,
        pub claimed_bounties_channel: Option<i64>,
        pub completed_bounties_channel: Option<i64>,
        pub denied_bounties_channel: Option<i64>,
        pub command_prefixes: Vec<String>,
        pub bounty_submission_format: serde_json::Value,
        pub command_channels: Option<Vec<i64>>,
    }
}
