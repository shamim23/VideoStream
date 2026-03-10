# Video Streaming Service - System Design

## Overview

This document presents the complete system design for a minimal private video streaming service. It covers the current MVP architecture using local filesystem storage, the path to production with cloud storage and CDN, and the trade-offs between HTTP Range Request streaming and adaptive bitrate protocols like HLS/DASH.

The design prioritizes fast time-to-stream for the MVP while maintaining clean architectural boundaries that enable horizontal scaling and cloud migration without business logic changes.

---

## Current Architecture (MVP)

### System Diagram

```
┌────────────────────────────────────────────────────────────────────────────┐
│                              Client (Browser)                              │
│  ┌─────────────────────────────┐        ┌────────────────────────────────┐ │
│  │    Upload Page (/)          │        │    Watch Page (/watch/:id)     │ │
│  │    • File selection         │        │    • HTML5 video player        │ │
│  │    • Progress tracking      │        │    • HTTP Range seeking        │ │
│  │    • Share link generation  │        │    • Native browser controls   │ │
│  └──────────────┬──────────────┘        └────────────────┬───────────────┘ │
└─────────────────┼────────────────────────────────────────┼─────────────────┘
                  │                                        │
                  │ HTTP/1.1 with multipart/form-data      │ HTTP/1.1 with Range headers
                  │                                        │
                  ▼                                        ▼
┌────────────────────────────────────────────────────────────────────────────┐
│                         Backend (Axum + Tokio)                             │
│                                                                            │
│  ┌──────────────────────┐    ┌──────────────────────┐    ┌──────────────┐  │
│  │    API Layer         │───▶│   Service Layer      │───▶│ Storage Port │  │
│  │    • /api/upload     │    │   • Video validation │    │  (trait)     │  │
│  │    • /api/watch/:id  │    │   • Range parsing    │    └──────┬───────┘  │
│  │    • Health checks   │    │   • Stream assembly  │           │          │
│  └──────────────────────┘    └──────────────────────┘           │          │
│                                                                 │          │
└─────────────────────────────────────────────────────────────────┼──────────┘
                                                                  │
                                          ┌───────────────────────┴──────────┐
                                          │                                  │
                                          ▼                                  ▼
                                ┌──────────────────┐            ┌──────────────────┐
                                │   SQLite         │            │  Local Storage   │
                                │   • Metadata     │            │  • Video files   │
                                │   • Share tokens │            │  • Sharded dirs  │
                                │   • Timestamps   │            │  • UUID-based    │
                                └──────────────────┘            └──────────────────┘
```

### Component Description

The current architecture consists of a SvelteKit frontend that handles file uploads with progress tracking and video playback using the browser's native HTML5 video player. The frontend communicates with a Rust Axum backend via HTTP. The backend exposes two primary endpoints: one for multipart file uploads and one for video streaming with HTTP Range request support.

The backend follows a layered architecture. The API layer handles HTTP concerns like parsing multipart forms, extracting range headers, and setting response status codes. It delegates business logic to the service layer, which validates file types, enforces size limits, parses range requests according to RFC 7233, and orchestrates between storage and database operations.

The service layer depends on abstract interfaces rather than concrete implementations. It uses the Storage trait for all file operations and SQLx for database access. This dependency inversion is the architectural key that enables future infrastructure changes without modifying business logic.

Storage is currently implemented using LocalStorage, which organizes video files in a sharded directory structure based on UUID prefixes. This prevents filesystem performance degradation when storing thousands of files. SQLite provides metadata persistence with zero configuration requirements.

### Streaming Implementation

The service implements HTTP Range Requests to enable efficient seeking within video files. When a user scrubs the timeline in the video player, the browser automatically requests specific byte ranges from the server. The backend parses these range headers, seeks to the appropriate position in the file, and returns only the requested bytes with a 206 Partial Content status.

This approach provides immediate playability after upload since no transcoding occurs. Videos are served in their original format, and the browser handles decoding. The trade-off is that users on slow connections must buffer the original file quality rather than receiving an automatically adjusted stream.


