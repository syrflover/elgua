use serenity::prelude::TypeMapKey;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::cfg::Cfg;

use self::history::HistoryStore;

mod history;
// mod search;
// mod track_queue;

pub use history::{History, HistoryKind};

pub struct Store {
    connection: PgPool,
    // toshi_url: String,
}

impl Store {
    pub async fn connect(cfg: &Cfg) -> Self {
        let pg_pool = PgPoolOptions::new()
            .connect(&cfg.database_url)
            .await
            .expect("connect pg");

        HistoryStore::init(&pg_pool).await;
        // search::HistoryStore::init(&cfg.toshi_url).await;

        Self {
            connection: pg_pool,
            // toshi_url: cfg.toshi_url.clone(),
        }
    }

    pub fn history(&self) -> HistoryStore {
        // let toshi = toshi::ToshiClient::new(&self.toshi_url);
        HistoryStore::new(self.connection.clone() /* , toshi */)
    }
}

impl TypeMapKey for Store {
    type Value = Store;
}
