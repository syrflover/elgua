use serenity::{
    model::prelude::interaction::application_command::{CommandDataOption, CommandDataOptionValue},
    prelude::Context,
};

use crate::{cfg::Cfg, interaction::Interaction, usecase};

// pub struct Parameter {
//     volume: f32,
//     history_channel_id: ChannelId,
// }

pub struct Parameter(f32);

impl From<&Vec<CommandDataOption>> for Parameter {
    fn from(options: &Vec<CommandDataOption>) -> Self {
        let volume = {
            let x = options.get(0).unwrap().resolved.as_ref().unwrap();

            match x {
                CommandDataOptionValue::Integer(volume) => *volume as f32 / 100.0,
                _ => unreachable!(),
            }
        };

        Self(volume)
    }
}

pub async fn volume<'a>(
    ctx: &Context,
    interaction: Interaction<'a>,
    Parameter(volume): Parameter,
) -> crate::Result<()> {
    let cfg = {
        let x = ctx.data.read().await;
        let cfg = x.get::<Cfg>().cloned().unwrap();
        // let event_tx = x.get::<EventSender>().cloned().unwrap();

        cfg
    };

    let x = usecase::volume(ctx, cfg.history_channel_id, volume).await?;

    interaction.send_message(&ctx.http, x).await?;

    Ok(())
}
