use std::fmt::Display;
use std::str::FromStr;

use serenity::model::prelude::interaction::{
    application_command::ApplicationCommandInteraction,
    message_component::MessageComponentInteraction,
};

use serenity::prelude::Context;

use crate::cfg::Cfg;

use super::{controller, interaction::Interaction};

mod route_constant {
    pub const PING: &str = "ping";
    pub const PLAY: &str = "play";
    pub const VOLUME: &str = "volume";
    pub const PLAY_FROM_SELECTED_MENU: &str = "play-from-selected-menu";
    pub const PLAY_FROM_CLICKED_BUTTON: &str = "play-from-clicked-button#";

    // pub const DEPRECATED_PLAY_FROM_SELECTED_MENU: &str = "play-yt-select-0";
    pub const DEPRECATED_PLAY_FROM_CLICKED_BUTTON: &str = "play-yt-button-0;";
}

pub enum Route {
    Ping,
    Play,
    Volume,
    PlayFromSelectedMenu,
    PlayFromClickedButton(String),
}

impl Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Route::*;

        let x = match self {
            Ping => route_constant::PING,

            Play => route_constant::PLAY,

            Volume => route_constant::VOLUME,

            PlayFromSelectedMenu => route_constant::PLAY_FROM_SELECTED_MENU,

            PlayFromClickedButton(url) => {
                return write!(f, "{}{url}", route_constant::PLAY_FROM_CLICKED_BUTTON)
            }
        };

        f.write_str(x)
    }
}

impl FromStr for Route {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl TryFrom<&str> for Route {
    type Error = ();

    fn try_from(x: &str) -> Result<Self, Self::Error> {
        use Route::*;

        let r = match x {
            route_constant::PING => Ping,

            route_constant::PLAY => Play,

            route_constant::VOLUME => Volume,

            route_constant::PLAY_FROM_SELECTED_MENU => PlayFromSelectedMenu,

            // route_constant::DEPRECATED_PLAY_FROM_SELECTED_MENU => PlayFromSelectedMenu,
            x if x.starts_with(route_constant::PLAY_FROM_CLICKED_BUTTON) => {
                let (_, url) = x.split_once('#').unwrap();

                PlayFromClickedButton(url.to_string())
            }

            x if x.starts_with(route_constant::DEPRECATED_PLAY_FROM_CLICKED_BUTTON) => {
                let (_, url) = x.split_once(';').unwrap();

                PlayFromClickedButton(url.to_string())
            }

            _ => return Err(()),
        };

        Ok(r)
    }
}

pub async fn route_application_command(
    ctx: &Context,
    interaction: &ApplicationCommandInteraction,
) -> crate::Result<()> {
    let options = &interaction.data.options;

    // let typing = interaction.channel_id.start_typing(&ctx.http)?;

    match interaction.data.name.as_str().try_into().ok() {
        Some(Route::Ping) => {
            // command.channel_id.say(&ctx.http, "pong").await.unwrap();
            let interaction: Interaction = interaction.into();
            interaction.send_message(&ctx.http, "pong").await?;
        }

        Some(Route::Play) => {
            let parameter = controller::play::Parameter::from(options);

            controller::play(ctx, interaction.into(), parameter).await?;

            // interaction.defer(&ctx.http).await?;
        }

        Some(Route::Volume) => {
            let parameter = controller::volume::Parameter::from(options);

            controller::volume(ctx, interaction.into(), parameter).await?;
        }

        _ => {}
    };

    // typing.stop().unwrap_or_default();

    Ok(())
}

pub async fn route_message_component(
    ctx: &Context,
    interaction: &mut MessageComponentInteraction,
) -> crate::Result<()> {
    // let typing = interaction.channel_id.start_typing(&ctx.http)?;

    match interaction.data.custom_id.as_str().try_into().ok() {
        Some(Route::PlayFromSelectedMenu) => {
            let parameter = controller::play::Parameter::from(&interaction.data);

            // select menu 메세지를 삭제함
            interaction.message.delete(&ctx.http).await?;

            controller::play(ctx, interaction.into(), parameter).await?;
        }

        Some(Route::PlayFromClickedButton(url)) => {
            let history_channel_id = ctx
                .data
                .read()
                .await
                .get::<Cfg>()
                .unwrap()
                .history_channel_id;

            let parameter = controller::play::Parameter::from(url);

            let do_interact = interaction.message.channel_id != history_channel_id;
            let interaction: Interaction = interaction.into();

            if do_interact {
                // history채널이 아닐때만 play button 메세지를 삭제함
                interaction.message().unwrap().delete(&ctx.http).await?;
            }

            controller::play(ctx, interaction.do_interact(do_interact), parameter).await?;
        }

        _ => {}
    }

    // typing.stop().unwrap_or_default();

    Ok(())
}
