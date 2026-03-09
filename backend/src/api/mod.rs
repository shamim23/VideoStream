use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{Json, Response},
};
use futures_util::TryStreamExt;
use std::sync::Arc;
use tokio_util::io::{ReaderStream, StreamReader};

use crate::domain::ShareResponse;
use crate::service::{ServiceError, VideoService};

pub struct AppState {
    pub video_service: Arc<VideoService>,
}

pub async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<ShareResponse>, (StatusCode, String)> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if field.name() == Some("video") {
            let filename = field
                .file_name()
                .map(|s| s.to_string())
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    "Missing filename in upload".to_string(),
                ))?;
            let content_type = field
                .content_type()
                .map(|s| s.to_string())
                .ok_or((
                    StatusCode::BAD_REQUEST,
                    "Missing content-type in upload".to_string(),
                ))?;

            let stream = field.map_err(std::io::Error::other);
            let mut reader = StreamReader::new(stream);

            let id = state
                .video_service
                .upload_video(filename, content_type, &mut reader)
                .await
                .map_err(map_service_error)?;

            return Ok(Json(ShareResponse {
                share_url: format!("/api/watch/{}", id),
            }));
        }
    }

    Err((StatusCode::BAD_REQUEST, "No video file provided".to_string()))
}

pub async fn stream_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, String)> {
    let range = headers.get(header::RANGE).and_then(|v| v.to_str().ok());

    let stream = state
        .video_service
        .get_video_stream(&id, range)
        .await
        .map_err(map_service_error)?;

    let mut builder = Response::builder()
        .header(header::CONTENT_TYPE, stream.content_type)
        .header(header::CONTENT_LENGTH, stream.content_length.to_string())
        .header(header::ACCEPT_RANGES, "bytes");

    if let Some(content_range) = stream.content_range {
        builder = builder
            .status(StatusCode::PARTIAL_CONTENT)
            .header(header::CONTENT_RANGE, content_range);
    } else {
        builder = builder.status(StatusCode::OK);
    }

    builder
        .body(Body::from_stream(ReaderStream::new(stream.stream)))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn health_check() -> &'static str {
    "Video service ready!"
}

fn map_service_error(err: ServiceError) -> (StatusCode, String) {
    match err {
        ServiceError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        ServiceError::UnsupportedMediaType(msg) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, msg),
        ServiceError::PayloadTooLarge(msg) => (StatusCode::PAYLOAD_TOO_LARGE, msg),
        ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        ServiceError::RangeNotSatisfiable(msg) => (StatusCode::RANGE_NOT_SATISFIABLE, msg),
        ServiceError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
    }
}
