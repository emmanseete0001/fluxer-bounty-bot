use fluxer_neptunium::{
    create_embed,
    exts::MessageExt,
    model::id::{
        Id,
        marker::{GuildMarker, RoleMarker},
    },
};

use crate::{
    colors::{DEFAULT, FAILURE, SUCCESS},
    commands::CommandContext,
    db::guild_permissions::{BotPermissions, GuildPermissionEntity},
};

pub async fn add_permission_to(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let (role_id, permission_str) = args.split_once(' ').unwrap_or((args, ""));
    let Some(role_id) = parse_role_id_or_mention(role_id, ctx.guild_id) else {
        ctx.reply(create_embed!(
            description: "Could not parse the role ID or role mention.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };
    let Some(permission) = parse_permission_str(permission_str.trim()) else {
        ctx.message
            .reply(
                ctx.ctx,
                create_embed!(
                    description: "Could not parse the permission string.",
                    color: FAILURE,
                ),
            )
            .await?;
        return Ok(());
    };

    let permissions = ctx.db.list_guild_permissions(ctx.guild_id).await?;
    let existing_permission = permissions
        .iter()
        .find(|entry| entry.entity == GuildPermissionEntity::Role(role_id));
    let permission = if let Some(existing_permission) = existing_permission {
        existing_permission.allow.union(permission)
    } else {
        permission
    };

    ctx.db
        .set_guild_permissions(ctx.guild_id, role_id, permission)
        .await?;

    ctx.reply(create_embed!(
        description: format!("Permissions for <@&{role_id}> updated."),
        color: SUCCESS,
    ))
    .await?;

    Ok(())
}

pub async fn list_permissions(ctx: CommandContext<'_>, _args: &str) -> anyhow::Result<()> {
    let permissions = ctx.db.list_guild_permissions(ctx.guild_id).await?;
    let permissions_string = permissions
        .iter()
        .map(|entry| {
            format!(
                "<@{}> - {} ({})",
                match entry.entity {
                    GuildPermissionEntity::Role(role_id) => format!("&{role_id}"),
                    GuildPermissionEntity::User(user_id) => user_id.to_string(),
                },
                permissions_to_string(entry.allow),
                entry.allow.bits()
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    ctx.reply(create_embed!(
        description: if permissions_string.is_empty() {
            "*none*".to_owned()
        } else { permissions_string },
        color: DEFAULT,
    ))
    .await?;
    Ok(())
}

pub async fn remove_permission_from(ctx: CommandContext<'_>, args: &str) -> anyhow::Result<()> {
    let (role_id, permission_str) = args.split_once(' ').unwrap_or((args, ""));
    let Some(role_id) = parse_role_id_or_mention(role_id, ctx.guild_id) else {
        ctx.reply(create_embed!(
            description: "Could not parse the role ID or role mention.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };
    let Some(permission) = parse_permission_str(permission_str.trim()) else {
        ctx.reply(create_embed!(
            description: "Could not parse the permission string.",
            color: FAILURE,
        ))
        .await?;
        return Ok(());
    };

    let permissions = ctx.db.list_guild_permissions(ctx.guild_id).await?;
    let existing_permission = permissions
        .iter()
        .find(|entry| entry.entity == GuildPermissionEntity::Role(role_id));
    let permission = if let Some(existing_permission) = existing_permission {
        existing_permission.allow.difference(permission)
    } else {
        BotPermissions::empty()
    };

    ctx.db
        .set_guild_permissions(ctx.guild_id, role_id, permission)
        .await?;

    ctx.reply(create_embed!(
        description: format!("Permissions for <@&{role_id}> updated."),
        color: SUCCESS,
    ))
    .await?;

    Ok(())
}

fn parse_role_id_or_mention(s: &str, guild_id: Id<GuildMarker>) -> Option<Id<RoleMarker>> {
    if s == "@everyone" || s == "everyone" {
        return Some(guild_id.cast());
    }
    let Some(s) = s.strip_prefix("<@&") else {
        return Id::try_from(s).ok();
    };
    let s = s.strip_suffix('>')?;
    Id::try_from(s).ok()
}

fn parse_permission_str(s: &str) -> Option<BotPermissions> {
    Some(match s.to_lowercase().as_str() {
        "create_bounties" | "create_bounty" | "create-bounties" | "create-bounty"
        | "create bounties" | "create bounty" => BotPermissions::CREATE_BOUNTIES,
        "manage_bounties" | "manage-bounties" | "manage bounties" => {
            BotPermissions::MANAGE_BOUNTIES
        }
        "manage_guild_config"
        | "manage-guild-config"
        | "manage_community_config"
        | "manage-community-config"
        | "manage community config"
        | "manage guild config" => BotPermissions::MANAGE_GUILD_CONFIG,
        "bounty_hunter" | "bounty-hunter" | "bounty hunter" => BotPermissions::BOUNTY_HUNTER,
        _ => return None,
    })
}

fn permissions_to_string(permissions: BotPermissions) -> String {
    permissions
        .iter_names()
        .map(|(name, _permission)| name)
        .collect::<Vec<&str>>()
        .join(", ")
}
