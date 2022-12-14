mod play;
mod volume;

use play::play;

use serenity::builder::{
    CreateActionRow, CreateApplicationCommandOption, CreateApplicationCommands, CreateComponents,
    CreateSelectMenu,
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
use serenity::utils::{EmbedMessageBuilding, MessageBuilder};
use songbird::tracks::TrackHandle;

use crate::cfg::Cfg;
use crate::component::create_play_button;
use crate::event::EventSender;
use crate::ytdl;

use self::volume::volume;

pub struct Track(pub String, pub TrackHandle);

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
    let (cfg, event_tx) = {
        let x = ctx.data.read().await;
        let cfg = x.get::<Cfg>().cloned().unwrap();
        let event_tx = x.get::<EventSender>().cloned().unwrap();

        (cfg, (&*event_tx).clone())
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

                message_send(ctx, command, "???????????? ???").await?;

                let uid = ytdl::parse_vid(url.parse().unwrap());

                let (metadata, volume) = play(
                    ctx,
                    event_tx,
                    cfg.guild_id,
                    cfg.voice_channel_id,
                    user_id,
                    &url,
                    volume,
                )
                .await?;

                command
                    .edit_original_interaction_response(&ctx.http, |edit| {
                        let play_button = create_play_button(&uid);

                        let action_row = CreateActionRow::default()
                            .add_button(play_button)
                            .to_owned();

                        let x = MessageBuilder::new()
                            .push_named_link(metadata.title.unwrap(), metadata.source_url.unwrap())
                            .push("\n?????? ??????: ")
                            .push((volume * 100.0) as u8)
                            .build();

                        edit.content(x)
                            .components(|components| components.set_action_row(action_row))
                    })
                    .await?;
            } else {
                message_send(ctx, command, "???????????? ???").await?;

                let metadata_vec = ytdl::search(&cfg.youtube_api_key, music).await?;

                command
                    .edit_original_interaction_response(&ctx.http, |edit| {
                        let select_menu = CreateSelectMenu::default()
                            .min_values(1)
                            .max_values(1)
                            .placeholder("????????? ????????? ????????? ?????????")
                            .custom_id("play-yt-select-0")
                            .options(|x| {
                                metadata_vec.into_iter().enumerate().take(5).fold(
                                    x,
                                    |acc, (i, metadata)| {
                                        let num_emoji = match i + 1 {
                                            1 => "1??????",
                                            2 => "2??????",
                                            3 => "3??????",
                                            4 => "4??????",
                                            5 => "5??????",
                                            _ => unreachable!(),
                                        }
                                        .to_string();

                                        let title = metadata.title.unwrap();
                                        let label = if title.chars().count() > 100 {
                                            title
                                                .chars()
                                                .take(96)
                                                .chain(" ...".chars())
                                                .collect::<String>()
                                        } else {
                                            title
                                        };

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

                        /* let button = CreateButton::default()
                        .custom_id("play-cancel-0")
                        .label("??????")
                        .style(ButtonStyle::Secondary)
                        .to_owned(); */

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

            let x = volume(ctx, cfg.history_channel_id, volume_).await?;

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
    let (cfg, event_tx) = {
        let x = ctx.data.read().await;
        let cfg = x.get::<Cfg>().cloned().unwrap();
        let event_tx = x.get::<EventSender>().cloned().unwrap();

        (cfg, (&*event_tx).clone())
    };
    let user_id = command.user.id;

    match command.data.custom_id.as_str() {
        "play-yt-select-0" => {
            let (url, volume): (String, Option<f32>) = {
                let mut x = command.data.values.get(0).unwrap().split(';');
                (
                    x.next().unwrap().to_string(),
                    x.next().and_then(|x| x.parse().ok()),
                )
            };
            let uid = ytdl::parse_vid(url.parse().unwrap());

            command.defer(&ctx.http).await?;

            let (metadata, volume) = play(
                ctx,
                event_tx,
                cfg.guild_id,
                cfg.voice_channel_id,
                user_id,
                &url,
                volume,
            )
            .await?;

            command
                .message
                .edit(&ctx.http, |message| {
                    let play_button = create_play_button(&uid);
                    let action_row = CreateActionRow::default()
                        .add_button(play_button)
                        .to_owned();
                    let components = CreateComponents::default()
                        .add_action_row(action_row)
                        .to_owned();

                    let x = MessageBuilder::new()
                        .push_named_link(metadata.title.unwrap(), &url)
                        .push("\n?????? ??????: ")
                        .push((volume * 100.0) as u8)
                        .build();

                    message.content(x).set_components(components)
                })
                .await?;
        }

        x if x.starts_with("play-yt-button-0;") => {
            let mut x = x.splitn(2, ';');
            let url = x.nth(1).unwrap();

            command.defer(&ctx.http).await?;

            play(
                ctx,
                event_tx,
                cfg.guild_id,
                cfg.voice_channel_id,
                user_id,
                url,
                None,
            )
            .await?;
        }
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
        .create_application_command(|command| command.name("ping").description("???!"))
        // play
        .create_application_command(|command| {
            command
                .name("play")
                .description("????????? ????????????")
                .create_option(|option| {
                    option
                        .name("music")
                        .description("????????? ?????? ?????? ????????? ???????????? ???????????????")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
                .create_option(|option| volume_option(option).required(false))
        })
        // volume
        .create_application_command(|command| {
            command
                .name("volume")
                .description("?????? ?????? ????????? ?????? ????????? ????????????")
                .create_option(|option| volume_option(option).required(true))
        })
}

fn volume_option(
    option: &mut CreateApplicationCommandOption,
) -> &mut CreateApplicationCommandOption {
    option
        .name("volume")
        .description("????????? ?????? ??????(1 ~ 100)??? ????????? ?????????")
        .kind(CommandOptionType::Integer)
        .min_int_value(1)
        .max_int_value(100)
}
