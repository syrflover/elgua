use std::process::{Child, Command, Stdio};

use songbird::input::{children_to_reader, Codec, Container, Input};

use super::AudioSourceError;

pub fn encode_to_source<T>(a: T, mut children: Vec<Child>) -> Result<Input, AudioSourceError>
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

    let ffmpeg = Command::new("ffmpeg")
        .arg("-i")
        .arg("-")
        .args(ffmpeg_args)
        .stdin(a)
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    children.push(ffmpeg);

    let source = Input::new(
        true,
        children_to_reader::<f32>(children),
        Codec::FloatPcm,
        Container::Raw,
        None,
    );

    Ok(source)
}