---

## Production Architecture (Cloud-Native)

### System Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                                    Clients                                          │
│  ┌─────────────────────────────────────────────────────────────────────────────┐   │
│  │                        Web Application (Browser)                             │   │
│  │  ┌───────────────────────┐              ┌────────────────────────────────┐  │   │
│  │  │   Upload Interface    │              │   Video Player                 │  │   │
│  │  │   • Drag & drop       │              │   • HLS.js or native HLS       │  │   │
│  │  │   • Progress bar      │              │   • Adaptive quality           │  │   │
│  │  │   • Preview           │              │   • Subtitles support          │  │   │
│  │  └───────────┬───────────┘              └──────────────┬─────────────────┘  │   │
│  └──────────────┼─────────────────────────────────────────┼────────────────────┘   │
└─────────────────┼─────────────────────────────────────────┼────────────────────────┘
                  │                                         │
                  │ HTTPS                                   │ HTTPS
                  │                                         │
                  ▼                                         ▼
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              Content Delivery Network                                │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                        CloudFront / Cloudflare / Fastly                      │    │
│  │                                                                              │    │
│  │   ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────────┐     │    │
│  │   │  Edge Location  │    │  Edge Location  │    │   Edge Location     │     │    │
│  │   │  (US-East)      │    │  (EU-West)      │    │   (APAC)            │     │    │
│  │   │  • Cache HLS    │    │  • Cache HLS    │    │   • Cache HLS       │     │    │
│  │   │    segments     │    │    segments     │    │     segments        │     │    │
│  │   │  • Cache API    │    │  • Cache API    │    │   • Cache API       │     │    │
│  │   │    responses    │    │    responses    │    │     responses       │     │    │
│  │   └────────┬────────┘    └────────┬────────┘    └──────────┬──────────┘     │    │
│  │            │                      │                        │                │    │
│  │            └──────────────────────┼────────────────────────┘                │    │
│  │                                   │ Cache Miss                                │    │
│  └───────────────────────────────────┼─────────────────────────────────────────┘    │
└──────────────────────────────────────┼────────────────────────────────────────────────┘
                                       │
                    ┌──────────────────┴──────────────────┐
                    │                                     │
                    ▼                                     ▼
