use crate::domain::Video;
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

#[async_trait]
pub trait Storage: Send + Sync {
    async fn store_stream(&self, video: &Video, data: Vec<u8>) -> Result<PathBuf>;
    async fn read_range(&self, video: &Video, start: u64, end: u64) -> Result<Vec<u8>>;
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
    async fn store_stream(&self, video: &Video, data: Vec<u8>) -> Result<PathBuf> {
        let dir = self.base_path.join(&video.storage_path);
        tokio::fs::create_dir_all(&dir).await?;
        
        let file_path = dir.join("original.mp4");
        tokio::fs::write(&file_path, data).await?;
        
        // Write metadata backup
        let metadata_path = dir.join("metadata.json");
        let metadata = serde_json::to_string_pretty(video)?;
        tokio::fs::write(&metadata_path, metadata).await?;
        
        Ok(file_path)
    }

    async fn read_range(&self, video: &Video, start: u64, end: u64) -> Result<Vec<u8>> {
        let file_path = self.base_path.join(&video.storage_path).join("original.mp4");
        let mut file = tokio::fs::File::open(&file_path).await?;
        
        let mut buffer = vec![0u8; (end - start + 1) as usize];
        file.seek(std::io::SeekFrom::Start(start)).await?;
        file.read_exact(&mut buffer).await?;
        
        Ok(buffer)
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
