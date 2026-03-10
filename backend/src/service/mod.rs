use std::path::Path;
use std::sync::Arc;
use tokio::io::AsyncRead;

use crate::domain::Video;
use crate::storage::Storage;

const MAX_UPLOAD_BYTES: i64 = 1024 * 1024 * 1024; // 1GB
const ALLOWED_VIDEO_EXTENSIONS: &[&str] = &["mp4", "webm", "mov", "mkv", "ogv"];
const ALLOWED_VIDEO_MIME_TYPES: &[&str] = &[
    "video/mp4",
    "video/webm",
    "video/quicktime",
    "video/x-matroska",
    "video/ogg",
];

#[derive(Debug, PartialEq)]
pub enum ServiceError {
    BadRequest(String),
    UnsupportedMediaType(String),
    PayloadTooLarge(String),
    NotFound(String),
    RangeNotSatisfiable(String),
    Internal(String),
}

pub struct StreamResult {
    pub content_type: String,
    pub content_length: u64,
    pub content_range: Option<String>,
    pub stream: Box<dyn AsyncRead + Unpin + Send>,
}

#[derive(Clone)]
pub struct VideoService {
    storage: Arc<dyn Storage>,
    db: sqlx::SqlitePool,
}

impl VideoService {
    pub fn new(storage: Arc<dyn Storage>, db: sqlx::SqlitePool) -> Self {
        Self { storage, db }
    }

    pub async fn upload_video(
        &self,
        filename: String,
        content_type: String,
        reader: &mut (dyn AsyncRead + Unpin + Send),
    ) -> Result<String, ServiceError> {
        if !is_supported_video(&filename, &content_type) {
            return Err(ServiceError::UnsupportedMediaType(format!(
                "Unsupported video format. Allowed extensions: {}",
                ALLOWED_VIDEO_EXTENSIONS.join(", ")
            )));
        }

        let mut video = Video::new(filename, content_type, 0);
        self.storage
            .store_stream(&mut video, reader)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        if video.size_bytes <= 0 {
            let _ = self.storage.delete(&video).await;
            return Err(ServiceError::BadRequest("Uploaded file is empty".to_string()));
        }

        if video.size_bytes > MAX_UPLOAD_BYTES {
            let _ = self.storage.delete(&video).await;
            return Err(ServiceError::PayloadTooLarge(
                "File too large. Maximum size is 1GB".to_string(),
            ));
        }

        sqlx::query(
            r#"
            INSERT INTO videos (id, filename, content_type, size_bytes, storage_path, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(&video.id)
        .bind(&video.filename)
        .bind(&video.content_type)
        .bind(video.size_bytes)
        .bind(&video.storage_path)
        .bind(video.created_at)
        .execute(&self.db)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok(video.id)
    }

    pub async fn get_video_stream(
        &self,
        id: &str,
        range_header: Option<&str>,
    ) -> Result<StreamResult, ServiceError> {
        let video: Video = sqlx::query_as::<_, Video>("SELECT * FROM videos WHERE id = ?1")
            .bind(id)
            .fetch_one(&self.db)
            .await
            .map_err(|_| ServiceError::NotFound("Video not found".to_string()))?;

        let file_size = self
            .storage
            .get_size(&video)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        if let Some(range_header) = range_header {
            let (start, end) = parse_range(range_header, file_size)?;
            let content_length = end - start + 1;
            let stream = self
                .storage
                .open_read_stream(&video, start, Some(content_length))
                .await
                .map_err(|e| ServiceError::Internal(e.to_string()))?;

            return Ok(StreamResult {
                content_type: video.content_type,
                content_length,
                content_range: Some(format!("bytes {}-{}/{}", start, end, file_size)),
                stream,
            });
        }

        let stream = self
            .storage
            .open_read_stream(&video, 0, None)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        Ok(StreamResult {
            content_type: video.content_type,
            content_length: file_size,
            content_range: None,
            stream,
        })
    }
}

fn is_supported_video(filename: &str, content_type: &str) -> bool {
    let mime_ok = ALLOWED_VIDEO_MIME_TYPES.contains(&content_type) || content_type.starts_with("video/");
    let ext_ok = Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ALLOWED_VIDEO_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false);

    mime_ok && ext_ok
}

fn parse_range(range_header: &str, file_size: u64) -> Result<(u64, u64), ServiceError> {
    if file_size == 0 {
        return Err(ServiceError::RangeNotSatisfiable(
            "Range out of bounds".to_string(),
        ));
    }

    let range_value = range_header.trim();
    if !range_value.starts_with("bytes=") {
        return Err(ServiceError::RangeNotSatisfiable(
            "Invalid range unit".to_string(),
        ));
    }

    let raw = &range_value["bytes=".len()..];
    let mut parts = raw.splitn(2, '-');
    let start_part = parts.next().unwrap_or_default();
    let end_part = parts.next().unwrap_or_default();

    let (start, end) = if start_part.is_empty() {
        let suffix_len: u64 = end_part
            .parse()
            .map_err(|_| ServiceError::RangeNotSatisfiable("Invalid range".to_string()))?;
        if suffix_len == 0 {
            return Err(ServiceError::RangeNotSatisfiable("Invalid range".to_string()));
        }
        let start = file_size.saturating_sub(suffix_len);
        (start, file_size - 1)
    } else {
        let start: u64 = start_part
            .parse()
            .map_err(|_| ServiceError::RangeNotSatisfiable("Invalid range".to_string()))?;
        let end: u64 = if end_part.is_empty() {
            file_size - 1
        } else {
            end_part
                .parse()
                .map_err(|_| ServiceError::RangeNotSatisfiable("Invalid range".to_string()))?
        };
        (start, end.min(file_size - 1))
    };

    if start >= file_size || end < start {
        return Err(ServiceError::RangeNotSatisfiable(
            "Range out of bounds".to_string(),
        ));
    }

    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_video() {
        // Valid videos
        assert!(is_supported_video("test.mp4", "video/mp4"));
        assert!(is_supported_video("test.webm", "video/webm"));
        assert!(is_supported_video("test.mov", "video/quicktime"));

        // Invalid extension
        assert!(!is_supported_video("test.exe", "video/mp4"));
        assert!(!is_supported_video("test.pdf", "application/pdf"));

        // Invalid MIME type
        assert!(!is_supported_video("test.mp4", "application/exe"));
    }

    #[test]
    fn test_parse_range() {
        // Absolute range
        assert_eq!(parse_range("bytes=0-1023", 10000), Ok((0, 1023)));

        // Open-ended range
        assert_eq!(parse_range("bytes=1000-", 10000), Ok((1000, 9999)));

        // Suffix range (last N bytes)
        assert_eq!(parse_range("bytes=-1000", 10000), Ok((9000, 9999)));
    }

    #[test]
    fn test_parse_range_errors() {
        // Out of bounds
        assert!(parse_range("bytes=10000-20000", 5000).is_err());

        // Invalid range (start > end)
        assert!(parse_range("bytes=500-100", 1000).is_err());

        // Empty file
        assert!(parse_range("bytes=0-100", 0).is_err());
    }
}
