use std::{collections::HashMap, time::Duration};

use crate::{audio::ytdl::Thumbnail, util::time::parse_iso8601_duration};

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
    pub thumbnail_url: Option<String>,
    pub uploaded_by: String,

    pub duration: Option<Duration>,
    pub(super) _kind: AudioSourceKind,
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
            thumbnail_url: x.artwork_url.or(x.user.avatar_url), // TODO: default thumbnail
            uploaded_by: x.user.username,

            duration: Some(Duration::from_millis(x.duration)),
            _kind: AudioSourceKind::SoundCloud,
        }
    }
}

impl TryFrom<VideoItem> for AudioMetadata {
    type Error = ();

    fn try_from(item: VideoItem) -> Result<Self, Self::Error> {
        fn temp(item: VideoItem) -> Option<AudioMetadata> {
            let snippet = item.snippet?;
            let video_id = item.id?;

            let url = format!("https://www.youtube.com/watch?v={}", video_id);
            let thumbnail = find_highest_thumbnail(snippet.thumbnails);

            Some(AudioMetadata {
                id: video_id,
                title: snippet.title?,
                url,
                thumbnail_url: thumbnail.and_then(|t| t.url),
                uploaded_by: snippet.channel_title?,

                duration: item
                    .content_details?
                    .duration
                    .and_then(|d| parse_iso8601_duration(&d)),
                _kind: AudioSourceKind::YouTube,
            })
        }

        temp(item).ok_or(())
    }
}

impl TryFrom<SearchItem> for AudioMetadata {
    type Error = ();

    fn try_from(item: SearchItem) -> Result<Self, Self::Error> {
        fn temp(item: SearchItem) -> Option<AudioMetadata> {
            let video_id = item.id?.video_id?;
            let snippet = item.snippet?;

            let url = format!("https://www.youtube.com/watch?v={}", video_id);
            let thumbnail = find_highest_thumbnail(snippet.thumbnails);

            Some(AudioMetadata {
                id: video_id,
                title: snippet.title?,
                url,
                thumbnail_url: thumbnail.and_then(|t| t.url),
                uploaded_by: snippet.channel_title?,

                duration: None,
                _kind: AudioSourceKind::YouTube,
            })
        }

        temp(item).ok_or(())
    }
}

fn find_highest_thumbnail(thumbnails: Option<HashMap<String, Thumbnail>>) -> Option<Thumbnail> {
    thumbnails?
        .into_values()
        .filter_map(|thumbnail| {
            thumbnail.width?;
            thumbnail.height?;
            Some(thumbnail)
        })
        .max_by(|a, b| {
            (a.width.unwrap() * a.height.unwrap()).cmp(&(b.width.unwrap() * b.height.unwrap()))
        })
}
