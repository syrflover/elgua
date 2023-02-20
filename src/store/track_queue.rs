use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use super::HistoryKind;

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackQueue {
    title: String,
    kind: HistoryKind,
    uid: String,
    created_at: DateTime<Utc>,
}

/// 대기열의 최대 길이를 설정해야하는데
/// 기준은 디스코드에서 보여줄 수 있는 만큼 또는 성능에 무리가 가지 않는 만큼
pub struct TrackQueueStore {
    path: PathBuf,
}

impl TrackQueueStore {
    pub fn new(p: impl AsRef<PathBuf>) -> io::Result<Self> {
        let p = p.as_ref();
        fs::create_dir_all(p)?;

        Ok(Self { path: p.clone() })
    }

    async fn read(&self) -> io::Result<Vec<TrackQueue>> {
        let mut file = File::open(&self.path).await?;
        let mut buf = Vec::new();

        file.read_to_end(&mut buf).await?;

        Ok(serde_json::from_slice(&buf).unwrap())
    }

    async fn write(&self, queue: Vec<TrackQueue>) -> io::Result<()> {
        let r = serde_json::to_string(&queue).unwrap();

        let mut file = File::open(&self.path).await?;

        file.write_all(r.as_bytes()).await
    }

    /// 중복된 노래를 추가하는 경우에는 이전 노래는 지우고 가장 마지막에 다시 추가
    pub async fn push_back(&self, track_queue: TrackQueue) -> io::Result<()> {
        let mut queue = self.read().await?;

        queue.push(track_queue);

        self.write(queue).await
    }

    pub async fn clear(&self) -> io::Result<()> {
        self.write(Vec::new()).await
    }

    pub async fn remove(&self, uid: impl AsRef<str>) -> io::Result<()> {
        let uid = uid.as_ref();

        todo!()
    }
}
