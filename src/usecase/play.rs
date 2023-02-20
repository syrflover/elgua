use std::sync::Arc;

use serenity::{
    model::id::{ChannelId, GuildId, MessageId},
    prelude::{Context, Mutex},
};
use songbird::{
    error::JoinError,
    tracks::{PlayMode, TrackError},
    Call,
};

use crate::{
    audio::AudioSource,
    audio::{ytdl, AudioMetadata},
    cfg::Cfg,
    store::{HistoryKind, Store},
    track::Track,
};

async fn get_voice_handler(
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
    url: &str,
    volume: Option<f32>,
) -> crate::Result<(AudioMetadata, f32, Option<MessageId>)> {
    let handler = get_voice_handler(ctx, guild_id, voice_channel_id).await?;
    let mut handler = handler.lock().await;

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
    let audio_metadata = audio_source.metadata().unwrap().clone();

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

    log::info!("url = {}", audio_metadata.url);
    log::info!("volume = {}", volume);

    x.insert::<Track>(Track(uid, track));

    Ok((audio_metadata.clone(), volume, prev_message_id))
}
