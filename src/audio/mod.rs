pub mod cache;
pub mod metadata;
pub mod source;
pub mod ytdl;

pub use metadata::AudioMetadata;

use std::io;

use songbird::input::{self, Input};
use youtube_dl::YoutubeDl;

use self::cache::AudioCache;

pub const YTDL: &str = "./yt-dlp";
pub const YTDL_CACHE: &str = "./cache/youtube";
pub const SCDL_CACHE: &str = "./cache/soundcloud";

#[derive(Debug, thiserror::Error)]
pub enum AudioSourceError {
    #[error("input: {0:?}")]
    InputError(#[from] input::error::Error),

    #[error("io: {0}")]
    IoError(#[from] io::Error),

    #[error("youtube_api: {0}")]
    YouTubeApiError(#[from] ytdl::Error),

    #[error("must be video url")]
    MustSingleVideo,
}

pub enum AudioSource {
    YouTube(AudioMetadata),
    SoundCloud,
}

pub enum AudioSourceKind {
    YouTube,
    SoundCloud,
}

impl AudioSource {
    pub async fn from_youtube(
        id: &str,
        account: Option<(String, String)>,
        api_key: &str,
    ) -> Result<Self, AudioSourceError> {
        let x = if !AudioCache::exists(AudioSourceKind::YouTube, id)? {
            let mut ytdl = YoutubeDl::new(id.to_string()).to_owned();
            ytdl.youtube_dl_path(YTDL)
                .download(true)
                .format("webm[abr>0]/bestaudio/best")
                .output_directory(YTDL_CACHE)
                .output_template("%(id)s");
            if let Some((user_email, user_password)) = account.as_ref() {
                ytdl.auth(user_email, user_password);
            }

            ytdl.run_async()
                .await
                .unwrap()
                .into_single_video()
                .map(Into::into)
                .ok_or(AudioSourceError::MustSingleVideo)?
        } else {
            // FIXME: error
            ytdl::get(api_key, id).await?
        };

        Ok(Self::YouTube(x))
    }

    // pub async fn from_soundcloud(url: &str) -> Result<Self, AudioSourceError> {
    //     let x = if !AudioCache::exists(AudioSourceKind::SoundCloud) {
    //         let mut ytdl = YoutubeDl::new(id.to_string()).to_owned();
    //         ytdl.youtube_dl_path(YTDL)
    //             .download(true)
    //             .format("webm[abr>0]/bestaudio/best")
    //             .output_directory(YTDL_CACHE)
    //             .output_template("%(id)s")
    //             .run_async()
    //             .await
    //             .unwrap()
    //             .into_single_video()
    //             .map(Into::into)
    //             .ok_or(AudioSourceError::MustSingleVideo);
    //     } else {
    //         unimplemented!()
    //     };

    //     Ok(Self::SoundCloud)
    // }

    pub fn metadata(&self) -> Option<&AudioMetadata> {
        match self {
            Self::YouTube(x) => Some(x),

            Self::SoundCloud => unimplemented!("soundclound not implemented"),
        }
    }

    pub async fn get_source(&self) -> Result<Input, AudioSourceError> {
        let cached_source = AudioCache::get_source(self).await?;
        if let Some(source) = cached_source {
            return Ok(source);
        }

        unimplemented!()

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
