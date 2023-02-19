use std::sync::Arc;

use serenity::{
    model::id::{ChannelId, GuildId, MessageId, UserId},
    prelude::{Context, Mutex},
};
use songbird::{
    error::JoinError,
    tracks::{PlayMode, TrackError},
    Call,
};
use tokio::sync::mpsc::Sender;

use crate::{
    audio::AudioSource,
    cfg::Cfg,
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
) -> crate::Result<(youtube_dl::SingleVideo, f32)> {
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

    handler.stop();

    let audio_source = {
        let cfg = x.get::<Cfg>().unwrap();
        AudioSource::from_youtube(&uid, cfg.youtube_account(), &cfg.youtube_api_key).await?
    };
    let audio_metadata = audio_source.youtube_metadata().unwrap().clone();

    let mut source = audio_source.get_source().await?;
    let mut track = handler.play_only_source(source);

    let mut try_count = 0;

    loop {
        let play_state;

        if try_count > 3 {
            return Err(crate::error::Error::CustomError(
                "not played this track".to_string(),
            ));
        }

        log::debug!("try_count = {try_count}");

        try_count += 1;

        let play_result = [track.set_volume(volume), track.play()]
            .into_iter()
            .collect::<Result<(), _>>();

        if let Err(TrackError::Finished) = play_result {
            play_state = PlayMode::End;
        } else {
            // sleep(Duration::from_millis(100)).await;

            play_state = track
                .get_info()
                .await
                .map(|x| x.playing)
                .unwrap_or(PlayMode::End);
        }

        match play_state {
            PlayMode::Play => break,

            PlayMode::End => {
                source = audio_source.get_source().await?;
                track = handler.play_only_source(source);
            }

            _ => {}
        }
    }

    log::info!("url = {}", audio_metadata.webpage_url.as_ref().unwrap());
    log::info!("volume = {}", volume);

    x.insert::<Track>(Track(uid, track));

    let event = Event::Play(audio_metadata.clone(), volume, user_id, prev_message_id);

    if let Err(err) = event_tx.send((ctx.clone(), event)).await {
        panic!("closed event channel: {}", err)
    }

    Ok((audio_metadata, volume))
}
