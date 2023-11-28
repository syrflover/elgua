use std::process::Stdio;

use songbird::input::{children_to_reader, Codec, Container, Input, Reader};
use tokio::io::AsyncReadExt;

use super::AudioSourceError;

pub async fn encode_to_source<T>(a: T, to_memory: bool) -> Result<Input, AudioSourceError>
where
    T: Into<Stdio>,
{
    let ffmpeg_args = [
        "-f",
        "s16le",
        "-ac",
        "2",
        "-ar",
        "48000",
        "-acodec",
        "pcm_f32le",
        "-",
    ];

    let source = if to_memory {
        let mut ffmpeg = tokio::process::Command::new("ffmpeg")
            .arg("-i")
            .arg("-")
            .args(ffmpeg_args)
            .stdin(a)
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut buf = Vec::new();
        let mut stdout = ffmpeg.stdout.take().unwrap();

        stdout.read_to_end(&mut buf).await?;

        Input::new(
            true,
            Reader::from_memory(buf),
            Codec::FloatPcm,
            Container::Raw,
            None,
        )
    } else {
        let ffmpeg = std::process::Command::new("ffmpeg")
            .arg("-i")
            .arg("-")
            .args(ffmpeg_args)
            .stdin(a)
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()?;

        Input::new(
            true,
            children_to_reader::<f32>(vec![ffmpeg]),
            Codec::FloatPcm,
            Container::Raw,
            None,
        )
    };

    Ok(source)
}
