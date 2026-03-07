Blocking Gaps vs architecture.md

  1. Not streaming-first (upload path buffers whole file in memory)

  - Spec says streaming-first and store_stream from stream (architecture.md:12,
    architecture.md:48, architecture.md:61).
  - Current code uses field.bytes().await then Vec<u8> and writes all at once:
    api/mod.rs:35, storage/mod.rs:9, storage/mod.rs:29.

  2. No explicit format validation

  - Spec says backend validates size and format (architecture.md:59).
  - Size limit exists via request layers: main.rs:75.
  - Format/content validation is not implemented in upload handler: api/
    mod.rs:17.

  3. Layering does not match documented clean architecture

  - Spec includes separate service layer and inward dependency direction
    (architecture.md:9, architecture.md:22).
  - Current business orchestration is in API handlers directly, with no service

  4. sqlx compile-time checked queries not used

  - Spec calls this out explicitly (architecture.md:13).
  - Spec says /api/stream/{token} (architecture.md:66).
  - Current route is /api/watch/:id: main.rs:74.