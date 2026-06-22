use chrono::{DateTime, Utc};
use fluxer_neptunium::{create_embed, exts::MessageExt};

use crate::{colors::DEFAULT, commands::CommandContext};

pub async fn ping(ctx: CommandContext<'_>) -> anyhow::Result<()> {
    let latency = {
        let now = Utc::now();
        let created_at: DateTime<Utc> = ctx.message.timestamp.into();
        now.signed_duration_since(created_at)
    };
    ctx.message
        .reply(
            ctx.ctx,
            create_embed!(
                title: "Pong!",
                description: format!("**Latency:** {} ms", latency.num_milliseconds()),
                color: DEFAULT,
            ),
        )
        .await?;
    Ok(())
}

pub async fn bounty_workflow(ctx: CommandContext<'_>) -> anyhow::Result<()> {
    ctx.message
        .reply(ctx.ctx, ctx.bounty_workflow_image_url)
        .await?;
    Ok(())
}
