use serenity::prelude::TypeMapKey;
use sqlx::{postgres::PgPoolOptions, PgPool};

use self::history::HistoryStore;

mod history;

pub use history::{History, HistoryKind};

pub struct Store {
    connection: PgPool,
}

impl Store {
    pub async fn connect(url: &str) -> Self {
        let pg_pool = PgPoolOptions::new().connect(url).await.expect("connect pg");

        HistoryStore::init(&pg_pool).await;

        Self {
            connection: pg_pool,
        }
    }

    pub fn history(&self) -> HistoryStore {
        HistoryStore::new(self.connection.clone())
    }
}

impl TypeMapKey for Store {
    type Value = Store;
}
