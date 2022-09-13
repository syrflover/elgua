use serenity::{prelude::*, utils::MessageBuilder};
use songbird::tracks::PlayMode;

use super::Track;

pub async fn volume(ctx: &Context, volume: f32) -> crate::Result<String> {
    let x = ctx.data.read().await;

    if let Some(Track(track)) = x.get::<Track>() {
        if let PlayMode::Play | PlayMode::Pause = track.get_info().await.unwrap().playing {
            track.set_volume(volume)?;

            return Ok(MessageBuilder::new()
                .push("소리 크기: ")
                .push((volume * 100.0) as u8)
                .build());
        }
    }

    Ok("재생 중이 아니에요".to_string())
}
