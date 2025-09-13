use serenity::{
    all::EditMessage,
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

            if let Some(message_id) = history.message_id.map(MessageId::new) {
                if let Ok(mut message) = ctx.http.get_message(history_channel_id, message_id).await
                {
                    let mut embed = message.embeds.first().cloned().unwrap();

                    for field in &mut embed.fields {
                        if field.name == "소리 크기" {
                            field.value = volume_u8.to_string();
                        }
                    }

                    if let Err(err) = message
                        .edit(&ctx.http, EditMessage::new().embed(embed.into()))
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
                .push(volume_u8.to_string())
                .build());
        }
    }

    Ok("재생 중인 음악이 없어요".to_string())
}
