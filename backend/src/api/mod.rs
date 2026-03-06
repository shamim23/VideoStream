use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    body::Body,
};
use std::sync::Arc;

use crate::domain::{ShareResponse, Video};
use crate::storage::Storage;

pub struct AppState {
    pub storage: Arc<dyn Storage>,
    pub db: sqlx::SqlitePool,
}

pub async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<ShareResponse>, (StatusCode, String)> {
    let mut file_data = None;
    let mut filename = None;
    let mut content_type = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if field.name() == Some("video") {
            filename = field.file_name().map(|s| s.to_string());
            content_type = field.content_type().map(|s| s.to_string());
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
                    .to_vec(),
            );
        }
    }

    let (data, filename, content_type) = match (file_data, filename, content_type) {
        (Some(d), Some(f), Some(c)) => (d, f, c),
        _ => return Err((StatusCode::BAD_REQUEST, "No video file provided".to_string())),
    };

    let size_bytes = data.len() as i64;
    let video = Video::new(filename, content_type, size_bytes);

    // Save to storage
    state
        .storage
        .store_stream(&video, data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Save to database
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
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ShareResponse {
        share_url: format!("/api/watch/{}", video.id),
    }))
}

pub async fn stream_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, String)> {
    // Get video metadata from database
    let video: Video = sqlx::query_as::<_, Video>("SELECT * FROM videos WHERE id = ?1")
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Video not found".to_string()))?;

    // Get file size
    let file_size = state
        .storage
        .get_size(&video)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Parse Range header
    let range = headers.get(header::RANGE).and_then(|v| v.to_str().ok());

    if let Some(range) = range {
        // Parse range: bytes=start-end
        let range = range.trim_start_matches("bytes=");
        let parts: Vec<&str> = range.split('-').collect();
        
        let start: u64 = parts[0].parse().unwrap_or(0);
        let end: u64 = parts
            .get(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(file_size - 1);

        let content_length = end - start + 1;

        // Read range from storage
        let data = state
            .storage
            .read_range(&video, start, end)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let response_headers = [
            (header::CONTENT_TYPE, video.content_type),
            (header::CONTENT_LENGTH, content_length.to_string()),
            (
                header::CONTENT_RANGE,
                format!("bytes {}-{}/{}", start, end, file_size),
            ),
            (header::ACCEPT_RANGES, "bytes".to_string()),
        ];

        return Ok((
            StatusCode::PARTIAL_CONTENT,
            response_headers,
            Body::from(data),
        )
            .into_response());
    }

    // No range requested, return full file
    let data = state
        .storage
        .read_range(&video, 0, file_size - 1)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response_headers = [
        (header::CONTENT_TYPE, video.content_type),
        (header::CONTENT_LENGTH, file_size.to_string()),
        (header::ACCEPT_RANGES, "bytes".to_string()),
    ];

    Ok((StatusCode::OK, response_headers, Body::from(data)).into_response())
}

pub async fn health_check() -> &'static str {
    "Video service ready!"
}

// Database mapping for Video
use sqlx::FromRow;

impl FromRow<'_, sqlx::sqlite::SqliteRow> for Video {
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> sqlx::Result<Self> {
        use sqlx::Row;
        Ok(Video {
            id: row.try_get("id")?,
            filename: row.try_get("filename")?,
            content_type: row.try_get("content_type")?,
            size_bytes: row.try_get("size_bytes")?,
            storage_path: row.try_get("storage_path")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
