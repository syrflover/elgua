use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Track {
    pub id: u64,
    pub title: String,
    /// track
    // pub kind: String,
    pub artwork_url: Option<String>,
    pub permalink_url: String,

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

pub async fn get_track(client_id: &str, track_url: &str) -> Result<Track, Error> {
    let params = [("client_id", client_id), ("url", track_url)];

    let resp = reqwest::Client::new()
        .get("https://api-v2.soundcloud.com/resolve")
        .query(&params)
        .send()
        .await?;

    let buf = resp.bytes().await?;

    println!("{}", String::from_utf8(buf.to_vec()).unwrap());

    let a = match serde_json::from_slice(&buf) {
        Ok(r) => r,
        Err(err) => return Err(Error::Other(err.to_string())),
    };

    Ok(a)
}

#[cfg(test)]
#[tokio::test]
async fn test_get_track() {
    let track = get_track("", "").await.unwrap();

    println!("{track:?}")
}
