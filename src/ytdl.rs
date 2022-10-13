use http::Uri;
use serde::Deserialize;
use songbird::input::Metadata;

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
    pub items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    // pub kind: String,
    // pub etag: String,
    pub id: ItemId,
    pub snippet: Snippet,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemId {
    // pub kind: String,
    pub video_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub published_at: Option<String>,
    pub channel_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    // pub thumbnails:
    pub channel_title: Option<String>,
    // pub live_broadcast_content: String,
    pub publish_time: Option<String>,
}

pub async fn search(
    youtube_api_key: impl AsRef<str>,
    keyword: impl AsRef<str>,
) -> crate::Result<Vec<Metadata>> {
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

    let a: SearchResult = serde_json::from_slice(&buf).unwrap();

    let r = a
        .items
        .into_iter()
        .map(|item| {
            let date = if let Some(d) = item.snippet.published_at {
                Some(d)
            } else {
                item.snippet.publish_time
            };

            Metadata {
                track: None,
                artist: None, /* Some(item.snippet.channel_title) */
                title: item.snippet.title,
                source_url: Some(format!(
                    "https://www.youtube.com/watch?v={}",
                    item.id.video_id
                )),
                date,
                channel: item.snippet.channel_title,
                channels: None,
                start_time: None,
                duration: None,
                sample_rate: None,
                thumbnail: None,
            }
        })
        .collect();

    Ok(r)
}

/* pub async fn search_metadata(
    keyword: &str,
    len: u8,
) -> Result<Vec<Metadata>, songbird::input::error::Error> {
    let keyword = format!("ytsearch{len}:{keyword}");

    let youtube_dl_output = Command::new("youtube-dl")
        .args(&["-s", "-j", &keyword])
        .stdin(Stdio::null())
        .output()
        .await?;

    let xs = String::from_utf8(youtube_dl_output.stdout).unwrap();

    let metadata = xs
        .lines()
        .into_iter()
        .map(|x| {
            serde_json::from_str(x)
                .map(Metadata::from_ytdl_output)
                .map_err(|err| songbird::input::error::Error::Json {
                    error: err,
                    parsed_text: xs.clone(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(metadata)

    /* let o_vec = youtube_dl_output.stderr;

    let end = (&o_vec)
        .iter()
        .position(|el| *el == 0xA)
        .unwrap_or(o_vec.len());

    let value = serde_json::from_slice(&o_vec[..end]).map_err(|err| {
        songbird::input::error::Error::Json {
            error: err,
            parsed_text: std::str::from_utf8(&o_vec)
                .unwrap_or_default()
                .to_string(),
        }
    })?;

    let metadata = Metadata::from_ytdl_output(value);

    Ok(metadata) */
} */

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

#[cfg(test)]
mod tests {
    use super::*;

    /* #[tokio::test]
    async fn test_search_metadata() {
        let xs = search_metadata("MC재앙 개구리", 5).await.unwrap();

        println!("{xs:#?}");
    } */

    #[tokio::test]
    async fn test_search() {
        let youtube_api_key = "AIzaSyDW0CC9RmNDtT4qHdYFqBY9cJO42TDDm6s";
        let xs = search(youtube_api_key, "MC재앙 개구리").await.unwrap();

        println!("{xs:#?}");
    }
}
