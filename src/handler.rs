use serenity::{
    builder::{CreateApplicationCommandOption, CreateApplicationCommands},
    model::prelude::{command::CommandOptionType, Ready},
    prelude::{Context, EventHandler},
};

use crate::{
    cfg::Cfg,
    route::{route_application_command, route_message_component},
};

pub struct Handler;

#[async_trait::async_trait]
impl EventHandler for Handler {
    async fn interaction_create(
        &self,
        ctx: Context,
        interaction: serenity::model::application::interaction::Interaction,
    ) {
        use serenity::model::application::interaction::Interaction;

        match interaction {
            Interaction::ApplicationCommand(interaction) => {
                if let Err(err) = route_application_command(&ctx, &interaction).await {
                    log::error!("{err:?}");

                    interaction
                        .edit_original_interaction_response(&ctx.http, |message| {
                            message
                                .content(err)
                                .components(|c| c.set_action_rows(Default::default()))
                        })
                        .await
                        .unwrap();
                }
            }

            Interaction::MessageComponent(mut interaction) => {
                if let Err(err) = route_message_component(&ctx, &mut interaction).await {
                    log::error!("{err:?}");

                    interaction
                        .message
                        .edit(&ctx.http, |message| {
                            message.content(err).set_components(Default::default())
                        })
                        .await
                        .ok();
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        let x = ctx.data.read().await;
        let cfg = x.get::<Cfg>().unwrap();

        cfg.guild_id
            .set_application_commands(&ctx.http, set_commands)
            .await
            .unwrap();
    }
}

fn set_commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
    commands
        // ping
        .create_application_command(|command| command.name("ping").description("퐁!"))
        // play
        .create_application_command(|command| {
            command
                .name("play")
                .description("음악을 재생해요")
                .create_option(|option| keyword_option(option).required(true))
                .create_option(|option| volume_option(option).required(false))
        })
        // volume
        .create_application_command(|command| {
            command
                .name("volume")
                .description("재생 중인 음악의 소리 크기를 조절해요")
                .create_option(|option| volume_option(option).required(true))
        })
}

fn volume_option(
    option: &mut CreateApplicationCommandOption,
) -> &mut CreateApplicationCommandOption {
    option
        .name("volume")
        .description("음악의 소리 크기(1 ~ 100)를 설정해 주세요")
        .kind(CommandOptionType::Integer)
        .min_int_value(1)
        .max_int_value(100)
}

fn keyword_option(
    option: &mut CreateApplicationCommandOption,
) -> &mut CreateApplicationCommandOption {
    option
        .name("music")
        .description("음악의 주소 또는 유튜브 검색어를 적어주세요")
        .kind(CommandOptionType::String)
}
