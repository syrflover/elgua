use serenity::prelude::TypeMapKey;
use songbird::tracks::TrackHandle;

use crate::audio::AudioMetadata;

pub struct Track(pub AudioMetadata, pub TrackHandle);

impl TypeMapKey for Track {
    type Value = Track;
}
