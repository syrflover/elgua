use serenity::{all::ButtonStyle, builder::CreateButton};

pub fn create_play_button(custom_id: impl Into<String>) -> CreateButton {
    // let url = format!("https://youtu.be/{}", uid.as_ref());

    CreateButton::new(custom_id)
        // .emoji(ReactionType::Unicode("▶︎".to_string()))
        .label("재생하기")
        // .url(url)
        .style(ButtonStyle::Success)
        .to_owned()
}
