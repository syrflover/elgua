use std::{sync::Arc, time::Duration};

use chrono::Utc;
use serenity::{
    model::id::{ChannelId, GuildId, UserId},
    prelude::{Context, Mutex},
    utils::{EmbedMessageBuilding, MessageBuilder},
};
use songbird::{error::JoinError, tracks::PlayMode, Call};
use tokio::time::sleep;

use crate::store::{History, HistoryKind, Store};

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
    channel_id: ChannelId,
    user_id: UserId,
    url: &str,
    volume: Option<f32>,
) -> crate::Result<String> {
    let handler = get_voice_handler(ctx, guild_id, channel_id).await?;
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

    let source = songbird::input::ytdl(url).await?;

    let metadata = source.metadata.clone();

    handler.stop();

    let mut x = ctx.data.write().await;

    let volume = {
        match volume {
            Some(volume) => volume,
            None => {
                let store = x.get::<Store>().unwrap();
                let history = store.history().find_one(HistoryKind::YouTube, url).await?;

                match history {
                    Some(history) => history.volume as f32 / 100.0,
                    None => 0.05,
                }
            }
        }
    };

    let track = handler.play_source(source);

    let mut try_count = 0;

    loop {
        try_count += 1;
        if try_count >= 3 {
            return Ok("재생하지 못 했어요".to_string());
        }

        track.set_volume(volume)?;
        track.play()?;

        if let PlayMode::Play = track.get_info().await.unwrap().playing {
            break;
        }

        sleep(Duration::from_millis(100)).await;
    }

    log::info!("play: try_count = {try_count}");

    {
        let store = x.get::<Store>().unwrap();

        let history_id = {
            let history = History {
                id: 0,
                kind: HistoryKind::YouTube,
                url: metadata.source_url.as_deref().unwrap_or(url).to_string(),
                user_id: user_id.0,
                volume: (volume * 100.0) as u8,
                created_at: Utc::now(),
            };

            store.history().add(&history).await?
        };

        x.insert::<Track>(Track(history_id, track.clone()));
    }

    log::info!("url = {}", metadata.source_url.as_ref().unwrap());
    log::info!("volume = {}", volume);

    Ok(MessageBuilder::new()
        .push_named_link(metadata.title.unwrap(), metadata.source_url.unwrap())
        .build())
}
