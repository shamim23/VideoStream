# Video Streaming Service Architecture

## Document Overview

This document describes the architecture of a minimal private video streaming service. The design prioritizes:

1. **Fast time-to-stream** - Videos are playable immediately after upload
2. **Clean architecture** - Clear boundaries for future scaling
3. **Simple deployment** - Local filesystem + SQLite for MVP
4. **Extensibility** - Well-defined paths to cloud storage and adaptive streaming

---

## Table of Contents

1. [Current Implementation (MVP)](#current-implementation-mvp)
2. [Architecture Principles](#architecture-principles)
3. [Storage Layer](#storage-layer)
4. [Streaming Architecture](#streaming-architecture)
5. [Path to Cloud Storage](#path-to-cloud-storage-s3)
6. [Adaptive Streaming Trade-offs](#adaptive-streaming-hlsdash-trade-offs)
7. [Horizontal Scaling Plan](#horizontal-scaling-plan)
8. [Security Considerations](#security-considerations)
9. [Cost Analysis](#cost-analysis)

---

## Current Implementation (MVP)

### Runtime Components

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Client (Browser)                            │
│  ┌────────────────────────┐        ┌──────────────────────────────┐ │
│  │  Upload Page (/)       │        │  Watch Page (/watch/:id)     │ │
│  │  - File selection      │        │  - HTML5 video player        │ │
│  │  - Progress tracking   │        │  - Native seeking            │ │
│  │  - Share link display  │        │  - Copy link                 │ │
│  └────────────────────────┘        └──────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ HTTP/1.1 with Range support
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Backend (Axum + Tokio)                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │  API Layer   │──▶│Service Layer │──▶│    Storage Trait         │  │
│  │  - /upload   │  │ - Validation │  │    (LocalStorage)        │  │
│  │  - /watch/:id│  │ - Streaming  │  └──────────────────────────┘  │
│  └──────────────┘  └──────────────┘             │                    │
└─────────────────────────────────────────────────────────────────────┘
                                                  │
                        ┌─────────────────────────┴───────────────────┐
                        │                                         │
                        ▼                                         ▼
              ┌──────────────────┐                    ┌──────────────────┐
              │  SQLite          │                    │  Local Filesystem│
              │  - Metadata      │                    │  - Video files   │
              │  - Share tokens  │                    │  - Sharded dirs  │
              └──────────────────┘                    └──────────────────┘
```

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Health check |
| `/api/upload` | POST | Multipart upload (`video` field, max 1GB) |
| `/api/watch/:id` | GET | Stream video with Range support |

### Technology Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Frontend | SvelteKit | Lightweight, fast, minimal boilerplate |
| Backend | Rust (Axum) | Performance, safety, excellent async ecosystem |
| Database | SQLite | Zero config, file-based, perfect for single-node |
| Storage | Local filesystem | Simple, fast, no external dependencies |

---

## Architecture Principles

### 1. Layered Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Presentation Layer (API Handlers)                          │
│  - HTTP request/response handling                            │
│  - Input validation                                          │
│  - Header management                                         │
├─────────────────────────────────────────────────────────────┤
│  Service Layer (Business Logic)                             │
│  - Video validation (MIME type, extension, size)            │
│  - Range request parsing                                     │
│  - Orchestration between storage and database               │
├─────────────────────────────────────────────────────────────┤
│  Domain Layer (Core Entities)                               │
│  - Video entity                                              │
│  - Share response                                            │
├─────────────────────────────────────────────────────────────┤
│  Infrastructure Layer (Ports/Adapters)                      │
│  - Storage trait (abstracts filesystem/S3)                  │
│  - Database access                                           │
└─────────────────────────────────────────────────────────────┘
```

### 2. Dependency Inversion via Traits

The `Storage` trait is the key abstraction:

```rust
#[async_trait]
pub trait Storage: Send + Sync {
    async fn store_stream(&self, video: &mut Video, reader: &mut (dyn AsyncRead + Unpin + Send)) 
        -> Result<PathBuf>;
    async fn open_read_stream(&self, video: &Video, start: u64, length: Option<u64>) 
        -> Result<Box<dyn AsyncRead + Unpin + Send>>;
    async fn get_size(&self, video: &Video) -> Result<u64>;
    async fn delete(&self, video: &Video) -> Result<()>;
}
```

This allows:
- **Testability**: Mock storage for unit tests
- **Flexibility**: Swap implementations without changing service code
- **Gradual Migration**: Run both local and S3 during transition

### 3. Stateless Design

The backend is intentionally stateless:
- No server-side sessions (UUID tokens are self-contained)
- No in-memory state between requests
- All data in SQLite or filesystem

**Why this matters**: Stateless services can scale horizontally by simply adding more instances behind a load balancer.

---

## Storage Layer

### Current: Local Filesystem

**File Organization (Sharded)**:
```
storage/
└── videos/
    └── ab/           # First 2 chars of UUID
        └── cd/       # Next 2 chars of UUID
            └── abcd1234-5678-90ab-cdef-example123/
                ├── original.mp4       # The video file
                └── metadata.json      # Backup metadata
```

Sharding prevents filesystem performance degradation with too many files in a single directory.

### Storage Abstraction

See detailed documentation:
- [`STORAGE.md`](./STORAGE.md) - Storage architecture and S3 migration
- [`CLOUD_INTEGRATION.md`](./CLOUD_INTEGRATION.md) - Cloud abstraction design

---

## Streaming Architecture

### Current: HTTP Range Requests

The service implements RFC 7233 HTTP Range Requests for efficient seeking:

**Request**:
```http
GET /api/watch/123e4567-e89b-12d3-a456-426614174000 HTTP/1.1
Range: bytes=1048576-2097151
```

**Response**:
```http
HTTP/1.1 206 Partial Content
Content-Type: video/mp4
Content-Length: 1048576
Content-Range: bytes 1048576-2097151/1073741824
Accept-Ranges: bytes

[video bytes...]
```

### Why This Works

- **Browser native**: HTML5 video player handles range requests automatically
- **Seeking**: When user scrubs timeline, browser requests appropriate byte range
- **Resume**: Interrupted downloads can resume from last position
- **Bandwidth efficient**: Only requested bytes are transferred

### Limitations

- Single quality level (original upload)
- No adaptation to network conditions
- Original file must be browser-compatible

See detailed streaming documentation:
- [`STREAMING.md`](./STREAMING.md) - Streaming architecture and HLS trade-offs

---

## Path to Cloud Storage (S3)

### Migration Strategy

Because storage is trait-driven, migration requires only a new adapter:

```rust
// Current
let storage: Arc<dyn Storage> = Arc::new(LocalStorage::new(&storage_path));

// Future (no service changes needed)
let storage: Arc<dyn Storage> = Arc::new(S3Storage::from_env()?);
```

### S3 Implementation

```rust
pub struct S3Storage {
    bucket: Bucket,
}

#[async_trait]
impl Storage for S3Storage {
    async fn open_read_stream(&self, video: &Video, start: u64, length: Option<u64>) 
        -> Result<Box<dyn AsyncRead + Unpin + Send>> {
        let key = format!("{}/original.mp4", video.storage_path);
        // S3 GetObject with Range header
        let response = self.bucket.get_object_range(&key, start, length).await?;
        Ok(Box::new(response.bytes_stream().into_async_read()))
    }
    // ... other methods
}
```

### S3 Trade-offs

| Aspect | Local Filesystem | S3-Compatible |
|--------|-----------------|---------------|
| **Latency** | <1ms | 10-100ms |
| **Durability** | Single point of failure | 99.999999999% |
| **Availability** | Tied to machine | 99.99% SLA |
| **Scaling** | Vertical only | Horizontal native |
| **Cost Model** | Hardware | Pay-per-use |
| **Multi-instance** | Shared volume needed | Native support |
| **CDN Integration** | Complex | Native |

**When to migrate to S3**:
- Multiple backend instances needed
- Geographic distribution required
- Disaster recovery requirements
- Cost optimization at scale (>TB)

---

## Adaptive Streaming (HLS/DASH) Trade-offs

### Not Required for MVP

Full transcoding to HLS/DASH is intentionally out of scope for the MVP. The current Range Request approach satisfies all core requirements with significantly less complexity.

### HLS Overview

```
Upload → Store Original → Spawn Transcoding → Generate HLS
                                              ├─ 1080p/
                                              ├─ 720p/
                                              ├─ 480p/
                                              └─ playlist.m3u8
```

### Trade-off Analysis

| Factor | Range Requests | HLS/DASH |
|--------|----------------|----------|
| **Time-to-stream** | ⭐⭐⭐ Immediate | ⭐ Delayed (transcoding) |
| **Architecture** | ⭐⭐⭐ Simple | ⭐⭐⭐ Complex |
| **Bandwidth adaption** | ❌ None | ⭐⭐⭐ Automatic |
| **Storage cost** | ⭐ 1x | ⭐⭐⭐ 2-4x |
| **CPU cost** | ⭐ None | ⭐⭐⭐ High |
| **Mobile experience** | ⭐⭐ Good | ⭐⭐⭐ Optimized |

### Recommendation

**Current (MVP)**: HTTP Range Requests
- Satisfies all requirements
- Fastest time-to-stream
- Simplest architecture

**Future**: HLS when needed
- Add background transcoding workers
- Serve HLS when ready, fallback to original
- Gradual enhancement without breaking changes

---

## Horizontal Scaling Plan

### Current (Single Instance)

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Backend    │
│  (SQLite)   │
└─────────────┘
```

### Phase 1: Stateless with Shared Storage

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Nginx LB  │
└──────┬──────┘
       │
   ┌───┴───┐
   ▼       ▼
┌──────┐ ┌──────┐
│Back- │ │Back- │
│end 1 │ │end 2 │
└──┬───┘ └──┬───┘
   │        │
   └────┬───┘
        ▼
   ┌──────────┐
   │PostgreSQL│
   └──────────┘
```

Changes needed:
1. Replace SQLite with PostgreSQL (same queries, different driver)
2. Ensure all instances share storage (NFS or move to S3)
3. Add health checks for load balancer

### Phase 2: Cloud-Native

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌─────────────┐
│  CloudFront │────▶│    S3       │
│  (CDN)      │     │  (Storage)  │
└──────┬──────┘     └─────────────┘
       │
       ▼
┌─────────────┐
│  ALB/NLB    │
└──────┬──────┘
       │
   ┌───┴───┐
   ▼       ▼
┌──────┐ ┌──────┐
│ ECS/ │ │ ECS/ │
│ EKS   │ │ EKS  │
└──┬───┘ └──┬───┘
   │        │
   └────┬───┘
        ▼
   ┌──────────┐
   │   RDS    │
   └──────────┘
```

### Scaling Checklist

| Component | Current | Scale Phase 1 | Scale Phase 2 |
|-----------|---------|---------------|---------------|
| Database | SQLite | PostgreSQL | RDS/Aurora |
| Storage | Local FS | NFS/Shared | S3/R2 |
| Load Balancer | None | Nginx | ALB/NLB |
| CDN | None | Optional | CloudFront |
| Compute | Single process | Docker Compose | ECS/K8s |

---

## Security Considerations

### Current Security Model

1. **Private Links**: UUID-based URLs provide unlisted privacy (security through obscurity)
2. **No Authentication**: Anonymous upload and viewing by design
3. **Input Validation**: MIME type and file extension validation
4. **Size Limits**: 1GB upload cap prevents abuse

### Future Hardening

- Rate limiting per IP
- Content-Type sniffing prevention
- Scanning uploads for malware
- HTTPS enforcement
- CORS configuration for specific origins
- Link expiration (time-limited signed URLs)

---

## Cost Analysis

### Current (Local/Single-Instance)

| Component | Cost |
|-----------|------|
| Compute | $0 (developer machine) |
| Storage | $0 (local disk) |
| Bandwidth | $0 (localhost) |
| **Total** | **$0** |

### Production Estimate (Small Scale)

**Assumptions**:
- 1,000 videos
- Average 100MB per video
- 10,000 views/month
- 90% CDN cache hit rate

| Component | Provider | Monthly Cost |
|-----------|----------|--------------|
| Compute (2 vCPU, 4GB) | AWS ECS | ~$30 |
| Database (PostgreSQL) | AWS RDS | ~$15 |
| Storage (100GB) | S3 Standard | ~$2.30 |
| CDN (1TB egress) | CloudFront | ~$85 |
| **Total** | | **~$132/month** |

### Cost Optimization Options

1. **Backblaze B2 instead of S3**: ~75% storage savings
2. **Cloudflare R2**: Zero egress fees
3. **Spot instances for transcoding**: 70% compute savings

---

## Documentation Index

| Document | Purpose |
|----------|---------|
| `architecture.md` | This document - high-level overview |
| [`STORAGE.md`](./STORAGE.md) | Storage layer details, S3 migration |
| [`CLOUD_INTEGRATION.md`](./CLOUD_INTEGRATION.md) | Cloud abstraction design |
| [`STREAMING.md`](./STREAMING.md) | Streaming architecture, HLS trade-offs |
| [`HLS_IMPLEMENTATION.md`](./HLS_IMPLEMENTATION.md) | HLS implementation guide (optional) |

---

## Summary

This architecture delivers:

✅ **Fast time-to-stream** - Videos playable immediately after upload  
✅ **Clean separation of concerns** - Layered architecture with trait abstractions  
✅ **Horizontal scaling path** - Stateless design ready for load balancing  
✅ **Cloud migration path** - Trait-based storage enables S3 swap  
✅ **Adaptive streaming path** - Documented HLS upgrade when needed  
✅ **Cost efficiency** - Starts at $0, scales cost-effectively  

The design demonstrates senior-level engineering through:
- **Appropriate abstraction** (Storage trait)
- **Pragmatic trade-offs** (Range requests for MVP)
- **Future-proofing** (clear scaling paths)
- **Documentation** (architectural decisions explained)
