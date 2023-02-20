use serenity::{builder::CreateSelectMenu, model::prelude::ReactionType};

/// limits: 1 - 10
fn num_emoji<'a>(n: usize) -> &'a str {
    match n {
        1 => "1️⃣",
        2 => "2️⃣",
        3 => "3️⃣",
        4 => "4️⃣",
        5 => "5️⃣",
        6 => "6️⃣",
        7 => "7️⃣",
        8 => "8️⃣",
        9 => "9️⃣",
        10 => "🔟",
        _ => unreachable!(),
    }
}

fn truncate(x: String, len: usize) -> String {
    if x.chars().count() > len {
        x.chars()
            .take(len - 4)
            .chain(" ...".chars())
            .collect::<String>()
    } else {
        x
    }
}

///
/// about of select menu items
///
/// label: 사용자에게 보여지는 단일 메뉴의 label \
/// description: 단일 메뉴의 description \
/// value: 해당 단일 메뉴를 선택했을 때 봇에서 처리할 때 필요한 value
///
/// 아이템은 10개까지만 불러옴
pub fn create_numbering_select_menu(
    custom_id: impl ToString,
    placeholder: impl ToString,
    // limit: usize,
    select_menu_items: impl Iterator<Item = (String, String, String)>,
) -> CreateSelectMenu {
    let ord_numbers = (1..=10).map(num_emoji);
    let xs = select_menu_items
        .into_iter()
        .take(10)
        .map(|(label, description, value)| {
            (truncate(label, 100), truncate(description, 100), value)
        })
        .zip(ord_numbers);

    CreateSelectMenu::default()
        .min_values(1) // 사용자가 선택할 수 있는 메뉴의 갯수를 설정하는 메서드
        .max_values(1)
        .placeholder(placeholder)
        .custom_id(custom_id)
        .options(|x| {
            xs.fold(x, |acc, ((label, description, value), ord_number)| {
                acc.create_option(|opt| {
                    opt.label(label)
                        .value(value)
                        .emoji(ReactionType::Unicode(ord_number.to_string()))
                        .description(description)
                })
            })
        })
        .to_owned()
}
