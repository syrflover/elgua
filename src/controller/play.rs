use serenity::{
    builder::CreateActionRow,
    model::prelude::interaction::{
        application_command::{CommandDataOption, CommandDataOptionValue},
        message_component::MessageComponentInteractionData,
    },
    prelude::Context,
    utils::{EmbedMessageBuilding, MessageBuilder},
};

use crate::{
    audio::{scdl, ytdl},
    cfg::Cfg,
    component::{create_numbering_select_menu, create_play_button},
    event::{Event, EventSender},
    interaction::Interaction,
    route::Route,
    usecase,
};

#[derive(Debug)]
pub struct Parameter {
    keyword: String,
    volume: Option<f32>,
    repeat_count: Option<usize>,
}

impl From<String> for Parameter {
    fn from(keyword: String) -> Self {
        Self {
            keyword,
            volume: None,
            repeat_count: None,
        }
    }
}

impl From<&Vec<CommandDataOption>> for Parameter {
    fn from(options: &Vec<CommandDataOption>) -> Self {
        let keyword = {
            let x = options
                .iter()
                .find(|x| x.name == "music")
                .and_then(|x| x.resolved.as_ref())
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
                .and_then(|x| x.resolved.as_ref());

            match x {
                Some(CommandDataOptionValue::Integer(v)) => Some(*v as f32 / 100.0),
                None => None, // 0.05
                _ => {
                    unreachable!()
                }
            }
        };

        let repeat_count = {
            let x = options
                .iter()
                .find(|x| x.name == "repeat_count")
                .and_then(|x| x.resolved.as_ref());

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
            repeat_count,
        }
    }
}

impl From<&MessageComponentInteractionData> for Parameter {
    fn from(data: &MessageComponentInteractionData) -> Self {
        let keyword = data.values.get(0).cloned().unwrap();

        Self {
            keyword,
            volume: None,
            repeat_count: None,
        }
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

pub async fn play<'a>(
    ctx: &Context,
    interaction: Interaction<'a>,
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
        repeat_count,
    } = parameter;

    let content_kind = ContentKind::new(&keyword);
    let user_id = interaction.user().id;

    log::info!("{content_kind:?}");

    match content_kind {
        ContentKind::YouTubeUrl | ContentKind::SoundCloudUrl => {
            let url = if keyword.contains("youtube.com/shorts/") {
                keyword.replacen("shorts", "watch", 1)
            } else {
                keyword
            };

            interaction.send_message(&ctx.http, "재생하는 중").await?;

            let parameter = usecase::play::Parameter::new(
                content_kind.into(),
                url.clone(),
                volume,
                repeat_count,
            );
            let (audio_metadata, volume, prev_message_id) =
                usecase::play(ctx, cfg.guild_id, cfg.voice_channel_id, parameter).await?;

            interaction
                .edit_original_interaction_response(&ctx.http, |edit| {
                    let play_button = create_play_button(Route::PlayFromClickedButton(
                        audio_metadata.url.clone(),
                    ));

                    let action_row = CreateActionRow::default()
                        .add_button(play_button)
                        .to_owned();

                    let x = MessageBuilder::new()
                        .push_named_link(&audio_metadata.title, &audio_metadata.url)
                        .push("\n소리 크기: ")
                        .push((volume * 100.0) as u8)
                        .push("\n재생 횟수: ")
                        .push(repeat_count.unwrap_or(1))
                        .build();

                    edit.content(x)
                        .components(|components| components.set_action_row(action_row))
                })
                .await?;

            let event = Event::Play(audio_metadata.clone(), volume, user_id, prev_message_id);
            if let Err(err) = event_tx.send((ctx.clone(), event)).await {
                panic!("closed event channel: {}", err)
            }
        }

        ContentKind::YouTubeSearchKeyword => {
            interaction.send_message(&ctx.http, "검색하는 중").await?;

            let searched_videos = ytdl::search(&cfg.youtube_api_key, &keyword).await?;

            interaction
                .edit_original_interaction_response(&ctx.http, |edit| {
                    // title, channel_name, url
                    let select_menu_items = searched_videos
                        .into_iter()
                        .map(|x| (x.title, x.uploaded_by, x.url));

                    let select_menu = create_numbering_select_menu(
                        Route::PlayFromSelectedMenu,
                        "재생할 음악을 선택해 주세요",
                        select_menu_items,
                    );

                    /* let button = CreateButton::default()
                    .custom_id("play-cancel-0")
                    .label("취소")
                    .style(ButtonStyle::Secondary)
                    .to_owned(); */

                    let action_row = CreateActionRow::default()
                        .add_select_menu(select_menu)
                        .to_owned();

                    edit.content(keyword)
                        .components(|components| components.add_action_row(action_row))
                })
                .await?;
        }
    };

    Ok(())
}
