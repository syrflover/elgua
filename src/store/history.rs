use chrono::{DateTime, Utc};
use sqlx::{postgres::PgQueryResult, PgPool, Row};

use super::search;

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct History {
    pub id: u64,
    pub title: String,
    pub channel: String,
    pub kind: HistoryKind,
    // unique하게 만들 방법
    // TODO:
    pub url: String,
    pub user_id: u64,
    pub volume: u8,
    pub created_at: DateTime<Utc>,
}

impl From<History> for search::History {
    fn from(x: History) -> Self {
        Self {
            id: x.id,
            title: x.title,
            kind: x.kind.as_str().to_string(),
            channel: x.channel,
            user_id: x.user_id,
            created_at: x.created_at.to_rfc3339(),
        }
    }
}

impl From<HistoryRow> for History {
    fn from(x: HistoryRow) -> Self {
        Self {
            id: x.id as u64,
            title: x.title,
            channel: x.channel,
            kind: x.kind.into(),
            url: x.url,
            user_id: x.user_id as u64,
            volume: x.volume as u8,
            created_at: x.created_at,
        }
    }
}

pub struct HistoryStore {
    conn: PgPool,
    search: search::HistoryStore,
}

impl HistoryStore {
    pub(super) async fn init(conn: &PgPool) {
        let _r: PgQueryResult = sqlx::query!(
            r#"CREATE TABLE IF NOT EXISTS history
            (
                id bigserial PRIMARY KEY,
                title varchar NOT NULL,
                channel varchar NOT NULL,
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

    pub(super) fn new(conn: PgPool, toshi: toshi::ToshiClient) -> Self {
        Self {
            conn,
            search: search::HistoryStore::new(toshi),
        }
    }

    pub async fn add(&self, history: &History) -> crate::Result<u64> {
        let mut conn = self.conn.begin().await?;

        let r = sqlx::query(
            r#"
            INSERT INTO history (title, channel, kind, url, user_id, volume, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(history.title.as_str())
        .bind(history.channel.as_str())
        .bind(history.kind.as_str())
        .bind(history.url.as_str())
        .bind(history.user_id as i64)
        .bind(history.volume as i16)
        .bind(history.created_at)
        .fetch_one(&mut conn)
        .await?;

        let id: i64 = r.try_get("id")?;

        {
            let x = History {
                id: id as u64,
                ..history.clone()
            };

            self.search.add(&x.into()).await?;
        }

        conn.commit().await?;

        Ok(id as u64)
    }

    // TODO: 여러가지
    // 1. 노래들 중복 여부
    // 2. 특정 유저
    // 3. 특정 채널
    // 4. 특정 kind

    pub async fn get(
        &self,
        page: usize,
        per_page: usize,
        than: Option<Than>,
    ) -> sqlx::Result<Vec<History>> {
        let sql = match than {
            Some(than) => {
                let r#where = match than {
                    Than::More(id) => format!("id > {id}"),
                    Than::Less(id) => format!("id < {id}"),
                };
                format!(
                    r#"
                SELECT id, title, channel, kind, url, user_id, volume, created_at FROM history
                WHERE {}
                ORDER BY created_at DESC
                OFFSET $1
                LIMIT $2"#,
                    r#where
                )
            }

            None => r#"
            SELECT id, title, channel, kind, url, user_id, volume, created_at FROM history
            ORDER BY created_at DESC
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
            .map(|x: HistoryRow| x.into())
            .collect();

        Ok(histories)
    }

    pub async fn find_one(
        &self,
        kind: HistoryKind,
        url: impl AsRef<str>,
    ) -> sqlx::Result<Option<History>> {
        let sql = r#"
            SELECT * FROM history
            WHERE kind = $1 AND
                  url = $2
            ORDER BY id DESC
            LIMIT 1
        "#;

        let history = sqlx::query_as(sql)
            .bind(kind.as_str())
            .bind(url.as_ref())
            .fetch_optional(&self.conn)
            .await?
            .map(|x: HistoryRow| x.into());

        Ok(history)
    }

    pub async fn update_volume(&self, id: u64, volume: u8) -> sqlx::Result<()> {
        let sql = r#"
            UPDATE history
            SET volume = $1
            WHERE id = $2
        "#;

        sqlx::query(sql)
            .bind(volume as i16)
            .bind(id as i64)
            .execute(&self.conn)
            .await?;

        Ok(())
    }

    pub async fn search() {}
}

#[derive(sqlx::FromRow)]
struct HistoryRow {
    id: i64,
    title: String,
    channel: String,
    kind: String,
    url: String,
    user_id: i64,
    volume: i16,
    created_at: DateTime<Utc>,
}

pub enum Than {
    More(u64),
    Less(u64),
}