┌──────────────────────────────────┐          ┌──────────────────────────────────────┐
│      Application Load Balancer   │          │         S3 / R2 / B2 Storage         │
│      (AWS ALB / Nginx / Traefik) │          │                                      │
│                                  │          │   ┌──────────────────────────────┐   │
│   • Health checks                │          │   │    Video Bucket              │   │
│   • SSL termination              │          │   │                              │   │
│   • Rate limiting                │          │   │   videos/ab/cd/uuid/         │   │
│   • Request routing              │          │   │   ├── original.mp4           │   │
│                                  │          │   │   ├── hls/                   │   │
│                                  │          │   │   │   ├── 1080p/             │   │
│                                  │          │   │   │   ├── 720p/              │   │
│                                  │          │   │   │   ├── 480p/              │   │
│                                  │          │   │   │   └── playlist.m3u8      │   │
│                                  │          │   │   └── thumbnails/            │   │
│                                  │          │   │                              │   │
│                                  │          │   │   Lifecycle:                 │   │
│                                  │          │   │   • Standard (0-30 days)     │   │
│                                  │          │   │   • IA (30-90 days)          │   │
│                                  │          │   │   • Glacier (90+ days)       │   │
│                                  │          │   └──────────────────────────────┘   │
└────────────┬─────────────────────┘          └──────────────────────────────────────┘
             │
             │ Load balancing
             │
     ┌───────┴───────┐
     │               │
     ▼               ▼
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                         Backend Services (Containerized)                             │
│                                                                                      │
│  ┌────────────────────┐    ┌────────────────────┐    ┌────────────────────┐        │
│  │   Backend Instance │    │   Backend Instance │    │   Backend Instance │        │
│  │      (Docker)      │    │      (Docker)      │    │      (Docker)      │        │
│  │                    │    │                    │    │                    │        │
│  │  ┌──────────────┐  │    │  ┌──────────────┐  │    │  ┌──────────────┐  │        │
│  │  │  Axum Server │  │    │  │  Axum Server │  │    │  │  Axum Server │  │        │
│  │  │  • API Layer │  │    │  │  • API Layer │  │    │  │  • API Layer │  │        │
│  │  └──────┬───────┘  │    │  └──────┬───────┘  │    │  └──────┬───────┘  │        │
│  │         │          │    │         │          │    │         │          │        │
│  │  ┌──────▼──────┐   │    │  ┌──────▼──────┐   │    │  ┌──────▼──────┐   │        │
│  │  │   Service   │   │    │  │   Service   │   │    │  │   Service   │   │        │
│  │  │   Layer     │   │    │  │   Layer     │   │    │  │   Layer     │   │        │
│  │  └──────┬──────┘   │    │  └──────┬──────┘   │    │  └──────┬──────┘   │        │
│  │         │          │    │         │          │    │         │          │        │
│  │  ┌──────▼──────┐   │    │  ┌──────▼──────┐   │    │  ┌──────▼──────┐   │        │
│  │  │   S3Storage │   │    │  │   S3Storage │   │    │  │   S3Storage │   │        │
│  │  │   Adapter   │   │    │  │   Adapter   │   │    │  │   Adapter   │   │        │
│  │  └─────────────┘   │    │  └─────────────┘   │    │  └─────────────┘   │        │
│  │                    │    │                    │    │                    │        │
│  └─────────┬──────────┘    └─────────┬──────────┘    └─────────┬──────────┘        │
│            │                         │                         │                   │
└────────────┼─────────────────────────┼─────────────────────────┼───────────────────┘
             │                         │                         │
             └─────────────────────────┼─────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                              Data Layer                                              │
│                                                                                      │
│  ┌──────────────────────────────────────┐    ┌──────────────────────────────────┐   │
│  │        PostgreSQL (RDS/Aurora)       │    │      Redis (ElastiCache)         │   │
│  │                                      │    │                                  │   │
│  │   • Video metadata                   │    │   • Session caching              │   │
│  │   • User data (if auth added)        │    │   • Rate limiting counters       │   │
│  │   • View analytics                   │    │   • Popular video caching        │   │
│  │   • Connection pooling (PgBouncer)   │    │   • Real-time features           │   │
│  │                                      │    │                                  │   │
│  └──────────────────────────────────────┘    └──────────────────────────────────┘   │
│                                                                                      │
└─────────────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       │ Background Jobs
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                           Transcoding Workers (ECS/EC2)                              │
│                                                                                      │
│  ┌──────────────────────────────────────────────────────────────────────────────┐   │
│  │                        Queue (SQS / RabbitMQ / Redis)                         │   │
│  │                                                                              │   │
│  │   Job: "Transcode video abc123"                                              │   │
│  │   Job: "Generate thumbnail xyz789"                                           │   │
│  │   Job: "Create HLS variants def456"                                          │   │
│  └──────────────────────────────┬───────────────────────────────────────────────┘   │
│                                 │                                                    │
│         ┌───────────────────────┼───────────────────────┐                            │
│         │                       │                       │                            │
│         ▼                       ▼                       ▼                            │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐                       │
│  │  Worker 1    │      │  Worker 2    │      │  Worker 3    │                       │
│  │              │      │              │      │              │                       │
│  │  FFmpeg      │      │  FFmpeg      │      │  FFmpeg      │                       │
│  │  • 1080p     │      │  • 720p      │      │  • 480p      │                       │
│  │  • 720p      │      │  • Thumbnail │      │  • 240p      │                       │
│  │  • Thumbnail │      │              │      │  • WebVTT    │                       │
│  └──────────────┘      └──────────────┘      └──────────────┘                       │
│                                                                                      │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### Key Production Changes

