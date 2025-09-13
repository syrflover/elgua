use serenity::{all::Interaction, prelude::Context};

use crate::{interaction::InteractionExtension, usecase};

pub async fn stop(ctx: &Context, interaction: &Interaction) -> crate::Result<()> {
    let r = usecase::stop(ctx).await?;

    interaction.send_message(&ctx.http, r).await?;

    Ok(())
}
