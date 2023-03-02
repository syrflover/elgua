use serenity::prelude::Context;

use crate::{interaction::Interaction, usecase};

pub async fn stop<'a>(ctx: &Context, interaction: Interaction<'a>) -> crate::Result<()> {
    let r = usecase::stop(ctx).await?;

    interaction.send_message(&ctx.http, r).await?;

    Ok(())
}