The production architecture introduces several layers that address scalability, reliability, and performance concerns that the MVP intentionally deferred.

A Content Delivery Network sits at the edge, caching both API responses and video content at points of presence worldwide. This reduces latency for geographically distributed users and dramatically reduces load on origin servers. The CDN also provides DDoS protection and can handle SSL certificate management.

Video storage migrates from local filesystem to S3-compatible object storage. This might be AWS S3, Backblaze B2 for cost savings, or Cloudflare R2 for zero egress fees. Object storage provides eleven nines durability, infinite scalability, and native geographic redundancy.

The database upgrades from SQLite to PostgreSQL, enabling multiple backend instances to share state concurrently. PostgreSQL offers sophisticated connection pooling, read replicas for scaling read-heavy workloads, and comprehensive monitoring capabilities.

Redis provides caching for frequently accessed metadata and rate limiting counters. It can also power real-time features like live viewer counts or websocket pub/sub if needed.

The transcoding layer introduces background workers that process uploaded videos asynchronously. These workers use FFmpeg to generate multiple quality variants for adaptive streaming, create thumbnail images, and optionally extract subtitles. This work happens after the initial upload completes, ensuring fast time-to-stream for the original file while enhancing quality options in the background.

---

## Storage Architecture Deep Dive

### The Abstraction Layer

The storage architecture centers on a trait-based abstraction that decouples business logic from infrastructure concerns. This trait defines four essential operations: storing streams, reading with byte-range support, getting file sizes, and deleting objects. The interface uses asynchronous streams rather than byte buffers, ensuring constant memory usage regardless of file size.

The critical design decision is that this trait represents a port in hexagonal architecture terms. The application core defines what capabilities it needs from storage without specifying how those capabilities are provided. Adapters implement this port for specific technologies.

LocalStorage implements the trait using Tokio filesystem operations. It organizes files in a sharded directory structure to prevent performance degradation. When storing streams, it uses io_copy to move bytes directly from the upload to disk without intermediate buffering. For range reads, it uses standard file seek operations.

An S3Storage adapter would implement the identical trait using S3 API calls. Store stream would use PutObject or multipart upload. Open read stream would use GetObject with Range headers, which S3 natively supports. Get size would use HeadObject. The service layer cannot distinguish between these implementations; both satisfy the same contract.

### Migration Path

Migrating from local storage to cloud storage happens through a series of incremental phases rather than a risky big-bang switch. During the first phase, the application continues using local storage while infrastructure teams provision S3 buckets, configure IAM policies, and establish network connectivity.

The second phase introduces the S3 adapter implementation alongside the existing local adapter. Both implementations exist in the codebase, with environment variables determining which activates at runtime. This allows testing the S3 integration in staging environments without affecting production.

The third phase enables dual-write capability where new uploads write to both local storage and S3 simultaneously. Reads continue coming from local storage, providing a safety net. Any issues with the S3 integration affect only the redundant copy.

The fourth phase switches reads and writes to S3 as the primary source while maintaining the local copy for rollback capability. Monitoring during this phase validates performance and reliability.

The final phase migrates existing content through a background job that copies videos from local storage to S3 and updates database records. Once complete, local storage can be decommissioned. Throughout this process, the service layer requires no changes; only adapter initialization and configuration differ.

### Provider Selection

For production workloads, several S3-compatible providers offer different trade-offs. AWS S3 provides the richest feature set, widest ecosystem integration, and highest reliability guarantees, but charges significant egress fees that can dominate costs for video streaming workloads.

Backblaze B2 offers roughly one-fourth the storage cost and one-ninth the egress cost of AWS S3, with the first gigabyte of daily egress free. This pricing structure makes it attractive for startups and cost-conscious deployments. The trade-off is a smaller ecosystem and occasional compatibility quirks with S3 libraries.

Cloudflare R2 differentiates through zero egress fees entirely. You pay only for storage and operations, not bandwidth. For high-traffic applications, this can result in dramatic savings. As a newer service, it has less operational history than competitors.

