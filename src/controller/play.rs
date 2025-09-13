use serenity::{
    all::{
        CommandDataOption, CommandDataOptionValue, ComponentInteractionData,
        ComponentInteractionDataKind, EditInteractionResponse, Interaction,
    },
    prelude::Context,
    utils::{EmbedMessageBuilding, MessageBuilder},
};

use crate::{
    audio::{scdl, ytdl},
    cfg::Cfg,
    component::{create_numbering_select_menu, create_play_button},
    event::{Event, EventSender},
    interaction::InteractionExtension,
    route::Route,
    usecase,
};

#[derive(Debug)]
pub struct Parameter {
    keyword: String,
    volume: Option<f32>,
    play_count: Option<usize>,
}

impl From<String> for Parameter {
    fn from(keyword: String) -> Self {
        Self {
            keyword,
            volume: None,
            play_count: None,
        }
    }
}

impl From<&Vec<CommandDataOption>> for Parameter {
    fn from(options: &Vec<CommandDataOption>) -> Self {
        let keyword = {
            let x = options
                .iter()
                .find(|x| x.name == "music")
                .map(|x| &x.value)
                .unwrap();

            match x {
                CommandDataOptionValue::String(st) => st.clone(),
                _ => {
                    unreachable!()
                }
            }
        };

        let volume = {
            let x = options
                .iter()
                .find(|x| x.name == "volume")
                .map(|x| &x.value);

            match x {
                Some(CommandDataOptionValue::Integer(v)) => Some(*v as f32 / 100.0),
                None => None, // 0.05
                _ => {
                    unreachable!()
                }
            }
        };

        let play_count = {
            let x = options
                .iter()
                .find(|x| x.name == "play_count")
                .map(|x| &x.value);

            match x {
                Some(CommandDataOptionValue::Integer(v)) => Some(*v as usize),
                None => None,
                _ => {
                    unreachable!()
                }
            }
        };

        Self {
            keyword,
            volume,
            play_count,
        }
    }
}

impl From<&ComponentInteractionData> for Parameter {
    fn from(data: &ComponentInteractionData) -> Self {
        match &data.kind {
            ComponentInteractionDataKind::Button => {
                if let Ok((volume, play_count, keyword)) = serde_json::from_str(&data.custom_id) {
                    Self {
                        keyword,
                        volume,
                        play_count,
                    }
                } else {
                    Self {
                        keyword: data.custom_id.clone(),
                        volume: None,
                        play_count: None,
                    }
                }
            }
            ComponentInteractionDataKind::StringSelect { values } => {
                log::debug!("selected values: {:#?}", values);

                let (volume, play_count, url): (Option<f32>, Option<usize>, String) =
                    serde_json::from_str(values.first().unwrap()).unwrap();

                Self {
                    keyword: url,
                    volume,
                    play_count,
                }
            }
            _ => {
                log::error!("Unexpected interaction data kind: {:?}", data.kind);
                unreachable!()
            }
        }
        // let x = data.values.get(0).cloned().unwrap();
    }
}

#[derive(Debug, Clone, Copy)]
enum ContentKind {
    YouTubeUrl,
    SoundCloudUrl,
    YouTubeSearchKeyword,
}

impl ContentKind {
    pub fn new(x: &str) -> Self {
        if ytdl::is_youtube_url(x) {
            Self::YouTubeUrl
        } else if scdl::is_soundcloud_url(x) {
            Self::SoundCloudUrl
        } else {
            Self::YouTubeSearchKeyword
        }
    }
}

impl From<ContentKind> for usecase::play::PlayableKind {
    fn from(x: ContentKind) -> Self {
        match x {
            ContentKind::YouTubeUrl => usecase::play::PlayableKind::YouTube,
            ContentKind::SoundCloudUrl => usecase::play::PlayableKind::SoundCloud,
            _ => unreachable!(),
        }
    }
}

pub async fn play(
    ctx: &Context,
    interaction: &Interaction,
    parameter: Parameter,
) -> crate::Result<()> {
    // let (cfg, event_tx) = {
    let (cfg, event_tx) = {
        let x = ctx.data.read().await;
        let cfg = x.get::<Cfg>().cloned().unwrap();
        let event_tx = x.get::<EventSender>().cloned().unwrap();

        (cfg, event_tx)
    };

    log::info!("{parameter:?}");

    let Parameter {
        keyword,
        volume,
        play_count,
    } = parameter;

    let content_kind = ContentKind::new(&keyword);
    let user_id = interaction.user().id;

    log::info!("content_kind={content_kind:?}");

    match content_kind {
        ContentKind::YouTubeUrl | ContentKind::SoundCloudUrl => {
            let url = if keyword.contains("youtube.com/shorts/") {
                keyword.replacen("shorts", "watch", 1)
            } else {
                keyword
            };

            interaction
                .send_message(
                    &ctx.http,
                    MessageBuilder::new()
                        .push("재생하는 중 : ")
                        .push(&url)
                        .build(),
                )
                .await?;

            let parameter =
                usecase::play::Parameter::new(content_kind.into(), url.clone(), volume, play_count);
            let (audio_metadata, volume, prev_message_id) =
                usecase::play(ctx, cfg.guild_id, cfg.voice_channel_id, parameter).await?;

            let do_interact = interaction.channel_id() != cfg.history_channel_id;

            if do_interact {
                interaction
                    .edit_response(&ctx.http, {
                        let play_info_str = MessageBuilder::new()
                            .push_named_link(&audio_metadata.title, &audio_metadata.url)
                            .push("\n소리 크기: ")
                            .push((volume * 100.0).to_string())
                            .push("\n재생 횟수: ")
                            .push(play_count.unwrap_or(1).to_string())
                            .build();

                        let play_button = create_play_button(Route::PlayFromClickedButton(
                            audio_metadata.url.clone(),
                        ));

                        EditInteractionResponse::new()
                            .content(play_info_str)
                            .button(play_button)
                    })
                    .await?;
            } else {
                interaction.delete_response(&ctx.http).await.ok();
            }

            let event = Event::Play(audio_metadata.clone(), volume, user_id, prev_message_id);
            if let Err(err) = event_tx.send((ctx.clone(), event)).await {
                panic!("closed event channel: {}", err)
            }
        }

        ContentKind::YouTubeSearchKeyword => {
            interaction
                .send_message(
                    &ctx.http,
                    MessageBuilder::new()
                        .push("검색하는 중 : ")
                        .push(&keyword)
                        .build(),
                )
                .await?;

            let searched_videos = ytdl::search(&cfg.youtube_api_key, &keyword).await?;

            interaction
                .edit_response(&ctx.http, {
                    let select_menu_items = searched_videos.into_iter().map(|x| {
                        (
                            x.title,
                            x.uploaded_by,
                            serde_json::to_string(&(volume, play_count, x.url)).unwrap(),
                        )
                    });

                    EditInteractionResponse::new()
                        .content(
                            MessageBuilder::new()
                                .push("검색 완료 : ")
                                .push(&keyword)
                                .build(),
                        )
                        .select_menu(create_numbering_select_menu(
                            Route::PlayFromSelectedMenu,
                            "재생할 음악을 선택해 주세요",
                            select_menu_items,
                        ))
                })
                .await?;
        }
    };

    Ok(())
}
