use super::ytdl::{SearchItem, VideoItem};

#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub id: String,
    pub title: String,
    pub url: String,
    pub thumbnail_url: String,
    pub uploaded_by: String,
}

impl From<youtube_dl::SingleVideo> for AudioMetadata {
    fn from(x: youtube_dl::SingleVideo) -> Self {
        Self {
            id: x.id,
            title: x.title,
            url: x.webpage_url.unwrap(),
            thumbnail_url: x.thumbnail.unwrap(),
            uploaded_by: x.channel.unwrap_or("#anonymous#".to_string()),
        }
    }
}

impl From<VideoItem> for AudioMetadata {
    fn from(x: VideoItem) -> Self {
        let url = format!("https://www.youtube.com/watch?v={}", x.id);
        let thumbnail_url = x
            .snippet
            .thumbnails
            .into_values()
            .max_by(|a, b| (a.width * a.height).cmp(&(b.width * b.height)))
            .unwrap()
            .url;

        Self {
            id: x.id,
            title: x.snippet.title,
            url,
            thumbnail_url,
            uploaded_by: x.snippet.channel_title,
        }
    }
}

impl From<SearchItem> for AudioMetadata {
    fn from(x: SearchItem) -> Self {
        let url = format!("https://www.youtube.com/watch?v={}", x.id.video_id);
        let thumbnail_url = x
            .snippet
            .thumbnails
            .into_values()
            .max_by(|a, b| (a.width * a.height).cmp(&(b.width * b.height)))
            .unwrap()
            .url;

        Self {
            id: x.id.video_id,
            title: x.snippet.title,
            url,
            thumbnail_url,
            uploaded_by: x.snippet.channel_title,
        }
    }
}
