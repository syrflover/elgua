use std::ops::Deref;

use chrono::Utc;
use serenity::{
    builder::{CreateActionRow, CreateComponents, CreateEmbed, CreateEmbedAuthor},
    model::id::{MessageId, UserId},
    prelude::{Context, TypeMapKey},
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    cfg::Cfg,
    component::create_play_button,
    store::{History, HistoryKind, Store},
    ytdl,
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
    Play(youtube_dl::SingleVideo, f32, UserId, Option<MessageId>),
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
        Event::Play(metadata, volume, user_id, prev_message_id) => {
            let x = ctx.data.read().await;

            let uid = ytdl::parse_vid(metadata.webpage_url.unwrap().parse().unwrap());
            let now = Utc::now();

            // 1. delete prev message
            if let Some(prev_message_id) = prev_message_id {
                if let Err(err) = ctx
                    .http
                    .delete_message(history_channel_id.0, prev_message_id.0)
                    .await
                {
                    log::error!("{err}");
                }
            }

            // 2. send message
            let message = {
                let user = user_id.to_user(&ctx.http).await?;

                let author = {
                    let mut x = CreateEmbedAuthor::default().name(&user.name).to_owned();
                    if let Some(avatar_url) = user.avatar_url() {
                        x.icon_url(avatar_url);
                    }
                    x
                };

                let embed = {
                    let mut x = CreateEmbed::default()
                        .set_author(author)
                        .title(metadata.title.as_str())
                        .field("채널", metadata.channel.as_ref().unwrap(), true)
                        .field("소리 크기", (volume * 100.0) as u8, true)
                        .url(format!("https://youtu.be/{}", uid))
                        .timestamp(now)
                        .to_owned();

                    if let Some(image_url) = metadata.thumbnail.as_ref() {
                        x.image(image_url);
                    };
                    x
                };

                let play_button = create_play_button(&uid);
                let action_row = CreateActionRow::default()
                    .add_button(play_button)
                    .to_owned();
                let components = CreateComponents::default()
                    .add_action_row(action_row)
                    .to_owned();

                history_channel_id
                    .send_message(&ctx.http, |message| {
                        message.set_embed(embed).set_components(components)
                    })
                    .await
                    .ok()
            };

            // 3. add or update db
            {
                let store = x.get::<Store>().unwrap();

                let _history_id = {
                    let history = History {
                        id: 0,
                        message_id: message.map(|x| x.id.0),
                        title: metadata.title.clone(),
                        channel: metadata.channel.clone().unwrap(),
                        kind: HistoryKind::YouTube,
                        uid,
                        user_id: user_id.0,
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
