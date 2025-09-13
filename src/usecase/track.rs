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

        let play_state = play_info
            .as_ref()
            .map(|x| x.playing.clone())
            .unwrap_or(PlayMode::End);

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
                .push((volume * 100.0).to_string())
                .push("\n재생 시간: ")
                .push(position.to_string())
                .to_owned();

            if position != position_with_looped {
                r.push(" - ").push(position_with_looped.to_string());
            }

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
                    r.push("\n남은 재생 횟수: ")
                        .push(remaining_play_count.to_string());
                    // .push(" / ")
                    // .push(content);
                }

                LoopState::Infinite => {
                    r.push("\n남은 재생 횟수: 평생");
                }
            }

            return Ok(r.build());
        }
    }

    Ok("재생 중인 음악이 없어요".to_string())
}
