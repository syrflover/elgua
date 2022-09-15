mod play;
mod volume;

use play::play;

use serenity::builder::{
    CreateActionRow, CreateApplicationCommandOption, CreateApplicationCommands, CreateSelectMenu,
};
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOptionValue,
};
use serenity::model::prelude::interaction::message_component::MessageComponentInteraction;
use serenity::model::prelude::ReactionType;
use serenity::model::{
    application::interaction::Interaction, gateway::Ready, prelude::command::CommandOptionType,
};
use serenity::prelude::*;
use songbird::tracks::TrackHandle;

use crate::cfg::Cfg;
use crate::ytdl;

use self::volume::volume;

pub struct Track(pub u64, pub TrackHandle);

impl TypeMapKey for Track {
    type Value = Track;
}

pub struct Handler;

async fn message_send(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    m: impl ToString,
) -> serenity::Result<()> {
    command
        .create_interaction_response(&ctx.http, |resp| {
            resp.interaction_response_data(|message| message.content(m))
        })
        .await
}

async fn route_application_command(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> crate::Result<()> {
    let cfg = {
        let x = ctx.data.read().await;
        x.get::<Cfg>().cloned().unwrap()
    };
    let user_id = command.user.id;

    let options = &command.data.options;

    match command.data.name.as_str() {
        "ping" => {
            // command.channel_id.say(&ctx.http, "pong").await.unwrap();
            message_send(ctx, command, "pong").await?;
        }

        "play" => {
            let music = options
                .get(0)
                .expect("expected str option")
                .resolved
                .as_ref()
                .expect("expected str object");

            let volume = options.get(1).and_then(|x| x.resolved.as_ref());

            let music = match music {
                CommandDataOptionValue::String(st) => st,
                _ => {
                    unreachable!()
                }
            };

            let volume = match volume {
                Some(CommandDataOptionValue::Integer(v)) => Some(*v as f32 / 100.0),
                None => None, // 0.05
                _ => {
                    unreachable!()
                }
            };

            if music.starts_with("https://") {
                let url = /* if music.starts_with("youtube.com/watch") || music.contains("youtu.be/") {
                    music
                } else */ if music.contains("youtube.com/shorts/") {
                    music.replacen("shorts", "watch", 1)
                } else {
                    music.clone()
                };

                message_send(ctx, command, "재생하는 중").await?;

                let x = play(ctx, cfg.guild_id, cfg.channel_id, user_id, &url, volume).await?;

                command
                    .edit_original_interaction_response(&ctx.http, |edit| edit.content(x))
                    .await?;
            } else {
                message_send(ctx, command, "검색하는 중").await?;

                let metadata_vec = ytdl::search(&cfg.youtube_api_key, music).await?;

                command
                    .edit_original_interaction_response(&ctx.http, |edit| {
                        let select_menu = CreateSelectMenu::default()
                            .min_values(1)
                            .max_values(1)
                            .placeholder("재생할 음악을 선택해 주세요")
                            .custom_id("play-yt-select-0")
                            .options(|x| {
                                metadata_vec.into_iter().enumerate().take(5).fold(
                                    x,
                                    |acc, (i, metadata)| {
                                        let num_emoji = match i + 1 {
                                            1 => "1️⃣",
                                            2 => "2️⃣",
                                            3 => "3️⃣",
                                            4 => "4️⃣",
                                            5 => "5️⃣",
                                            _ => unreachable!(),
                                        }
                                        .to_string();

                                        let label = metadata.title.unwrap();
                                        let value = match volume {
                                            Some(volume) => format!(
                                                "{};{}",
                                                metadata.source_url.unwrap(),
                                                volume
                                            ),
                                            None => metadata.source_url.unwrap(),
                                        };
                                        let description = metadata.channel.unwrap();
                                        acc.create_option(|opt| {
                                            opt.label(label)
                                                .value(value)
                                                .emoji(ReactionType::Unicode(num_emoji))
                                                .description(description)
                                        })
                                    },
                                )
                            })
                            .to_owned();

                        let action_row = CreateActionRow::default()
                            .add_select_menu(select_menu)
                            .to_owned();

                        edit.content(music)
                            .components(|components| components.add_action_row(action_row))
                    })
                    .await?;
            }
        }

        "volume" => {
            let volume_ = options.get(0).unwrap().resolved.as_ref().unwrap();

            let volume_ = match volume_ {
                CommandDataOptionValue::Integer(volume) => *volume as f32 / 100.0,
                _ => unreachable!(),
            };

            let x = volume(ctx, volume_).await?;

            message_send(ctx, command, x).await?;
        }

        _ => {}
    };

    Ok(())
}

async fn route_message_component(
    ctx: &Context,
    command: &mut MessageComponentInteraction,
) -> crate::Result<()> {
    match command.data.custom_id.as_str() {
        "play-yt-select-0" => {
            let cfg = {
                let x = ctx.data.read().await;
                x.get::<Cfg>().cloned().unwrap()
            };
            let user_id = command.user.id;

            let (url, volume): (String, Option<f32>) = {
                let mut x = command.data.values.get(0).unwrap().split(';');
                (
                    x.next().unwrap().to_string(),
                    x.next().and_then(|x| x.parse().ok()),
                )
            };

            command.defer(&ctx.http).await?;

            let x = play(ctx, cfg.guild_id, cfg.channel_id, user_id, &url, volume).await?;

            command
                .message
                .edit(&ctx.http, |message| {
                    message.content(x).set_components(Default::default())
                })
                .await?;
        }
        " " => {}
        _ => {}
    }

    Ok(())
}

#[async_trait::async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                if let Err(err) = route_application_command(&ctx, &command).await {
                    log::error!("err: {err:?}");

                    command
                        .edit_original_interaction_response(&ctx.http, |message| {
                            message
                                .content(err)
                                .components(|c| c.set_action_rows(Default::default()))
                        })
                        .await
                        .unwrap();
                }
            }

            Interaction::MessageComponent(mut command) => {
                if let Err(err) = route_message_component(&ctx, &mut command).await {
                    log::error!("err: {err:?}");

                    command
                        .message
                        .edit(&ctx.http, |message| {
                            message.content(err).set_components(Default::default())
                        })
                        .await
                        .unwrap();
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
                .create_option(|option| {
                    option
                        .name("music")
                        .description("음악의 주소 또는 유튜브 검색어를 적어주세요")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
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
