use std::{collections::HashMap, fmt::Display};

use http::Uri;
use serde::Deserialize;

use crate::audio::AudioMetadata;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub total_results: usize,
    pub results_per_page: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    // pub kind: String,
    // pub etag: String,
    // pub next_page_token: Option<String>,
    // pub region_code: String,
    // pub page_info: PageInfo,
    pub items: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoResult {
    // pub kind: String,
    // pub etag: String,
    // pub next_page_token: Option<String>,
    // pub region_code: String,
    // pub page_info: PageInfo,
    pub items: Vec<VideoItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchItem {
    // pub kind: String,
    // pub etag: String,
    pub id: SearchItemId,
    pub snippet: Snippet,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoItem {
    pub id: String,
    pub snippet: Snippet,
    // pub file_details: FileDetail,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchItemId {
    // pub kind: String,
    pub video_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    pub url: String,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDetail {
    pub duration_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub published_at: String,
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub thumbnails: HashMap<String, Thumbnail>,
    pub channel_title: String,
    // pub live_broadcast_content: String,
    // pub publish_time: String,
}

#[derive(Debug, Deserialize)]
pub struct Error {
    pub code: u16,
    pub message: String,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error {
    pub fn from_slice(xs: &[u8]) -> serde_json::Result<Self> {
        let x: YouTuneErrorWrapper = serde_json::from_slice(xs)?;

        Ok(x.error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(x: reqwest::Error) -> Self {
        Error {
            code: 0,
            message: x.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct YouTuneErrorWrapper {
    pub error: Error,
}

pub async fn get(
    youtube_api_key: impl AsRef<str>,
    id: impl AsRef<str>,
) -> Result<AudioMetadata, Error> {
    let params = [
        ("part", "snippet"),
        ("type", "video"),
        ("key", youtube_api_key.as_ref()),
        ("id", id.as_ref()),
    ];

    let resp = reqwest::Client::new()
        .get("https://www.googleapis.com/youtube/v3/videos")
        .query(&params)
        .send()
        .await?;

    let buf = resp.bytes().await?;

    // println!("{buf}");

    let a: VideoResult = match serde_json::from_slice(&buf) {
        Ok(r) => r,
        Err(err) => {
            let err = if let Ok(err) = Error::from_slice(&buf) {
                err
            } else {
                Error {
                    code: 0,
                    message: err.to_string(),
                }
            }
            .into();
            return Err(err);
        }
    };

    Ok(a.items.into_iter().next().unwrap().into())
}

#[cfg(test)]
#[tokio::test]
async fn test_get() {
    let r = get("", "j5Ejpw9RkzA").await.unwrap();

    println!("{r:#?}");
}

pub async fn search(
    youtube_api_key: impl AsRef<str>,
    keyword: impl AsRef<str>,
) -> Result<Vec<AudioMetadata>, Error> {
    let params = [
        ("part", "snippet"),
        ("type", "video"),
        ("key", youtube_api_key.as_ref()),
        ("q", keyword.as_ref()),
    ];

    let resp = reqwest::Client::new()
        .get("https://www.googleapis.com/youtube/v3/search")
        .query(&params)
        .send()
        .await?;

    let buf = resp.bytes().await?;

    let a: SearchResult = match serde_json::from_slice(&buf) {
        Ok(r) => r,
        Err(err) => {
            let err = if let Ok(err) = Error::from_slice(&buf) {
                err.into()
            } else {
                Error {
                    code: 0,
                    message: err.to_string(),
                }
                .into()
            };
            return Err(err);
        }
    };

    Ok(a.items.into_iter().map(Into::into).collect())

    // println!("{a:#?}");

    // let r = a
    //     .items
    //     .into_iter()
    //     .map(|item| {
    //         // let date = if let Some(d) = item.snippet.published_at {
    //         //     Some(d)
    //         // } else {
    //         //     item.snippet.publish_time
    //         // };

    //         let date = item.snippet.published_at;

    //         Metadata {
    //             track: None,
    //             artist: None, /* Some(item.snippet.channel_title) */
    //             title: Some(item.snippet.title),
    //             source_url: Some(format!(
    //                 "https://www.youtube.com/watch?v={}",
    //                 item.id.video_id
    //             )),
    //             date: Some(date),
    //             channel: Some(item.snippet.channel_title),
    //             channels: None,
    //             start_time: None,
    //             duration: None,
    //             sample_rate: None,
    //             thumbnail: None,
    //         }
    //     })
    //     .collect();
}

#[cfg(test)]
#[tokio::test]
async fn test_search() {
    let youtube_api_key = "";
    let xs = search(youtube_api_key, "MC재앙 개구리").await.unwrap();

    println!("{xs:#?}");
}

pub fn parse_vid(uri: Uri) -> String {
    #[derive(Deserialize)]
    struct Query {
        v: Option<String>,
    }

    let host = uri.host().unwrap_or("x");
    let path = uri.path();
    let query = uri.query().unwrap_or_default();

    log::debug!("host = {host}");
    log::debug!("path = {path}");
    log::debug!("query = {query}");

    if path.starts_with("/watch") {
        // https://www.youtube.com/watch?v=CLUDmYy9VP8
        let Query { v } = serde_qs::from_str(query).unwrap();

        if let Some(uid) = v {
            uid
        } else {
            // https://www.youtube.com/watch/CLUDmYy9VP8
            let x = path.split("/watch/");

            x.last().unwrap().to_string()
        }
    } else if path.starts_with("/v/") {
        // https://www.youtube.com/v/CLUDmYy9VP8
        let x = path.split("/v/");

        x.last().unwrap().to_string()
    } else if host == "youtu.be" {
        // https://youtu.be/CLUDmYy9VP8
        path.replace('/', "")
    } else {
        todo!("error handle");
    }
}