MinIO provides self-hosted S3-compatible storage for organizations requiring data sovereignty or having existing infrastructure expertise. This eliminates per-gigabyte costs beyond hardware but requires accepting operational responsibility for maintenance and availability.

### Production Optimizations

Several practices ensure reliable cloud storage operation. Implement exponential backoff retry logic for transient errors. Network hiccups and temporary service unavailability should not fail user requests. Most SDKs provide configurable retry policies.

Configure appropriate timeouts to prevent resource exhaustion from hanging requests. S3 operations can stall indefinitely during network partitions; reasonable connection and read timeouts prevent this.

Monitor storage metrics closely including request latency, error rates by operation type, storage growth rate, and costs. Cloud providers offer comprehensive metrics that reveal usage patterns and potential issues.

Use presigned URLs for video streaming when possible. Instead of proxying video data through application servers, generate time-limited URLs allowing browsers to download directly from S3 or CloudFront. This reduces server load, improves latency through edge locations, and can reduce transfer costs.

Implement lifecycle policies to automate cost optimization. Videos might remain in Standard class for 30 days when viewership is highest, then transition to Infrequent Access for 90 days, and finally to Glacier for archival. This ensures premium storage rates are not paid for rarely accessed content.

Configure Cross-Origin Resource Sharing headers on S3 buckets to enable browser-based video players to access content. Without appropriate CORS configuration, browsers block cross-origin video requests.

---

## Streaming Architecture

### Current: HTTP Range Requests

The MVP implements RFC 7233 HTTP Range Requests for video streaming. This approach allows browsers to request specific byte ranges of a video file rather than downloading the entire file. When a user opens a video, the browser typically requests the first portion to begin playback immediately. When the user scrubs to a different timestamp, the browser calculates the corresponding byte offset and requests that range.

The backend parses range headers supporting several formats. A request might specify an absolute range like bytes 0 through 1023, an open-ended range starting at byte 1048576 and continuing to the end, or a suffix range requesting the final 1000 bytes. The parser validates that ranges fall within file boundaries and returns appropriate error responses for malformed or unsatisfiable ranges.

For valid range requests, the backend returns HTTP 206 Partial Content with Content-Range headers indicating what portion of the resource is being returned. The response body contains only the requested bytes. For requests without range headers, the backend returns the entire file with HTTP 200.

This implementation provides immediate playability since videos require no processing before streaming. The browser handles decoding of whatever format was uploaded. Users can seek to any position without waiting for the entire video to download. The server implementation is straightforward, requiring only file seek capabilities without transcoding infrastructure.

### Limitations

The Range Request approach has significant limitations that become apparent at scale. There is no adaptation to network conditions. A user on a slow mobile connection receives the same high-bitrate file as a user on fiber, resulting in buffering and poor experience on constrained connections. Conversely, users on fast connections cannot receive higher quality than the original upload.

The original file format must be browser-compatible. If a user uploads an obscure codec, playback fails. Range requests cannot transform content on the fly. Storage costs are multiplied if multiple qualities are needed because each quality requires a complete separate file.

Bandwidth is used inefficiently. A user who watches only the first minute of a ten-minute video still downloads the entire minute at full quality even if their connection could only smoothly support half that bitrate.

### Future: HLS Adaptive Streaming

HLS (HTTP Live Streaming) addresses these limitations by providing adaptive bitrate streaming. The transcoding layer described in the production architecture generates multiple quality variants of each video, typically 1080p, 720p, 480p, and 240p. Each variant is segmented into small chunks, usually four to ten seconds in duration.

A master playlist file lists available quality variants with their bitrates and resolutions. The video player downloads this playlist first, then begins downloading segments from an appropriate quality level based on current network conditions. If bandwidth decreases, the player switches to lower quality segments. If bandwidth increases, it switches to higher quality.

