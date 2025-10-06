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
    pub items: Option<Vec<SearchItem>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoResult {
    // pub kind: String,
    // pub etag: String,
    // pub next_page_token: Option<String>,
    // pub region_code: String,
    // pub page_info: PageInfo,
    pub items: Option<Vec<VideoItem>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchItem {
    // pub kind: String,
    // pub etag: String,
    pub id: Option<SearchItemId>,
    pub snippet: Option<Snippet>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoItem {
    pub id: Option<String>,
    pub snippet: Option<Snippet>,
    pub content_details: Option<ContentDetail>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchItemId {
    // pub kind: String,
    pub video_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    pub url: Option<String>,
    pub width: Option<usize>,
    pub height: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDetail {
    /// iso8601
    ///
    /// PT#M#S, PT#H#M#S, P#DT#H#M#S
    ///
    /// .e.g, P1DT23H11M1S
    pub duration: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub published_at: Option<String>,
    pub channel_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumbnails: Option<HashMap<String, Thumbnail>>,
    pub channel_title: Option<String>,
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

pub fn is_youtube_url(x: &str) -> bool {
    x.starts_with("https://music.youtube.com/watch")
        || x.starts_with("https://www.youtube.com/watch")
        || x.starts_with("https://www.youtube.com/shorts/")
        || x.starts_with("https://www.youtube.com/v/")
        || x.starts_with("https://youtu.be/")
}

pub async fn get(
    youtube_api_key: impl AsRef<str>,
    id: impl AsRef<str>,
) -> Result<AudioMetadata, Error> {
    let id = id.as_ref();
    let params = [
        ("part", "snippet,id,contentDetails"),
        ("type", "video"),
        ("key", youtube_api_key.as_ref()),
        ("id", id),
    ];

    let resp = reqwest::Client::new()
        .get("https://www.googleapis.com/youtube/v3/videos")
        .query(&params)
        .send()
        .await?;

    let buf = resp.bytes().await?;

    // println!("{}", String::from_utf8(buf.to_vec()).unwrap());

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
            };
            return Err(err);
        }
    };

    let item = a.items.unwrap_or_default().into_iter().next();

    match item.map(|item| AudioMetadata::try_from(item)) {
        Some(Ok(r)) => Ok(r),
        Some(Err(_)) => Err(Error {
            code: 404,
            message: "유튜브에서 제대로 된 정보를 주지 않았습니다".to_owned(),
        }),
        None => Err(Error {
            code: 404,
            message: "영상을 찾을 수 없습니다".to_owned(),
        }),
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_get() {
    let r = get("AIzaSyDW0CC9RmNDtT4qHdYFqBY9cJO42TDDm6s", "3sdVp4lWI9E")
        .await
        .unwrap();

    println!("{r:#?}");
}

pub async fn search(
    youtube_api_key: impl AsRef<str>,
    keyword: impl AsRef<str>,
) -> Result<Vec<AudioMetadata>, Error> {
    let params = [
        ("part", "snippet"),
        ("type", "video"),
        // ("maxResults", "5"),
        ("safeSearch", "none"),
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
                err
            } else {
                Error {
                    code: 0,
                    message: err.to_string(),
                }
            };
            return Err(err);
        }
    };

    let search_results = a
        .items
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| AudioMetadata::try_from(item).ok())
        .collect::<Vec<_>>();

    if search_results.is_empty() {
        return Err(Error {
            code: 404,
            message: "검색된 결과가 없습니다".to_owned(),
        });
    }

    Ok(search_results)

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
