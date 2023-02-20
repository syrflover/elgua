use serenity::{builder::CreateButton, model::prelude::component::ButtonStyle};

pub fn create_play_button(custom_id: impl ToString) -> CreateButton {
    // let url = format!("https://youtu.be/{}", uid.as_ref());

    CreateButton::default()
        .custom_id(custom_id)
        // .emoji(ReactionType::Unicode("▶︎".to_string()))
        .label("재생하기")
        // .url(url)
        .style(ButtonStyle::Success)
        .to_owned()
}
