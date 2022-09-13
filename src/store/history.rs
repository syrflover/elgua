use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, PgPool};

pub enum HistoryKind {
    YouTube,
    SoundCloud,
}

impl From<String> for HistoryKind {
    fn from(x: String) -> Self {
        match x.as_str() {
            "youtube" => Self::YouTube,
            "soundcloud" => Self::SoundCloud,
            _ => unreachable!(),
        }
    }
}

impl HistoryKind {
    pub fn as_str(&self) -> &str {
        use HistoryKind::*;

        match self {
            YouTube => "youtube",
            SoundCloud => "soundcloud",
        }
    }
}

pub struct History {
    pub kind: HistoryKind,
    pub url: String,
    pub user_id: u64,
    pub volume: u8,
    pub created_at: DateTime<Utc>,
}

pub struct HistoryStore {
    conn: PgPool,
}

impl HistoryStore {
    pub(super) async fn init(conn: &PgPool) {
        let _r: PgQueryResult = sqlx::query!(
            r#"CREATE TABLE IF NOT EXISTS history
            (
                id bigserial PRIMARY KEY,
                kind varchar NOT NULL,
                url varchar NOT NULL,
                user_id bigint NOT NULL,
                volume smallint NOT NULL,
                created_at timestamptz NOT NULL
            )"#,
        )
        .execute(conn)
        .await
        .expect("create table history");
    }

    pub(super) fn new(conn: PgPool) -> Self {
        Self { conn }
    }

    pub async fn add(&self, history: &History) -> sqlx::Result<()> {
        let _r = sqlx::query(
            r#"
            INSERT INTO history (kind, url, user_id, volume, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(history.kind.as_str())
        .bind(history.url.as_str())
        .bind(history.user_id as i64)
        .bind(history.volume as i16)
        .bind(history.created_at)
        .execute(&self.conn)
        .await?;

        Ok(())
    }

    pub async fn get(
        &self,
        page: usize,
        per_page: usize,
        than: Option<Than>,
    ) -> sqlx::Result<Vec<History>> {
        #[derive(sqlx::FromRow)]
        struct HistoryRow {
            // id: i64,
            kind: String,
            url: String,
            user_id: i64,
            volume: i16,
            created_at: DateTime<Utc>,
        }

        let sql = match than {
            Some(than) => {
                let r#where = match than {
                    Than::More(id) => format!("id > {id}"),
                    Than::Less(id) => format!("id < {id}"),
                };
                format!(
                    r#"
                SELECT DISTINCT(id), kind, url, user_id, volume, created_at FROM history
                WHERE {}
                ORDER BY created_at ASC
                OFFSET $1
                LIMIT $2"#,
                    r#where
                )
            }

            None => r#"
            SELECT DISTINCT(id), kind, url, user_id, volume, created_at FROM history
            ORDER BY created_at ASC
            OFFSET $1
            LIMIT $2
            "#
            .to_string(),
        };

        let histories: Vec<History> = sqlx::query_as(&sql)
            .bind(per_page as i32)
            .bind(((page - 1) * per_page) as i32)
            .fetch_all(&self.conn)
            .await?
            .into_iter()
            .map(|x: HistoryRow| History {
                kind: x.kind.into(),
                url: x.url,
                user_id: x.user_id as u64,
                volume: x.volume as u8,
                created_at: x.created_at,
            })
            .collect();

        Ok(histories)
    }
}

pub enum Than {
    More(u64),
    Less(u64),
}
