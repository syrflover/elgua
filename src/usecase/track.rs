use serenity::{
    prelude::Context,
    utils::{EmbedMessageBuilding, MessageBuilder},
};
use songbird::tracks::{LoopState, PlayMode, TrackState};

use crate::{track::Track, util::time::seperate_duration};

pub async fn track(ctx: &Context) -> crate::Result<String> {
    let x = ctx.data.read().await;

    if let Some(Track(audio_metadata, track)) = x.get::<Track>() {
        let play_info = track.get_info().await.ok();

        let play_state = play_info.map(|x| x.playing).unwrap_or(PlayMode::End);

        if let PlayMode::Play | PlayMode::Pause = play_state {
            let TrackState {
                volume,
                loops,
                position,
                play_time,
                ..
            } = play_info.unwrap();

            let position = seperate_duration(position);
            let position_with_looped = seperate_duration(play_time);

            let mut r = MessageBuilder::new()
                .push_named_link(&audio_metadata.title, &audio_metadata.url)
                .push("\n소리 크기: ")
                .push((volume * 100.0) as u8)
                .push("\n재생 시간: ")
                .push(position)
                .push(" - ")
                .push(position_with_looped)
                .to_owned();

            // match &audio_metadata.duration {
            //     Either::Left(x) => {
            //         let total = seperate_duration(*x);

            //         r.push("\n").push(position).push(" / ").push(total);
            //     }
            //     Either::Right(x) => {
            //         r.push("\n").push(position).push(" / ").push(x);
            //     }
            // }

            match loops {
                LoopState::Finite(remaining_play_count) => {
                    r.push("\n재생 횟수: ").push(remaining_play_count);
                    // .push(" / ")
                    // .push(content);
                }

                LoopState::Infinite => {
                    r.push("\n재생 횟수: 평생");
                }
            }

            return Ok(r.build());
        }
    }

    Ok("재생 중인 음악이 없어요".to_string())
}
