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
    audio::ytdl,
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
}

impl From<String> for Parameter {
    fn from(keyword: String) -> Self {
        Self {
            keyword,
            volume: None,
        }
    }
}

impl From<&Vec<CommandDataOption>> for Parameter {
    fn from(options: &Vec<CommandDataOption>) -> Self {
        let keyword = {
            let x = options
                .get(0)
                .expect("expected str option")
                .resolved
                .as_ref()
                .expect("expected str object");

            match x {
                CommandDataOptionValue::String(st) => st.clone(),
                _ => {
                    unreachable!()
                }
            }
        };

        let volume = {
            let x = options.get(1).and_then(|x| x.resolved.as_ref());

            match x {
                Some(CommandDataOptionValue::Integer(v)) => Some(*v as f32 / 100.0),
                None => None, // 0.05
                _ => {
                    unreachable!()
                }
            }
        };

        Self { keyword, volume }
    }
}

impl From<&MessageComponentInteractionData> for Parameter {
    fn from(data: &MessageComponentInteractionData) -> Self {
        let keyword = data.values.get(0).cloned().unwrap();

        Self {
            keyword,
            volume: None,
        }
    }
}

enum ContentKind {
    YouTubeUrl,
    YouTubeSearchKeyword,
    SoundCloudUrl,
}

impl ContentKind {
    pub fn new(x: &str) -> Self {
        if is_youtube_url(x) {
            Self::YouTubeUrl
        } else if is_soundcloud_url(x) {
            Self::SoundCloudUrl
        } else {
            Self::YouTubeSearchKeyword
        }
    }
}

fn is_youtube_url(x: &str) -> bool {
    x.starts_with("https://www.youtube.com/watch")
        || x.starts_with("https://www.youtube.com/v/")
        || x.starts_with("https://youtu.be/")
}

fn is_soundcloud_url(_x: &str) -> bool {
    // TODO:
    false
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

    let Parameter { keyword, volume } = parameter;

    let content_kind = ContentKind::new(&keyword);
    let user_id = interaction.user().id;

    match content_kind {
        ContentKind::YouTubeUrl => {
            let url = /* if music.starts_with("youtube.com/watch") || music.contains("youtu.be/") {
                music
            } else */ if keyword.contains("youtube.com/shorts/") {
                keyword.replacen("shorts", "watch", 1)
            } else {
                keyword.clone()
            };

            interaction.send_message(&ctx.http, "재생하는 중").await?;

            let (audio_metadata, volume, prev_message_id) =
                usecase::play(ctx, cfg.guild_id, cfg.voice_channel_id, &url, volume).await?;

            interaction
                .edit_original_interaction_response(&ctx.http, |edit| {
                    let play_button = create_play_button(Route::PlayFromClickedButton(url));

                    let action_row = CreateActionRow::default()
                        .add_button(play_button)
                        .to_owned();

                    let x = MessageBuilder::new()
                        .push_named_link(&audio_metadata.title, &audio_metadata.url)
                        .push("\n소리 크기: ")
                        .push((volume * 100.0) as u8)
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

        ContentKind::SoundCloudUrl => unimplemented!("not implemented soundcloud"),
    };

    Ok(())
}
