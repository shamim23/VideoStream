use crate::domain::Video;
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt};

#[async_trait]
pub trait Storage: Send + Sync {
    async fn store_stream(
        &self,
        video: &mut Video,
        reader: &mut (dyn AsyncRead + Unpin + Send),
    ) -> Result<PathBuf>;
    async fn open_read_stream(
        &self,
        video: &Video,
        start: u64,
        length: Option<u64>,
    ) -> Result<Box<dyn AsyncRead + Unpin + Send>>;
    async fn get_size(&self, video: &Video) -> Result<u64>;
    async fn delete(&self, video: &Video) -> Result<()>;
}

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn store_stream(
        &self,
        video: &mut Video,
        reader: &mut (dyn AsyncRead + Unpin + Send),
    ) -> Result<PathBuf> {
        let dir = self.base_path.join(&video.storage_path);
        tokio::fs::create_dir_all(&dir).await?;

        let file_path = dir.join("original.mp4");
        let mut file = tokio::fs::File::create(&file_path).await?;
        let written = tokio::io::copy(reader, &mut file).await?;
        video.size_bytes = written as i64;

        // Write metadata backup
        let metadata_path = dir.join("metadata.json");
        let metadata = serde_json::to_string_pretty(video)?;
        tokio::fs::write(&metadata_path, metadata).await?;

        Ok(file_path)
    }

    async fn open_read_stream(
        &self,
        video: &Video,
        start: u64,
        length: Option<u64>,
    ) -> Result<Box<dyn AsyncRead + Unpin + Send>> {
        let file_path = self.base_path.join(&video.storage_path).join("original.mp4");
        let mut file = tokio::fs::File::open(&file_path).await?;
        file.seek(std::io::SeekFrom::Start(start)).await?;

        if let Some(length) = length {
            Ok(Box::new(file.take(length)))
        } else {
            Ok(Box::new(file))
        }
    }

    async fn get_size(&self, video: &Video) -> Result<u64> {
        let file_path = self.base_path.join(&video.storage_path).join("original.mp4");
        let metadata = tokio::fs::metadata(&file_path).await?;
        Ok(metadata.len())
    }

    async fn delete(&self, video: &Video) -> Result<()> {
        let dir = self.base_path.join(&video.storage_path);
        tokio::fs::remove_dir_all(&dir).await?;
        Ok(())
    }
}
