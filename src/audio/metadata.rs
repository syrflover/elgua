use super::{
    scdl,
    ytdl::{SearchItem, VideoItem},
    AudioSourceKind,
};

#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub id: String,
    pub title: String,
    pub url: String,
    pub thumbnail_url: String,
    pub uploaded_by: String,

    // pub duration: Either<Duration, String>,
    _kind: AudioSourceKind,
}

impl AudioMetadata {
    pub fn kind(&self) -> AudioSourceKind {
        self._kind
    }
}

impl From<scdl::Track> for AudioMetadata {
    fn from(x: scdl::Track) -> Self {
        Self {
            id: x.id.to_string(),
            title: x.title,
            url: x.permalink_url,
            thumbnail_url: x
                .artwork_url
                .unwrap_or(x.user.avatar_url.unwrap_or_default()), // TODO: default thumbnail
            uploaded_by: x.user.username,

            // duration: Either::Left(Duration::from_millis(x.duration)),
            _kind: AudioSourceKind::SoundCloud,
        }
    }
}

impl From<youtube_dl::SingleVideo> for AudioMetadata {
    fn from(x: youtube_dl::SingleVideo) -> Self {
        Self {
            id: x.id,
            title: x.title,
            url: x.webpage_url.unwrap(),
            thumbnail_url: x.thumbnail.unwrap(),
            uploaded_by: x.channel.unwrap_or("#anonymous#".to_string()),

            // duration: Either::Right(x.duration_string.unwrap()),
            _kind: AudioSourceKind::YouTube,
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

            // duration: Either::Left(Duration::from_millis(x.file_details.duration_ms)),
            _kind: AudioSourceKind::YouTube,
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

            // duration: Either::Right("알 수 없음".to_string()),
            _kind: AudioSourceKind::YouTube,
        }
    }
}
