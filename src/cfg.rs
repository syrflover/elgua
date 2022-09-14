use std::{fs::File, io::Read};

use serde::Deserialize;
use serenity::{
    model::id::{ChannelId, GuildId},
    prelude::TypeMapKey,
};

#[derive(Clone, Deserialize)]
pub struct Cfg {
    pub token: String,
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub database_url: String,
    pub toshi_url: String,
    pub youtube_api_key: String,
}

impl Cfg {
    pub fn new() -> Self {
        let mut file = File::open("./cfg.json").unwrap();

        let mut cfg_buf = Vec::new();

        file.read_to_end(&mut cfg_buf).unwrap();

        serde_json::from_slice(&cfg_buf).unwrap()
    }
}

impl Default for Cfg {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeMapKey for Cfg {
    type Value = Cfg;
}
