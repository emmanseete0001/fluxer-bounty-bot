use std::sync::Arc;

use anyhow::bail;
use bitflags::bitflags;
use fluxer_neptunium::model::id::{
    Id,
    marker::{GuildMarker, RoleMarker, UserMarker},
};

use crate::db::DbManager;

impl DbManager {
    pub async fn list_guild_permissions(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> anyhow::Result<Arc<Vec<GuildPermissionsEntry>>> {
        if let Some(permissions) = self.cached_guild_permissions.get(&guild_id) {
            return Ok(permissions);
        }
        let raw = sqlx::query_as!(
            GuildPermissionsEntrySchema,
            "SELECT * FROM guild_permissions
            WHERE guild_id = $1",
            guild_id.into_inner().cast_signed()
        )
        .fetch_all(&self.pool)
        .await?;
        let permissions = Arc::new(
            raw.into_iter()
                .map(TryInto::try_into)
                .collect::<anyhow::Result<_>>()?,
        );
        self.cached_guild_permissions
            .insert(guild_id, Arc::clone(&permissions));
        Ok(permissions)
    }

    pub async fn set_guild_permissions(
        &self,
        guild_id: Id<GuildMarker>,
        role_id: Id<RoleMarker>,
        permissions: BotPermissions,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            "INSERT INTO guild_permissions (guild_id, kind, entity_id, allow)
            VALUES ($1, 'Role', $2, $3)
            ON CONFLICT (guild_id, entity_id) DO UPDATE
            SET allow = $3",
            guild_id.into_inner().cast_signed(),
            role_id.into_inner().cast_signed(),
            permissions.bits().cast_signed(),
        )
        .execute(&self.pool)
        .await?;
        self.cached_guild_permissions.invalidate(&guild_id);
        sqlx::query!(
            "DELETE FROM guild_permissions
            WHERE allow = 0",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

bitflags! {
    #[derive(Copy, Clone)]
    pub struct BotPermissions: u64 {
        const CREATE_BOUNTIES = 1 << 0;
        const MANAGE_BOUNTIES = 1 << 1;
        const MANAGE_GUILD_CONFIG = 1 << 2;
        const BOUNTY_HUNTER = 1 << 3;
    }
}

#[derive(PartialEq, Eq)]
pub enum GuildPermissionEntity {
    User(Id<UserMarker>),
    Role(Id<RoleMarker>),
}

pub struct GuildPermissionsEntry {
    #[expect(unused)]
    pub guild_id: Id<GuildMarker>,
    pub entity: GuildPermissionEntity,
    pub allow: BotPermissions,
}

struct GuildPermissionsEntrySchema {
    pub guild_id: i64,
    pub kind: String,
    pub entity_id: i64,
    pub allow: i64,
}

impl TryFrom<GuildPermissionsEntrySchema> for GuildPermissionsEntry {
    type Error = anyhow::Error;
    fn try_from(value: GuildPermissionsEntrySchema) -> Result<Self, Self::Error> {
        Ok(Self {
            guild_id: value.guild_id.cast_unsigned().into(),
            entity: match value.kind.to_lowercase().as_str() {
                "user" => GuildPermissionEntity::User(value.entity_id.cast_unsigned().into()),
                "role" => GuildPermissionEntity::Role(value.entity_id.cast_unsigned().into()),
                _ => bail!("Failed to parse value.kind which is \"{}\"", value.kind),
            },
            allow: BotPermissions::from_bits_truncate(value.allow.cast_unsigned()),
        })
    }
}
