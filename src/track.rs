use serenity::prelude::TypeMapKey;
use songbird::tracks::TrackHandle;

pub struct Track(pub String, pub TrackHandle);

impl TypeMapKey for Track {
    type Value = Track;
}
