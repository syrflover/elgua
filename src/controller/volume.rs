use serenity::{
    all::{CommandDataOption, CommandDataOptionValue, Interaction},
    prelude::Context,
};

use crate::{cfg::Cfg, interaction::InteractionExtension, usecase};

// pub struct Parameter {
//     volume: f32,
//     history_channel_id: ChannelId,
// }

pub struct Parameter(f32);

impl From<&Vec<CommandDataOption>> for Parameter {
    fn from(options: &Vec<CommandDataOption>) -> Self {
        let volume = {
            let x = &options.first().as_ref().unwrap().value;

            match x {
                CommandDataOptionValue::Integer(volume) => *volume as f32 / 100.0,
                _ => unreachable!(),
            }
        };

        Self(volume)
    }
}

pub async fn volume(
    ctx: &Context,
    interaction: &Interaction,
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