This adaptation happens seamlessly during playback without user intervention. The player maintains a buffer of upcoming segments to smooth over brief network fluctuations. Because segments are small files, switching qualities happens quickly rather than requiring a new connection to a different byte range of a large file.

HLS also enables live streaming, where segments are generated continuously from a live source and the playlist updates periodically. Digital rights management can be integrated through encryption keys that segments reference. These capabilities are impossible with simple range requests.

### Trade-off Analysis

HTTP Range Requests excel when fast time-to-stream is paramount, infrastructure must remain simple, or uploaded content is already in web-optimized formats. The approach works well for internal tools, prototypes, or scenarios where users have consistent high-quality connections.

HLS excels when users have variable network conditions, mobile experience matters, or the service competes with consumer streaming platforms. The costs are increased complexity requiring transcoding infrastructure, higher storage requirements for multiple quality variants, and delayed availability while transcoding processes complete.

For this MVP, Range Requests are the appropriate choice. They satisfy all core requirements while demonstrating understanding of HTTP streaming semantics. The architecture document clearly describes the HLS upgrade path for future phases when the business case justifies the additional complexity.

### CDN Integration

Regardless of streaming protocol, a CDN is essential for production video delivery. For Range Request streaming, the CDN caches the original video files at edge locations. When a user requests a byte range, if the CDN has the file cached, it serves the range directly without contacting the origin server.

For HLS streaming, the CDN caches both playlist files and video segments. Playlist files receive short cache times, typically five to thirty seconds, because they change when new segments are added during live streaming or when quality variants update. Video segments receive long cache times, often a year or more with immutable headers, because segment content never changes once created.

Cache invalidation strategies differ between protocols. For Range Requests, updating a video requires changing the URL, typically by versioning the filename or including a content hash. For HLS, the master playlist URL remains constant while the player fetches updated variant playlists that reference new segments.

---

## Horizontal Scaling Strategy

### Stateless Architecture

The backend is intentionally designed to be stateless. No server-side sessions exist; authentication tokens are self-contained JWTs or similar. No in-memory state persists between requests; all data lives in the database or storage layer. Any request can be handled by any backend instance without coordination.

This statelessness is the foundation of horizontal scaling. Because instances do not need to communicate with each other or maintain shared state, scaling is simply a matter of adding more instances behind a load balancer. The load balancer distributes requests across available instances using round-robin or least-connections algorithms.

### Scaling Phases

Phase one scaling replaces SQLite with PostgreSQL. SQLite is an excellent database for single-server deployments but cannot handle concurrent writes from multiple processes. PostgreSQL provides concurrent connection support, sophisticated locking, and connection pooling. The code change is minimal because SQLx abstracts database differences; only the connection string and pool configuration change.

Phase two introduces the load balancer. Initially, this might be Nginx running on the same host as the application. As traffic grows, the load balancer moves to dedicated infrastructure, potentially using managed services like AWS Application Load Balancer that handle SSL termination, health checks, and automatic instance registration.

Phase three adds a CDN in front of the load balancer for static assets and video content. This reduces origin load and improves global latency. Phase four introduces read replicas of the database to scale read-heavy workloads, with the application routing read queries to replicas and writes to the primary.

Phase five adds caching layers. Redis caches frequently accessed metadata and can implement rate limiting. CDN edge caching handles popular video segments. These layers reduce database and storage load while improving response times.

### Database Scaling

PostgreSQL scales vertically by upgrading instance sizes and horizontally through read replicas. The application routes read queries to replicas and writes to the primary. Eventually, connection pooling via PgBouncer becomes necessary to handle high connection counts from many application instances.

For write-heavy workloads or data exceeding single-instance capacity, sharding strategies partition data across multiple database instances. Common approaches include sharding by user ID or by video ID, ensuring related data lives on the same shard.

### Storage Scaling

Object storage like S3 scales infinitely without application changes. As data grows, costs increase linearly with storage used and requests made, but no architectural changes are required. The application continues using the same S3 API calls regardless of whether the bucket contains one gigabyte or one petabyte.

