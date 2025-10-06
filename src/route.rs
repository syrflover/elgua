use std::str::FromStr;

use serenity::all::{CommandDataOptionValue, Interaction};
use serenity::prelude::Context;

use crate::audio::scdl;
use crate::cfg::Cfg;
use crate::interaction::InteractionExtension;
use crate::store::{CfgKey, Store};

use super::controller;

mod route_constant {
    pub const PING: &str = "ping";
    pub const PLAY: &str = "play";
    pub const VOLUME: &str = "volume";
    pub const STOP: &str = "stop";
    pub const TRACK: &str = "track";
    pub const SEARCH: &str = "search";
    pub const PLAY_FROM_SELECTED_MENU: &str = "play-from-selected-menu";
    pub const PLAY_FROM_CLICKED_BUTTON: &str = "play-from-clicked-button#";

    pub const UPDATE_SC_API_KEY: &str = "sc";

    // pub const DEPRECATED_PLAY_FROM_SELECTED_MENU: &str = "play-yt-select-0";
    pub const DEPRECATED_PLAY_FROM_CLICKED_BUTTON: &str = "play-yt-button-0;";
}

pub enum Route {
    Ping,
    Play,
    Volume,
    Stop,
    Track,
    Search,
    PlayFromSelectedMenu,
    PlayFromClickedButton(String),

    UpdateScApiKey,
}

impl From<Route> for String {
    fn from(val: Route) -> Self {
        use Route::*;

        match val {
            Ping => route_constant::PING,

            Play => route_constant::PLAY,

            Volume => route_constant::VOLUME,

            Stop => route_constant::STOP,

            Track => route_constant::TRACK,

            Search => route_constant::SEARCH,

            PlayFromSelectedMenu => route_constant::PLAY_FROM_SELECTED_MENU,

            PlayFromClickedButton(url) => {
                return format!("{}{url}", route_constant::PLAY_FROM_CLICKED_BUTTON)
            }

            UpdateScApiKey => route_constant::UPDATE_SC_API_KEY,
        }
        .to_owned()
    }
}

impl FromStr for Route {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl TryFrom<&str> for Route {
    type Error = ();

