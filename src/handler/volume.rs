use serenity::{prelude::*, utils::MessageBuilder};
use songbird::tracks::PlayMode;

use crate::store::Store;

use super::Track;

pub async fn volume(ctx: &Context, volume: f32) -> crate::Result<String> {
    let x = ctx.data.read().await;

    if let Some(Track(history_id, track)) = x.get::<Track>() {
        if let PlayMode::Play | PlayMode::Pause = track.get_info().await.unwrap().playing {
            track.set_volume(volume)?;

            let store = x.get::<Store>().unwrap();

            let volume_u8 = (volume * 100.0) as u8;

            store
                .history()
                .update_volume(*history_id, volume_u8)
                .await?;

            return Ok(MessageBuilder::new()
                .push("소리 크기: ")
                .push(volume_u8)
                .build());
        }
    }

    Ok("재생 중이 아니에요".to_string())
}