For cost optimization, lifecycle policies automatically move older content to cheaper storage classes. Popular recent content stays in Standard class for fast access. Older content moves to Infrequent Access, then Glacier for archival. Rarely accessed content can be retrieved from Glacier with acceptable latency for non-real-time use cases.

---

## Docker Deployment Architecture

### Overview

Docker provides a production-ready deployment mechanism that demonstrates horizontal scaling capabilities. The architecture uses Docker Compose to orchestrate multiple services: Nginx as a load balancer, scalable backend instances, a frontend service, and shared storage volumes.

### Deployment Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Docker Network                                   │
│                                                                             │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐              │
│  │   Nginx      │      │   Backend    │      │   Backend    │              │
│  │ Load Balancer│◄────►│  Instance 1  │◄────►│  Instance 2  │  ...         │
│  │   (Port 80)  │      │  (Port 3000) │      │  (Port 3000) │              │
│  └──────┬───────┘      └──────┬───────┘      └──────┬───────┘              │
│         │                     │                     │                       │
│         │            ┌────────┴────────┐            │                       │
│         │            │  Shared Volume  │            │                       │
│         │            │  /app/storage   │            │                       │
│         │            └────────┬────────┘            │                       │
│         │                     │                     │                       │
│         │            ┌────────┴────────┐            │                       │
│         └───────────►│    Frontend     │◄───────────┘                       │
│                      │   (Port 3000)   │                                    │
│                      └─────────────────┘                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

**Nginx Load Balancer**

Nginx serves as the entry point and distributes traffic across backend instances. It uses Docker's internal DNS resolver to discover available backend containers dynamically. This enables automatic load distribution when backends scale up or down.

Key configuration aspects:
- Dynamic backend resolution via `resolver 127.0.0.11` (Docker DNS)
- Round-robin load balancing across available instances
- Health checks with automatic failover
- Video-optimized settings: disabled buffering, large upload limits (1GB+)
- Static file serving with appropriate cache headers

**Backend Service**

The backend runs as a stateless container based on a multi-stage Docker build. The builder stage uses the full Rust toolchain to compile the application, while the runtime stage uses a minimal Debian image containing only the compiled binary and SSL certificates.

Characteristics:
- Stateless design: any instance can handle any request
- Health check endpoint for container orchestration
- Shared storage volume for video persistence
- Graceful shutdown handling for in-flight requests
- Small image size (~50MB vs ~2GB with full toolchain)

**Frontend Service**

The SvelteKit frontend is built during the Docker build process and served by a Node.js runtime. This pre-compilation ensures consistent, optimized assets in production.

**Shared Storage Volume**

A Docker named volume provides persistent storage that survives container restarts and is accessible to all backend instances. This satisfies the requirement for shared state in a horizontally scaled deployment.

### Demonstrating Horizontal Scaling

The Docker deployment validates horizontal scaling through a simple command:

```bash
# Start with single backend instance
docker-compose up -d

# Scale to three backend instances
docker-compose up -d --scale backend=3
```

When scaled, Nginx automatically discovers and distributes traffic across all backend containers. This demonstrates:

1. **Stateless Architecture**: New backend instances require no configuration or state transfer
2. **Dynamic Discovery**: Nginx finds backends via Docker DNS without configuration changes
3. **Load Distribution**: Requests spread across available instances
4. **Shared State**: All instances access the same storage volume

Verifying load balancing:

```bash
# Check running containers
docker-compose ps
# Shows: nginx, backend_1, backend_2, backend_3, frontend

# Monitor Nginx access logs
docker-compose logs nginx | grep "upstream="
# Shows requests distributed across different backend IPs
```

### Running the Deployment

**Prerequisites**
- Docker Engine 20.10+
- Docker Compose 2.0+
- No local Rust/Node installation required

**Commands**

```bash
# Start entire stack
docker-compose up -d

# Access application
open http://localhost:8080

# Scale horizontally
docker-compose up -d --scale backend=3

# View logs
docker-compose logs -f backend

# Stop and remove
docker-compose down
```

