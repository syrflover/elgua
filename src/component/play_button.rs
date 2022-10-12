use serenity::{builder::CreateButton, model::prelude::component::ButtonStyle};

pub fn create_play_button(uid: &str) -> CreateButton {
    let url = format!("https://youtu.be/{}", uid);

    CreateButton::default()
        .custom_id(format!("play-yt-button-0;{url}"))
        // .emoji(ReactionType::Unicode("▶︎".to_string()))
        .label("재생하기")
        .style(ButtonStyle::Success)
        .to_owned()
}
