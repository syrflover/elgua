use std::{io, path::Path};

use songbird::input::{File, Input};

use super::{AudioSource, AudioSourceError, AudioSourceKind, SCDL_CACHE, YTDL_CACHE};

pub struct AudioCache;

impl AudioCache {
    pub fn exists(kind: AudioSourceKind, id: impl AsRef<str>) -> io::Result<bool> {
        let id = id.as_ref();

        let p = match kind {
            AudioSourceKind::YouTube => format!("{YTDL_CACHE}/{}", id),
            AudioSourceKind::SoundCloud => format!("{SCDL_CACHE}/{}", id),
        };

        Path::new(&p).try_exists()
    }

    pub async fn get_source(audio_source: &AudioSource) -> Result<Input, AudioSourceError> {
        let file_path = match audio_source {
            AudioSource::YouTube(x) => format!("{YTDL_CACHE}/{}", x.id),
            AudioSource::SoundCloud(x) => format!("{SCDL_CACHE}/{}", x.id),
        };

        Ok(Input::from(File::new(file_path)))
    }
}
