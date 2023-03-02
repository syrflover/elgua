use serenity::prelude::Context;
use songbird::tracks::PlayMode;

use crate::track::Track;

pub async fn stop(ctx: &Context) -> crate::Result<String> {
    let x = ctx.data.read().await;

    if let Some(Track(_, track)) = x.get::<Track>() {
        let play_state = track
            .get_info()
            .await
            .map(|x| x.playing)
            .unwrap_or(PlayMode::End);

        if let PlayMode::Play | PlayMode::Pause = play_state {
            track.stop()?;

            return Ok("재생 중인 음악이 중지되었어요".to_string());
        }
    }

    Ok("재생 중인 음악이 없어요".to_string())
}
