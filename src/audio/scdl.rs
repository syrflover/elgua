use reqwest::redirect;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Track {
    pub id: u64,
    pub title: String,
    /// track
    // pub kind: String,
    pub artwork_url: Option<String>,
    pub permalink_url: String,
    pub duration: u64,
    pub full_duration: Option<u64>,

    pub user: User,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: u64,
    // pub full_name: String,
    // pub first_name: String,
    // pub last_name: String,
    pub username: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("other: {0}")]
    Other(String),
}

pub fn is_soundcloud_shared_url(x: &str) -> bool {
    x.starts_with("https://on.soundcloud.com/")
}

pub fn is_soundcloud_url(x: &str) -> bool {
    x.starts_with("https://soundcloud.com/") || is_soundcloud_shared_url(x)
}

async fn get_track_url_from_shared_url(shared_url: &str) -> Result<String, Error> {
    let resp = reqwest::Client::builder()
        .redirect(redirect::Policy::none())
        .build()?
        .get(shared_url)
        .send()
        .await?;

    if resp.status() == reqwest::StatusCode::FOUND {
        match resp.headers().get(reqwest::header::LOCATION) {
            Some(x) => Ok(x.to_str().unwrap().to_string()),
            None => Err(Error::Other(
                "succeeded redirect, but doesn't have location header".to_string(),
            )),
        }
    } else {
        Err(Error::Other("not redirected".to_string()))
    }

    // println!("{resp:#?}");
    // println!("{:?}", resp.headers().get(reqwest::header::LOCATION));
}

pub async fn get_track(client_id: &str, track_url: &str) -> Result<Track, Error> {
    let track_url = if is_soundcloud_shared_url(track_url) {
        get_track_url_from_shared_url(track_url).await?
    } else {
        track_url.to_string()
    };

    // println!("{}", track_url);

    let params = [("client_id", client_id), ("url", &track_url)];

    let resp = reqwest::Client::new()
        .get("https://api-v2.soundcloud.com/resolve")
        .query(&params)
        .send()
        .await?;

    let status_code = resp.status();
    let buf = resp.bytes().await?;

    // println!("{}", String::from_utf8(buf.to_vec()).unwrap());

    let a = match serde_json::from_slice(&buf) {
        Ok(r) => r,
        Err(err) => {
            if status_code.is_success() {
                return Err(Error::Other(err.to_string()));
            } else {
                return Err(Error::Other(format!(
                    "{}: {}",
                    status_code,
                    status_code.as_str()
                )));
            }
        }
    };

    Ok(a)
}

#[cfg(test)]
#[tokio::test]
async fn test_get_track() {
    let track = get_track("", "https://on.soundcloud.com/WdDryML5RrJGtANu9")
        .await
        .unwrap();

    println!("{track:?}")
}
