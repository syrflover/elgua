use std::ops::Deref;

use chrono::Utc;
use serenity::{
    all::CreateMessage,
    builder::{CreateEmbed, CreateEmbedAuthor},
    model::{id::UserId, prelude::MessageId},
    prelude::{Context, TypeMapKey},
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    audio::{AudioMetadata, AudioSourceKind},
    cfg::Cfg,
    component::create_play_button,
    route::Route,
    store::{History, Store},
};

#[derive(Debug, Clone)]
pub struct EventSender(Sender<(Context, Event)>);

impl EventSender {
    pub fn new(tx: Sender<(Context, Event)>) -> Self {
        Self(tx)
    }
}

impl TypeMapKey for EventSender {
    type Value = EventSender;
}

impl Deref for EventSender {
    type Target = Sender<(Context, Event)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    /// metadata, volume
    Play(AudioMetadata, f32, UserId, Option<MessageId>),
}

pub async fn process(mut rx: Receiver<(Context, Event)>) {
    while let Some((ctx, event)) = rx.recv().await {
        if let Err(err) = handle(ctx, event).await {
            log::error!("{err}");
        }
    }
}

async fn handle(ctx: Context, event: Event) -> crate::Result<()> {
    let Cfg {
        history_channel_id, ..
    } = {
        let x = ctx.data.read().await;
        x.get::<Cfg>().cloned().unwrap()
    };

    match event {
        // 재생하고나서 history channel에 매세지 보냄
        Event::Play(audio_metadata, volume, user_id, prev_message_id) => {
            let x = ctx.data.read().await;

            let kind = audio_metadata.kind();
            let url = audio_metadata.url;
            let uid = audio_metadata.id;
            let color = match kind {
                AudioSourceKind::YouTube => 0xFF0000,
                AudioSourceKind::SoundCloud => 0xF26F23,
            };
            let now = Utc::now();

            // 1. delete prev message
            if let Some(prev_message_id) = prev_message_id {
                let _result_of_deleted_message = ctx
                    .http
                    .delete_message(history_channel_id, prev_message_id, None)
                    .await;
            }

            // 2. send message
            let message = {
                let user = user_id.to_user(&ctx.http).await?;

                let author = {
                    let mut x = CreateEmbedAuthor::new(&user.name).to_owned();
                    if let Some(avatar_url) = user.avatar_url() {
                        x = x.icon_url(avatar_url);
                    }
                    x
                };

                let mut embed = CreateEmbed::default()
                    .author(author)
                    .title(audio_metadata.title.as_str())
                    .field("채널", &audio_metadata.uploaded_by, true)
                    .field("소리 크기", (volume * 100.0).to_string(), true)
                    .url(&url)
                    .timestamp(now)
                    .color(color);

                if let Some(thumbnail_url) = audio_metadata.thumbnail_url {
                    embed = embed.image(thumbnail_url);
                }

                let play_button = create_play_button(Route::PlayFromClickedButton(url));

                history_channel_id
                    .send_message(
                        &ctx.http,
                        CreateMessage::new().embed(embed).button(play_button),
                    )
                    .await
                    .ok()
            };

            // 3. add or update db
            {
                let store = x.get::<Store>().unwrap();

                let _history_id = {
                    let history = History {
                        id: 0,
                        message_id: message.map(|x| x.id.get()),
                        title: audio_metadata.title.clone(),
                        channel: audio_metadata.uploaded_by,
                        kind: kind.into(),
                        uid,
                        user_id: user_id.get(),
                        volume: (volume * 100.0) as u8,
                        created_at: now,
                    };

                    store.history().add_or_update(&history).await?
                };
            }
        }
    }

    Ok(())
}
