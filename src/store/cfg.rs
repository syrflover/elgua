use sqlx::{postgres::PgQueryResult, PgPool, Row};

#[derive(Debug, Clone, Copy)]
pub enum CfgKey {
    SoundCloudApiKey,
}

impl From<String> for CfgKey {
    fn from(x: String) -> Self {
        match x.as_str() {
            "soundcloudapikey" => Self::SoundCloudApiKey,
            _ => unreachable!(),
        }
    }
}

impl CfgKey {
    pub fn as_str(&self) -> &str {
        use CfgKey::*;

        match self {
            SoundCloudApiKey => "soundcloudapikey",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ElguaCfg {
    pub key: CfgKey,
    pub value: String,
}

pub struct CfgStore {
    conn: PgPool,
}

impl CfgStore {
    pub(super) async fn init(conn: &PgPool) {
        let _r: PgQueryResult = sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS elgua_cfg
            (
                key varchar PRIMARY KEY,
                value varchar NOT NULL
            )"#,
        )
        .execute(conn)
        .await
        .expect("create table elgua_cfg");
    }

    pub(super) fn new(conn: PgPool) -> Self {
        Self { conn }
    }

    pub async fn add_or_update(&self, key: CfgKey, value: impl AsRef<str>) -> crate::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO elgua_cfg (key, value)
            VALUES ($1, $2)
            ON CONFLICT (key)
            DO UPDATE
                SET value = $2
        "#,
        )
        .bind(key.as_str())
        .bind(value.as_ref())
        .execute(&self.conn)
        .await?;

        Ok(())
    }

    pub async fn get(&self, key: CfgKey) -> crate::Result<Option<String>> {
        let r = sqlx::query(
            r#"
            SELECT * FROM elgua_cfg
            WHERE key = $1
        "#,
        )
        .bind(key.as_str())
        .fetch_optional(&self.conn)
        .await?;

        Ok(r.and_then(|x| x.try_get("value").ok()))
    }
}