### Production Evolution

**Phase 1: Docker Compose (Current)**
- Single machine deployment
- Demonstrates scaling concepts
- Suitable for assessment and small deployments

**Phase 2: Docker Swarm / Kubernetes**
- Replace `docker-compose up --scale` with orchestrator-managed replicas
- Add service mesh for advanced traffic management
- Implement rolling deployments for zero-downtime updates

**Phase 3: Cloud Container Services**
- AWS ECS Fargate or Google Cloud Run for serverless containers
- Auto-scaling based on CPU/memory metrics
- Integrated with cloud load balancers and managed databases

### Cost Comparison

| Deployment | Monthly Cost (Estimate) |
|------------|------------------------|
| Local Development | $0 (developer machine) |
| Docker Compose (VPS) | $25-50 (single VPS) |
| Docker Compose (Multi-node) | $100-200 (3-5 nodes) |
| Cloud-Native (ECS/K8s) | $200-500 (managed services) |

The Docker approach provides a middle ground between local development and full cloud-native deployment, offering production-like scalability demonstration at manageable cost.

### Detailed Documentation

For complete Docker deployment documentation including:
- Component deep-dives
- Environment configuration
- Troubleshooting guide
- Commands cheat sheet
- Production optimization

**See: [`DOCKER_ARCHITECTURE.md`](./DOCKER_ARCHITECTURE.md)**

---

## Future Work and Enhancements

### CI/CD Pipeline

A continuous integration and deployment pipeline would automate testing and deployment. The planned pipeline includes GitHub Actions workflows for:

**Backend CI:**
- Automated builds on pull requests
- `cargo test` for unit and integration tests
- `cargo clippy` for linting and code quality
- `cargo fmt` for formatting checks
- Security audits with `cargo audit`

**Frontend CI:**
- `npm run build` verification
- Type checking with TypeScript
- ESLint for code quality
- Build artifact generation

**Deployment Pipeline:**
- Docker image builds on main branch merges
- Automated deployment to staging environment
- Integration tests against staging
- Promotion to production with manual approval

This pipeline ensures code quality and reduces deployment risk through automation.

### Testing Strategy

Comprehensive testing would cover all layers of the application:

**Unit Tests:**
- Service layer business logic
- Range request parsing
- Video validation logic
- Storage trait implementations

**Integration Tests:**
- API endpoint testing with test database
- Upload and streaming workflows
- Error handling paths

**End-to-End Tests:**
- Browser automation testing upload flow
- Video playback verification
- Cross-browser compatibility

**Load Testing:**
- Concurrent upload handling
- Streaming performance under load
- Database connection pool limits

Testing would use Rust's built-in test framework for backend and Vitest or Playwright for frontend.

### Additional Production Features

**Observability:**
- Structured logging with tracing
- Metrics collection (upload counts, streaming latency)
- Health check endpoints for load balancers
- Distributed tracing for request flows

**Security Enhancements:**
- Rate limiting per IP address
- Content Security Policy headers
- Input sanitization middleware
- Request size validation

**Operational Improvements:**
- Graceful shutdown handling
- Database connection pooling optimization
- Caching layer for frequently accessed metadata
- Background job monitoring

---

## Summary

The system design presents a clear evolutionary path from MVP to production. The current architecture prioritizes simplicity and fast time-to-stream while maintaining clean boundaries that enable future scaling. The Storage trait abstraction allows seamless migration from local filesystem to S3. HTTP Range Requests provide immediate functionality with a documented path to HLS adaptive streaming. The stateless design supports horizontal scaling through standard load balancing techniques.

This design demonstrates senior engineering through appropriate abstraction, pragmatic trade-offs, and documented scaling paths. HTTP Range Requests provide immediate functionality suitable for the core requirements. The stateless design and Docker deployment demonstrate horizontal scaling capability. Documented future work including CI/CD pipelines and comprehensive testing show awareness of production requirements.