    fn try_from(x: &str) -> Result<Self, Self::Error> {
        use Route::*;

        let r = match x {
            route_constant::PING => Ping,

            route_constant::PLAY => Play,

            route_constant::VOLUME => Volume,

            route_constant::STOP => Stop,

            route_constant::TRACK => Track,

            route_constant::SEARCH => Search,

            route_constant::PLAY_FROM_SELECTED_MENU => PlayFromSelectedMenu,

            route_constant::UPDATE_SC_API_KEY => UpdateScApiKey,

            // route_constant::DEPRECATED_PLAY_FROM_SELECTED_MENU => PlayFromSelectedMenu,
            x if x.starts_with(route_constant::PLAY_FROM_CLICKED_BUTTON) => {
                let (_, url) = x.split_once('#').unwrap();

                PlayFromClickedButton(url.to_string())
            }

            x if x.starts_with(route_constant::DEPRECATED_PLAY_FROM_CLICKED_BUTTON) => {
                let (_, url) = x.split_once(';').unwrap();

                PlayFromClickedButton(url.to_string())
            }

            _ => return Err(()),
        };

        Ok(r)
    }
}

pub async fn route_application_command(
    ctx: &Context,
    interaction: &Interaction,
) -> crate::Result<()> {
    let Interaction::Command(command) = interaction else {
        return Ok(());
    };

    let history_channel_id = ctx
        .data
        .read()
        .await
        .get::<Cfg>()
        .unwrap()
        .history_channel_id;
    let do_interact = command.channel_id != history_channel_id;

    let typing = interaction.channel_id().start_typing(&ctx.http);
    let options = &command.data.options;

    match command.data.name.as_str().try_into().ok() {
        Some(Route::Ping) => {
            interaction.send_message(&ctx.http, "pong").await?;
        }

        Some(Route::Play) => {
            let parameter = controller::play::Parameter::from(options);

            controller::play(ctx, interaction, parameter, do_interact).await?;

            // interaction.defer(&ctx.http).await?;
        }

        Some(Route::Volume) => {
            let parameter = controller::volume::Parameter::from(options);

            controller::volume(ctx, interaction, parameter).await?;
        }

        Some(Route::Stop) => {
            controller::stop(ctx, interaction).await?;
        }

        Some(Route::Track) => {
            controller::track(ctx, interaction).await?;
        }

        Some(Route::Search) => {
            // TODO:
            // 일단 검색엔진 뭐 쓸지 생각좀 하자
            // 1. https://tantivy-search.github.io/examples/basic_search.html
            //
            //
            // 그리고 어떻게 구현할지도 생각하자
            //
            // 버튼으로 페이지 구현 (수동으로 페이지 지정? ㄴㄴ하지말자 그냥 버튼만 ㄱㄱ)
            //
            //
            // 검색결과는 어떻게 보여줄까
            //
            // [제목링크] [재생 버튼]
            // 이런식으로 하면 될 듯?
            //
            // 커스텀 검색어 지정할 수 있도록
        }

        Some(Route::UpdateScApiKey) => {
            let sc_api_key = {
                let opt = &command.data.options.first().unwrap().value;

                match opt {
                    CommandDataOptionValue::String(x) => x.clone(),
                    _ => unreachable!(),
                }
            };

            let x = ctx.data.read().await;
            let store = x.get::<Store>().unwrap();

            let is_valid = scdl::get_track(
                &sc_api_key,
                "https://soundcloud.com/user-675880115/ofdxfd?si=276d5de5c87845e79de2e620e2f4aa40",
            )
            .await
            .is_ok();

            if is_valid {
                store
                    .elgua_cfg()
                    .add_or_update(CfgKey::SoundCloudApiKey, sc_api_key)
                    .await?;

                interaction
                    .send_ephemeral_message(&ctx.http, "업데이트 성공")
                    .await?;
            } else {
                interaction
                    .send_ephemeral_message(&ctx.http, "업데이트 실패")
                    .await?;
            }
        }

        _ => {}
    };

    typing.stop();

    Ok(())
}

pub async fn route_message_component(
    ctx: &Context,
    interaction: &Interaction,
) -> crate::Result<()> {
    let Interaction::Component(component) = interaction else {
        return Ok(());
    };

    let history_channel_id = ctx
        .data
        .read()
        .await
        .get::<Cfg>()
        .unwrap()
        .history_channel_id;
    let do_interact = component.message.channel_id != history_channel_id;

    let typing = interaction.channel_id().start_typing(&ctx.http);

    match component.data.custom_id.as_str().try_into().ok() {
        Some(Route::PlayFromSelectedMenu) => {
            let parameter = controller::play::Parameter::from(&component.data);

            // SelectedMenu는 무조건 지워야함. history 채널 여부 상관 없음
            component.message.delete(&ctx.http).await.ok();

            controller::play(ctx, interaction, parameter, do_interact).await?;
        }

        Some(Route::PlayFromClickedButton(url)) => {
            let parameter = controller::play::Parameter::from(url);

            // history 채널이 아니면 이전 메세지 삭제함
            if do_interact {
                interaction.message().unwrap().delete(&ctx.http).await.ok();
            }

            controller::play(ctx, interaction, parameter, do_interact).await?;

            // if do_interact {
            //     // history채널이 아닐때만 이전에 생성된 play button 메세지를 삭제함
            //     // interaction.message().unwrap().delete(&ctx.http).await.ok();
            //     controller::play(ctx, interaction, parameter).await?;
            // } else {
            //     // history채널일 경우 여기서부터는 더이상 상호작용을 하지 않을 것이기 때문에 끝냄
            //     // interaction.defer(&ctx.http).await?;
            // }
        }

        _ => {}
    }

    typing.stop();

    Ok(())
}
