use std::{io, path::Path};

use songbird::input::Input;

use tokio::fs::File;

use super::{
    source::encode_to_source, AudioSource, AudioSourceError, AudioSourceKind, SCDL_CACHE,
    YTDL_CACHE,
};

pub struct AudioCache;

impl AudioCache {
    pub fn exists(kind: AudioSourceKind, id: impl AsRef<str>) -> io::Result<bool> {
        let p = match kind {
            AudioSourceKind::YouTube => format!("{YTDL_CACHE}/{}", id.as_ref()),
            AudioSourceKind::SoundCloud => format!("{SCDL_CACHE}/{}", id.as_ref()),
        };

        Path::new(&p).try_exists()
    }

    pub async fn get_source(audio_source: &AudioSource) -> Result<Option<Input>, AudioSourceError> {
        let filepath = match audio_source {
            AudioSource::YouTube(x) => format!("{YTDL_CACHE}/{}", x.id),
            AudioSource::SoundCloud(x) => format!("{SCDL_CACHE}/{}", x.id),
        };

        let f = match File::open(filepath).await {
            Ok(r) => r.into_std().await,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        let source = encode_to_source(f).await?;

        Ok(Some(source))
    }
}
