# Video Streaming Service Architecture

## System Overview

A minimal private video streaming platform with clean architecture, supporting uploads up to 1GB and streaming via shareable private links.

## Architecture Principles

1. **Separation of Concerns**: Clear boundaries between API, domain, services, and storage
2. **Dependency Inversion**: Dependencies point inward (Domain → Service → API)
3. **Storage Abstraction**: Trait-based storage enables swapping implementations
4. **Streaming-First**: All file operations use async streaming
5. **Type Safety**: Leverage Rust's type system and sqlx compile-time checked queries

## Component Layers

```
┌─────────────────────────────────────────────┐
│                 API Layer                    │
│   (Axum handlers - HTTP concerns only)       │
├─────────────────────────────────────────────┤
│              Service Layer                   │
│   (Business logic, orchestration)            │
├─────────────────────────────────────────────┤
│              Domain Layer                    │
│   (Entities, value objects, errors)          │
├─────────────────────────────────────────────┤
│           Infrastructure Layer               │
│   (Storage traits, DB repositories)          │
└─────────────────────────────────────────────┘
```

## Storage Strategy

### Local Storage Structure
```
storage/
└── videos/
    ├── ab/
    │   └── cd/
    │       └── abcd1234.../     # Video directory (first 4 chars of UUID)
    │           ├── original.mp4  # Original uploaded file
    │           └── metadata.json # Video metadata backup
```

### Storage Trait Design
The `Storage` trait abstracts all file operations:
- `store_stream`: Upload from stream
- `read_range`: Read specific byte range (for HTTP 206)
- `get_size`: Get file size for Content-Length
- `delete`: Cleanup

This allows seamless migration to S3 by implementing the same trait.

## Video Loming Mechanism

### Upload Flow
1. Client streams multipart upload to `/api/upload`
2. Backend validates file size (< 1GB) and format
3. Generate UUID token for the video
4. Stream file to storage layer
5. Store metadata in SQLite
6. Return shareable URL token

### Streaming Flow
1. Client requests `/api/stream/{token}`
2. Backend validates token and retrieves metadata
3. Check for `Range` header
4. Stream requested byte range using `tokio::fs` or storage trait
5. Return `206 Partial Content` with appropriate headers

### Range Request Support
```
Request:  Range: bytes=0-1023
Response: 206 Partial Content
          Content-Range: bytes 0-1023/4587201
          Content-Length: 1024
```

## Privacy Model

- Videos are unlisted (no directory/index)
- 128-bit random UUID tokens (extremely low collision probability)
- No authentication required (simple private links)
- Future: Token expiration, password protection

## Database Schema

```sql
CREATE TABLE videos (
    id          TEXT PRIMARY KEY,     -- UUID token
    filename    TEXT NOT NULL,        -- Original filename
    content_type TEXT NOT NULL,       -- MIME type
    size_bytes  INTEGER NOT NULL,     -- File size
    storage_path TEXT NOT NULL,       -- Path in storage layer
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);
```
https://dev.to/rajasekhar_beemireddy_cb8/understanding-web-streaming-chunked-vs-normal-content-delivery-3fm8
## Scalability Considerations

### Current Design (Single Instance)
- SQLite database
- Local filesystem storage
- Suitable for personal use, small scale

### Scaling Path
1. **Database**: Replace SQLite with PostgreSQL
   - Change connection pool, no query changes needed (sqlx)
   
2. **Storage**: Implement S3 storage trait
   - Same interface, different implementation
   - Use S3 range requests for streaming
   
3. **Horizontal Scaling**:
   - Stateless backend (no local session state)
   - Move SQLite to PostgreSQL
   - Use S3 for storage
   - Add CDN for video delivery

## Tradeoffs Made

| Decision | Tradeoff | Rationale |
|----------|----------|-----------|
| SQLite over Postgres | Simplicity vs concurrency | Easy setup, can migrate later |
| No transcoding | Simplicity vs compatibility | Assume modern browser formats |
| No auth system | Simplicity vs security | Private links sufficient for MVP |
| Direct streaming | Simplicity vs adaptive bitrate | HTTP range requests work well |
| UUID tokens | Security vs usability | Unlisted links are private enough |

## Future Improvements

1. **Video Transcoding**: HLS/DASH for adaptive streaming
2. **Thumbnail Generation**: Extract poster frames
3. **Token Expiration**: Time-limited access
4. **Rate Limiting**: Prevent abuse
5. **Progress Tracking**: Upload/download progress in DB
6. **Metrics**: Prometheus/Grafana integration
