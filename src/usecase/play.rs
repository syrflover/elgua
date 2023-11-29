use std::{sync::Arc, time::Duration};

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
    audio::{scdl, ytdl, AudioMetadata},
    cfg::Cfg,
    store::{CfgKey, HistoryKind, Store},
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

pub struct Parameter {
    url: String,
    kind: PlayableKind,
    volume: Option<f32>,
    play_count: Option<usize>,
}

impl Parameter {
    pub fn new(
        kind: PlayableKind,
        url: String,
        volume: Option<f32>,
        play_count: Option<usize>,
    ) -> Self {
        Self {
            url,
            kind,
            volume,
            play_count,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlayableKind {
    YouTube,
    SoundCloud,
}

impl From<PlayableKind> for HistoryKind {
    fn from(x: PlayableKind) -> Self {
        match x {
            PlayableKind::YouTube => Self::YouTube,
            PlayableKind::SoundCloud => Self::SoundCloud,
        }
    }
}

pub async fn play(
    ctx: &Context,
    guild_id: GuildId,
    voice_channel_id: ChannelId,
    Parameter {
        url,
        kind,
        volume,
        play_count,
    }: Parameter,
) -> crate::Result<(AudioMetadata, f32, Option<MessageId>)> {
    let handler = get_voice_handler(ctx, guild_id, voice_channel_id).await?;
    let mut handler = handler.lock().await;

    let mut x = ctx.data.write().await;

    let is_repeat = play_count.unwrap_or(1) >= 2;

    let uid = match kind {
        PlayableKind::YouTube => ytdl::parse_vid(url.parse().unwrap()),
        PlayableKind::SoundCloud => {
            let cfg = x.get::<Cfg>().unwrap();
            let store = x.get::<Store>().unwrap();

            let sc_client_id = store.elgua_cfg().get(CfgKey::SoundCloudApiKey).await?;
            let sc_client_id = match sc_client_id.as_deref() {
                Some(r) => r,
                None => &cfg.soundcloud_client_id,
            };

            scdl::get_track(sc_client_id, &url).await?.id.to_string()
        }
    };

    let (volume, prev_message_id) = {
        match volume {
            Some(volume) => (volume, None),
            None => {
                let store = x.get::<Store>().unwrap();
                let history = store.history().find_one(kind.into(), &uid).await?;

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

    let audio_source = {
        let cfg = x.get::<Cfg>().unwrap();
        match kind {
            PlayableKind::YouTube => AudioSource::from_youtube(&cfg.youtube_api_key, &uid).await?,
            PlayableKind::SoundCloud => {
                let cfg = x.get::<Cfg>().unwrap();
                let store = x.get::<Store>().unwrap();

                let sc_client_id = store.elgua_cfg().get(CfgKey::SoundCloudApiKey).await?;
                let sc_client_id = match sc_client_id.as_deref() {
                    Some(r) => r,
                    None => &cfg.soundcloud_client_id,
                };

                AudioSource::from_soundcloud(sc_client_id, &url).await?
            }
        }
    };
    let audio_metadata = audio_source.metadata().clone();

    if is_repeat {
        if let Some(duration) = audio_metadata.duration {
            const M10: u64 = 10 * 60;
            if duration > Duration::from_secs(M10) {
                return Err(crate::error::Error::CustomError(
                    "10분 이상의 음악은 반복 재생할 수 없습니다".to_owned(),
                ));
            }
        } else {
            return Err(crate::error::Error::CustomError(
                "기술적인 문제로 재생 시간을 알 수 없는 음악은 반복 재생할 수 없습니다".to_owned(),
            ));
        }
    }

    let mut source = audio_source.get_source(is_repeat).await?;

    handler.stop();

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

        if let Some(play_count) = play_count {
            track.loop_for(play_count)?;
        }

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
                source = audio_source.get_source(is_repeat).await?;
                track = handler.play_only_source(source);
            }

            _ => {}
        }
    }

    log::info!("url = {}", audio_metadata.url);
    log::info!("volume = {}", volume);

    x.insert::<Track>(Track(audio_metadata.clone(), track));

    Ok((audio_metadata.clone(), volume, prev_message_id))
}
