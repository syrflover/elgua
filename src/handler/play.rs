use std::{sync::Arc, time::Duration};

use chrono::Utc;
use serenity::{
    builder::{CreateActionRow, CreateComponents, CreateEmbed, CreateEmbedAuthor},
    model::id::{ChannelId, GuildId, MessageId, UserId},
    prelude::{Context, Mutex},
};
use songbird::{error::JoinError, input::Metadata, tracks::PlayMode, Call};
use tokio::time::sleep;

use crate::{
    handler::create_play_button,
    store::{History, HistoryKind, Store},
};

use super::Track;

pub async fn get_voice_handler(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<Arc<Mutex<Call>>, JoinError> {
    let manager = songbird::get(ctx).await.unwrap().clone();

    let (handler_lock, join_result) = manager.join(guild_id, channel_id).await;

    join_result.map(|_| handler_lock)
}

pub async fn play(
    ctx: &Context,
    guild_id: GuildId,
    voice_channel_id: ChannelId,
    history_channel_id: ChannelId,
    user_id: UserId,
    url: &str,
    volume: Option<f32>,
) -> crate::Result<(Metadata, f32)> {
    let handler = get_voice_handler(ctx, guild_id, voice_channel_id).await?;
    let mut handler = handler.lock().await;

    /* let track = if url.contains("soundcloud.com") {
        todo!()
    } else {
        /* if url.contains("youtube.com") || url.contains("youtu.be") */
    }; */

    /* let source = if url.contains("youtube.com") || url.contains("youtu.be") {
        songbird::input::ytdl(url).await?
    } else {
        songbird::input::ytdl_search(url).await?
    }; */

    let mut x = ctx.data.write().await;

    let (volume, prev_message_id) = {
        match volume {
            Some(volume) => (volume, None),
            None => {
                let store = x.get::<Store>().unwrap();
                let history = store.history().find_one(HistoryKind::YouTube, url).await?;

                match history {
                    Some(history) => (
                        history.volume as f32 / 100.0,
                        history.message_id.map(MessageId),
                    ),
                    None => (0.05, None),
                }
            }
        }
    };

    handler.stop();

    let mut track;
    let mut metadata;

    let mut try_count = 0;

    loop {
        let source = songbird::input::ytdl(url).await?;
        metadata = source.metadata.clone();
        track = handler.play_source(source);

        if try_count > 3 {
            return Err(crate::error::Error::CustomError(
                "not played this track".to_string(),
            ));
        }

        log::debug!("try_count = {try_count}");

        try_count += 1;

        track.set_volume(volume)?;
        track.play()?;

        if track.get_info().await?.playing == PlayMode::Play {
            break;
        }

        sleep(Duration::from_millis(200)).await;
    }

    let now = Utc::now();

    let message = {
        let user = user_id.to_user(&ctx.http).await?;

        let author = {
            let mut x = CreateEmbedAuthor::default().name(&user.name).to_owned();
            if let Some(avatar_url) = user.avatar_url() {
                x.icon_url(avatar_url);
            }
            x
        };

        let embed = {
            let mut x = CreateEmbed::default()
                .set_author(author)
                .title(metadata.title.as_ref().unwrap())
                .field("채널", metadata.channel.as_ref().unwrap(), true)
                .field("소리 크기", (volume * 100.0) as u8, true)
                .url(metadata.source_url.as_ref().unwrap())
                .timestamp(now)
                .to_owned();

            if let Some(image_url) = metadata.thumbnail.as_ref() {
                x.image(image_url);
            };
            x
        };

        let play_button = create_play_button(url);
        let action_row = CreateActionRow::default()
            .add_button(play_button)
            .to_owned();
        let components = CreateComponents::default()
            .add_action_row(action_row)
            .to_owned();

        if let Some(prev_message_id) = prev_message_id {
            if let Ok(prev_message) = ctx
                .http
                .get_message(history_channel_id.0, prev_message_id.0)
                .await
            {
                prev_message.delete(&ctx.http).await?;
            }
        };

        history_channel_id
            .send_message(&ctx.http, |message| {
                message.set_embed(embed).set_components(components)
            })
            .await?
    };

    {
        let store = x.get::<Store>().unwrap();

        let history_id = {
            let history = History {
                id: 0,
                message_id: Some(message.id.0),
                title: metadata.title.clone().unwrap(),
                channel: metadata.channel.clone().unwrap(),
                kind: HistoryKind::YouTube,
                url: metadata.source_url.as_deref().unwrap_or(url).to_string(),
                user_id: user_id.0,
                volume: (volume * 100.0) as u8,
                created_at: now,
            };

            store.history().add(&history).await?
        };

        x.insert::<Track>(Track(history_id, track));
    }

    log::info!("url = {}", metadata.source_url.as_ref().unwrap());
    log::info!("volume = {}", volume);

    Ok((*metadata, volume))
}
