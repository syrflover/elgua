use serenity::prelude::Context;

use crate::{interaction::Interaction, usecase};

pub async fn track<'a>(ctx: &Context, interaction: Interaction<'a>) -> crate::Result<()> {
    let r = usecase::track(ctx).await?;

    interaction.send_message(&ctx.http, r).await?;

    Ok(())
}
