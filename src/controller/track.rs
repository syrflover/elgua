use serenity::{all::Interaction, prelude::Context};

use crate::{interaction::InteractionExtension, usecase};

pub async fn track(ctx: &Context, interaction: &Interaction) -> crate::Result<()> {
    let r = usecase::track(ctx).await?;

    interaction.send_message(&ctx.http, r).await?;

    Ok(())
}
