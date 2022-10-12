use std::{sync::Arc, time::Duration};

use serenity::{
    model::id::{ChannelId, GuildId, MessageId, UserId},
    prelude::{Context, Mutex},
};
use songbird::{error::JoinError, input::Metadata, tracks::PlayMode, Call};
use tokio::{sync::mpsc::Sender, time::sleep};

use crate::{
    event::Event,
    store::{HistoryKind, Store},
    ytdl,
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
    event_tx: Sender<(Context, Event)>,
    guild_id: GuildId,
    voice_channel_id: ChannelId,
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

    let uid = ytdl::parse_vid(url.parse().unwrap());

    let mut x = ctx.data.write().await;

    let (volume, prev_message_id) = {
        match volume {
            Some(volume) => (volume, None),
            None => {
                let store = x.get::<Store>().unwrap();
                let history = store.history().find_one(HistoryKind::YouTube, &uid).await?;

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

    // event 발생
    // -> 메세지 보내고 업데이트 하는 방식으로?

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

        sleep(Duration::from_millis(200)).await;

        if track.get_info().await?.playing == PlayMode::Play {
            break;
        }
    }

    log::info!("url = {}", metadata.source_url.as_ref().unwrap());
    log::info!("volume = {}", volume);

    x.insert::<Track>(Track(uid, track));

    let event = Event::Play(*metadata.clone(), volume, user_id, prev_message_id);

    if let Err(err) = event_tx.send((ctx.clone(), event)).await {
        panic!("closed event channel: {}", err)
    }

    Ok((*metadata, volume))
}
