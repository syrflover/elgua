use serenity::{
    all::{
        CommandOptionType, CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Interaction,
        InteractionType,
    },
    model::prelude::Ready,
    prelude::{Context, EventHandler},
};
use tap::TapFallible;

use crate::{
    cfg::Cfg,
    interaction::InteractionExtension,
    route::{route_application_command, route_message_component},
};

pub struct Handler;

#[async_trait::async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction.kind() {
            InteractionType::Command => {
                if let Err(err) = route_application_command(&ctx, &interaction).await {
                    log::error!("{err:?}");

                    interaction
                        .create_followup(
                            &ctx.http,
                            CreateInteractionResponseFollowup::new().content(err.to_string()),
                        )
                        .await
                        .tap_err(|err| log::error!("{err:?}"))
                        .ok();
                }
            }

            InteractionType::Component => {
                if let Err(err) = route_message_component(&ctx, &interaction).await {
                    log::error!("{err:?}");

                    interaction
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new().content(err.to_string()),
                            ),
                        )
                        .await
                        .tap_err(|err| log::error!("{err:?}"))
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
            .set_commands(&ctx.http, commands())
            .await
            .unwrap();
    }
}

fn commands() -> Vec<CreateCommand> {
    vec![
        CreateCommand::new("ping").description("퐁"),
        CreateCommand::new("play")
            .description("음악을 재생해요")
            .set_options(vec![
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "music",
                    "음악의 주소 또는 유튜브 검색어를 입력해 주세요.",
                )
                .required(true),
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "volume",
                    "음악의 소리 크기(1 ~ 100)를 입력해 주세요.",
                )
                .min_int_value(1)
                .max_int_value(100)
                .required(false),
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "play_count",
                    "재생 횟수를 입력해 주세요.",
                )
                .min_int_value(1)
                .max_int_value(523)
                .required(false),
            ]),
        CreateCommand::new("volume")
            .description("재생 중인 음악의 소리 크기를 조절해요.")
            .set_options(vec![CreateCommandOption::new(
                CommandOptionType::Integer,
                "volume",
                "음악의 소리 크기(1 ~ 100)를 입력해 주세요.",
            )
            .min_int_value(1)
            .max_int_value(100)
            .required(true)]),
        CreateCommand::new("stop").description("재생 중인 음악을 중지해요."),
        CreateCommand::new("track").description("재생 중인 음악의 정보를 가져와요."),
        CreateCommand::new("sc")
            .description("SoundCloud Client ID를 업데이트해요")
            .set_options(vec![CreateCommandOption::new(
                CommandOptionType::String,
                "sc_client_id",
                "sc_client_id",
            )
            .required(true)]),
    ]
}

// fn keyword_option(
//     option: &mut CreateApplicationCommandOption,
// ) -> &mut CreateApplicationCommandOption {
//     option
//         .name("music")
//         .description("음악의 주소 또는 유튜브 검색어를 적어주세요")
//         .kind(CommandOptionType::String)
// }

// fn volume_option(
//     option: &mut CreateApplicationCommandOption,
// ) -> &mut CreateApplicationCommandOption {
//     option
//         .name("volume")
//         .description("음악의 소리 크기(1 ~ 100)를 설정해 주세요")
//         .kind(CommandOptionType::Integer)
//         .min_int_value(1)
//         .max_int_value(100)
// }

// fn play_count(option: &mut CreateApplicationCommandOption) -> &mut CreateApplicationCommandOption {
//     option
//         .name("play_count")
//         .description("재생할 횟수를 설정해 주세요")
//         .kind(CommandOptionType::Integer)
//         .min_int_value(1)
//         .max_int_value(523)
// }
