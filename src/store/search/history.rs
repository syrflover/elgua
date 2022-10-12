use futures::AsyncReadExt;
use serde::{Deserialize, Serialize};
use tantivy::schema::*;
use toshi::{AsyncClient, IndexOptions, ToshiClient};

use crate::error::ToshiError;

const HISTORY_INDEX_NAME: &str = "history";

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    pub id: u64,
    pub title: String,
    pub kind: String,
    pub channel: String,
    pub user_id: u64,
    /// i64 UTC Timestamp
    pub created_at: String,
}

pub struct HistoryStore {
    client: ToshiClient,
}

impl HistoryStore {
    pub(crate) async fn init<'c>(url: &str) {
        let mut schema = Schema::builder();

        schema.add_u64_field("id", NumericOptions::default().set_stored());
        schema.add_text_field(
            "title",
            TextOptions::default()
                .set_indexing_options(
                    TextFieldIndexing::default()
                        .set_tokenizer("lang_ko")
                        .set_index_option(IndexRecordOption::Basic),
                )
                .set_stored(),
        );
        schema.add_text_field(
            "kind",
            TextOptions::default()
                .set_indexing_options(
                    TextFieldIndexing::default()
                        .set_tokenizer("lang_ko")
                        .set_index_option(IndexRecordOption::Basic),
                )
                .set_stored(),
        );
        schema.add_text_field(
            "channel",
            TextOptions::default()
                .set_indexing_options(
                    TextFieldIndexing::default()
                        .set_tokenizer("lang_ko")
                        .set_index_option(IndexRecordOption::Basic),
                )
                .set_stored(),
        );
        schema.add_u64_field(
            "user_id",
            NumericOptions::default().set_indexed().set_stored(),
        );
        schema.add_date_field(
            "created_at",
            NumericOptions::default().set_indexed().set_stored(),
        );

        let client = ToshiClient::new(url);

        client
            .create_index(HISTORY_INDEX_NAME, schema.build())
            .await
            .unwrap();
    }

    pub(crate) fn new(client: ToshiClient) -> Self {
        Self { client }
    }

    pub async fn add(&self, doc: &History) -> crate::Result<()> {
        let index_options = IndexOptions { commit: true };

        let mut resp = self
            .client
            .add_document(HISTORY_INDEX_NAME, doc, Some(index_options))
            .await?;

        let status = resp.status();

        log::debug!("status = {}", status);

        if resp.status().as_u16() == 201 {
            Ok(())
        } else {
            let mut x = Vec::new();
            resp.body_mut().read_to_end(&mut x).await.unwrap();

            let mut x: crate::error::ToshiError = serde_json::from_slice(&x).unwrap_or_else(|_e| {
                let a = String::from_utf8(x).unwrap_or_else(|_e| "unknown error".to_string());
                ToshiError {
                    status: 0,
                    message: a,
                }
            });
            x.status = status.as_u16();

            log::error!("add: {x}");

            Err(x.into())
        }
    }

    // 1. title
    pub async fn _search(&self) -> crate::Result<()> {
        todo!()
    }
}
