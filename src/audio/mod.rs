pub mod cache;
pub mod metadata;
pub mod scdl;
pub mod ytdl;

pub use metadata::AudioMetadata;

use std::io;

use songbird::input::{self, Input};
use youtube_dl::YoutubeDl;

use crate::store::{History, HistoryKind};

use self::cache::AudioCache;

pub const YTDL: &str = "./yt-dlp";
pub const YTDL_CACHE: &str = "./cache/youtube";
pub const SCDL_CACHE: &str = "./cache/soundcloud";

#[derive(Debug, thiserror::Error)]
pub enum AudioSourceError {
    #[error("input: {0:?}")]
    InputError(#[from] input::AudioStreamError),

    #[error("io: {0}")]
    IoError(#[from] io::Error),

    #[error("youtube_dl: {0}")]
    YouTubeDlError(#[from] youtube_dl::Error),

    #[error("youtube_api: {0}")]
    YouTubeApiError(#[from] ytdl::Error),

    #[error("soundcloud_api: {0}")]
    SoundCloudApiError(#[from] scdl::Error),

    #[error("must be video url")]
    MustSingleVideo,
}

pub enum AudioSource {
    YouTube(AudioMetadata),
    SoundCloud(AudioMetadata),
}

#[derive(Debug, Clone, Copy)]
pub enum AudioSourceKind {
    YouTube,
    SoundCloud,
}

pub fn starts_with_invalid_char(x: &str) -> bool {
    x.starts_with(['-'])
}

impl AudioSource {
    pub async fn from_youtube(api_key: &str, id: &str) -> Result<Self, AudioSourceError> {
        if !AudioCache::exists(AudioSourceKind::YouTube, id)? {
            let starts_with_invalid_char = starts_with_invalid_char(id);

            YoutubeDl::new(if starts_with_invalid_char {
                // format!("ytsearch:{id}")
                format!("https://youtu.be/{id}")
            } else {
                id.to_string()
            })
            .youtube_dl_path(YTDL)
            .format("webm[abr>0]/bestaudio/best")
            .output_template("%(id)s")
            .download_to_async(YTDL_CACHE)
            .await?;
        }

        let x = ytdl::get(api_key, id).await?;

        Ok(Self::YouTube(x))
    }

    pub async fn from_soundcloud(
        client_id: &str,
        track_url: &str,
    ) -> Result<Self, AudioSourceError> {
        let track = scdl::get_track(client_id, track_url).await?;
        let track_id = track.id.to_string();

        if !AudioCache::exists(AudioSourceKind::SoundCloud, &track_id)? {
            YoutubeDl::new(track_url)
                .to_owned()
                .youtube_dl_path(YTDL)
                .format("webm[abr>0]/bestaudio/best")
                .output_template("%(id)s")
                .download_to_async(SCDL_CACHE)
                .await?;
        }

        Ok(Self::SoundCloud(track.into()))
    }

    pub fn from_history(
        History {
            title,
            channel,
            kind,
            uid,
            ..
        }: History,
    ) -> Self {
        let (kind, url) = match kind {
            HistoryKind::YouTube => (
                AudioSourceKind::YouTube,
                format!("https://youtu.be/{}", uid),
            ),
            HistoryKind::SoundCloud => (
                AudioSourceKind::SoundCloud,
                format!("https://api-v2.soundcloud.com/tracks/{}", uid),
            ),
        };
        let metadata = AudioMetadata {
            id: uid,
            title,
            url,
            thumbnail_url: "".to_owned(),
            uploaded_by: channel,
            duration: None,
            _kind: kind,
        };

        match kind {
            AudioSourceKind::YouTube => AudioSource::YouTube(metadata),
            AudioSourceKind::SoundCloud => AudioSource::SoundCloud(metadata),
        }
    }

    pub fn metadata(&self) -> &AudioMetadata {
        match self {
            Self::YouTube(x) => x,
            Self::SoundCloud(x) => x,
        }
    }

    pub async fn get_source(&self) -> Result<Input, AudioSourceError> {
        let metadata = self.metadata();

        if AudioCache::exists(metadata.kind(), &metadata.id)? {
            Ok(AudioCache::get_source(self).await?)
        } else {
            Err(AudioSourceError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "캐시된 영상을 찾을 수 없습니다",
            )))
        }

        // match self {
        //     Self::YouTube(_x) => {
        //         // this code is not call
        //         // get_source_from_youtube(x.webpage_url.as_deref().unwrap(), self.account())
        //         unreachable!()
        //     }

        //     Self::SoundCloud => unimplemented!("soundcloud not implemented"),
        // }
    }

    // fn account(&self) -> Option<(&str, &str)> {
    //     match self {
    //         Self::YouTube(_, Some((user_email, user_password))) => {
    //             Some((user_email, user_password))
    //         }
    //         _ => None,
    //     }
    // }
}

// fn get_source_from_youtube(
//     url: &str,
//     account: Option<(&str, &str)>,
// ) -> Result<Input, AudioSourceError> {
//     let p = format!("{YTDL_CACHE}/%(id)s");
//     let ytdl_args = [
//         // "--print-json",
//         "-f",
//         "webm[abr>0]/bestaudio/best",
//         "-R",
//         "infinite",
//         url,
//         "-o",
//         &p,
//         "-o",
//         "-",
//     ];

//     let mut ytdl = Command::new(YTDL);

//     if let Some((user_email, user_password)) = account {
//         ytdl.arg("-u").arg(user_email).arg("-p").arg(user_password);
//     }

//     let mut ytdl = ytdl
//         .args(ytdl_args)
//         .stdin(Stdio::null())
//         .stderr(Stdio::null())
//         .stdout(Stdio::piped())
//         .spawn()?;

//     let ytdl_stdout = ytdl.stdout.take().ok_or(input::error::Error::Stdout)?;

//     let source = encode_to_source(ytdl_stdout, vec![ytdl])?;

//     Ok(source)
// }
