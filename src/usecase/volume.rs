use serenity::{
    model::id::{ChannelId, MessageId},
    prelude::*,
    utils::MessageBuilder,
};
use songbird::tracks::PlayMode;

use crate::{store::Store, track::Track};

pub async fn volume(
    ctx: &Context,
    history_channel_id: ChannelId,
    volume: f32,
) -> crate::Result<String> {
    let x = ctx.data.read().await;

    if let Some(Track(audio_metadata, track)) = x.get::<Track>() {
        let play_state = track
            .get_info()
            .await
            .map(|x| x.playing)
            .unwrap_or(PlayMode::End);

        if let PlayMode::Play | PlayMode::Pause = play_state {
            track.set_volume(volume)?;

            let store = x.get::<Store>().unwrap();

            let volume_u8 = (volume * 100.0) as u8;

            let history = store
                .history()
                .find_one(audio_metadata.kind().into(), &audio_metadata.id)
                .await?
                .unwrap();

            if let Some(message_id) = history.message_id.map(MessageId) {
                if let Ok(mut message) = ctx
                    .http
                    .get_message(history_channel_id.0, message_id.0)
                    .await
                {
                    let mut embed = message.embeds.get(0).cloned().unwrap();

                    for field in &mut embed.fields {
                        if field.name == "소리 크기" {
                            field.value = volume_u8.to_string();
                        }
                    }

                    if let Err(err) = message
                        .edit(&ctx.http, |message| message.set_embed(embed.into()))
                        .await
                    {
                        log::error!("{err}");
                    }
                }
            }

            store
                .history()
                .update_volume(audio_metadata.kind().into(), &audio_metadata.id, volume_u8)
                .await?;

            return Ok(MessageBuilder::new()
                .push("소리 크기: ")
                .push(volume_u8)
                .build());
        }
    }

    Ok("재생 중이 아니에요".to_string())
}
