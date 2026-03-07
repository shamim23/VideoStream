# Video Streaming Service - Assessment Context

## 📋 Project Overview
- **Purpose:** Technical assessment for job application
- **Deadline:** 1 week
- **Tech Stack:** Rust (Axum) + SvelteKit
- **Assessment Focus:** Architecture design, clean code, maintainability

---

## 🎯 Core Requirements (Must Implement)

1. ✅ **Video Upload**
   - Max 1GB file size
   - Support common video formats (MP4, WebM, MOV, MKV, OGV)
   - Anonymous upload (no auth)
   - Generate shareable link per video

2. ✅ **Video Streaming**
   - Access via shareable link in browser
   - HTTP Range request support (for seeking/scrubbing)

3. ✅ **Architecture Document**
   - Explain core decisions
   - Overall design

---

## 🚀 Bonus Requirements (Prioritized)

### Bonus 1: Consistent Playback Performance
**Problem:** Large files (1GB) cause buffering on slow connections

**Solution Options:**

| Option | Description | Complexity | Impact |
|--------|-------------|------------|--------|
| A | HTTP Range Requests (already done) | Low | Allows seeking, but doesn't adapt quality |
| B | **HLS Transcoding** ⭐ | Medium | Adaptive bitrate, multiple quality levels |
| C | Document only | Low | Architecture without implementation |

**Recommended: Option B (HLS)**
```
Upload → Save Original → Spawn Transcoding Task → Return Link
                                   ↓
                        ffmpeg generates:
                        - 1080p (high quality)
                        - 720p (medium)
                        - 480p (low)
                        - 240p (mobile)
                        + playlist.m3u8 (manifest)
```

**Why HLS is impressive:**
- Industry standard (Netflix, YouTube use this)
- Shows video encoding knowledge
- Demonstrates async job processing
- Testable in browser with visible quality switching

---

### Bonus 2: Horizontal Scaling
**Question:** Do they want actual implementation or just architecture?

**Scaling Levels:**

| Level | What to Add | Complexity | Recommended? |
|-------|-------------|------------|--------------|
| 1 | Stateless app + PostgreSQL + S3 + Nginx LB | Medium | ✅ YES |
| 2 | Level 1 + Health checks + K8s/ECS config | High | Maybe |
| 3 | Microservices (split upload/stream/transcode) | Very High | ❌ No |

**Recommended: Level 1**
```
┌─────────────┐     ┌─────────────┐
│   Client    │────▶│   Nginx     │
└─────────────┘     │ Load Balancer│
                    └──────┬──────┘
                           │
           ┌───────────────┼───────────────┐
           ▼               ▼               ▼
      ┌─────────┐     ┌─────────┐     ┌─────────┐
      │Backend 1│     │Backend 2│     │Backend 3│
      │:3000    │     │:3000    │     │:3000    │
      └────┬────┘     └────┬────┘     └────┬────┘
           │               │               │
           └───────────────┼───────────────┘
                           ▼
                    ┌─────────────┐
                    │  PostgreSQL │
                    └─────────────┘
                           │
                    ┌─────────────┐
                    │     S3      │
                    └─────────────┘
```

**Code changes for Level 1:**
- SQLite → PostgreSQL (change `sqlx` features)
- LocalStorage → S3Storage (implement trait)
- Docker Compose with Nginx

---

### Bonus 3: Cost Efficiency
**Key points for architecture doc:**
- S3/Backblaze B2 for storage (cheaper than EBS)
- Serverless functions for transcoding (pay per use)
- SQLite → PostgreSQL only when scaling (start simple)
- No Kubernetes until needed (expensive)

---

## ❓ Questions to Ask Hiring Team

### Critical Questions

1. **"Is a live deployed demo required, or is local development sufficient?"**
   - Determines if you need Render/Vercel/AWS accounts

2. **"For the 'consistent playback' bonus - should I implement HLS transcoding with ffmpeg, or is HTTP range streaming sufficient?"**
   - HLS = ~2-4 hours extra work, but much more impressive

3. **"For horizontal scaling - should I actually implement Docker Compose with Nginx + PostgreSQL, or just document the architecture?"**
   - Implementation = proof it works

4. **"Should I include Infrastructure as Code (Terraform) even though guidelines say it's not required?"**
   - Shows extra initiative

### Secondary Questions

5. **"What format for the architecture document - Markdown, diagrams, or both?"**
6. **"Should the shareable links expire, or are they permanent?"**
7. **"How much time on UI vs backend architecture? (Current UI is 'functional' only)"**
8. **"Should I include tests (unit/integration)?"**

---

## ✅ Current Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| Axum backend server | ✅ Done | Port 3000 |
| File upload (1GB limit) | ✅ Done | With validation |
| UUID generation | ✅ Done | For shareable links |
| SQLite database | ✅ Done | Stores metadata |
| Local file storage | ✅ Done | With sharding (ab/cd/uuid) |
| HTTP Range streaming | ✅ Done | For seeking |
| MIME type validation | ✅ Done | video/* + extensions |
| CORS configuration | ✅ Done | For frontend |
| SvelteKit frontend | ✅ Done | Port 5173 |
| Real upload progress bar | ✅ Done | Speed + ETA |
| Video player page | ✅ Done | With share link |
| **HLS Transcoding** | ❌ Not done | Bonus - ask if needed |
| **PostgreSQL** | ❌ Not done | Bonus - scaling |
| **S3 Storage** | ❌ Not done | Bonus - scaling |
| **Docker Compose** | ❌ Not done | Bonus - deployment |
| **Tests** | ❌ Not done | Good to add |

---

## 🎯 Recommended Next Steps

### Day 1-2: Polish Core
1. Add error handling improvements
2. Add basic unit tests
3. Write architecture document (v1)

### Day 3-4: Add HLS (Bonus 1)
1. Add `ffmpeg-next` crate
2. Create transcoding service
3. Generate playlist.m3u8
4. Update player to use HLS

### Day 5-6: Add Scaling (Bonus 2)
1. Add PostgreSQL support
2. Add S3 storage trait
3. Create Docker Compose setup
4. Add Nginx load balancer

### Day 7: Final Polish
1. Write README with run instructions
2. Add architecture diagrams
3. Record demo video (if deployed)

---

## 📦 Project Structure

```
video-app/
├── backend/
│   ├── src/
│   │   ├── main.rs          # Server entry
│   │   ├── api/mod.rs       # HTTP handlers
│   │   ├── service/mod.rs   # Business logic & validation
│   │   ├── domain/mod.rs    # Video entity
│   │   └── storage/mod.rs   # File storage trait & impl
│   ├── Cargo.toml
│   └── storage/             # Videos & database
├── frontend/
│   ├── src/routes/
│   │   ├── +page.svelte     # Upload page
│   │   └── watch/[id]/+page.svelte  # Video player
│   └── package.json
└── architecture.md          # Existing doc
```

---

## 🔧 Quick Commands

### Run Backend
```bash
cd backend
cargo run
# http://localhost:3000
```

### Run Frontend
```bash
cd frontend
npm run dev
# http://localhost:5173
```

### Test Upload
```bash
curl -X POST -F "video=@test.mp4" http://localhost:3000/api/upload
```

---

## 💡 Key Architectural Decisions Already Made

1. **Layered Architecture:** API → Service → Domain → Storage
2. **Trait-based Storage:** Easy to swap LocalStorage → S3Storage
3. **Streaming Upload:** Uses `StreamReader` to handle large files without memory issues
4. **UUID Tokens:** Unlisted/private links without auth complexity
5. **Stateless Design:** Ready for horizontal scaling

---

## 📊 Scaling Path Document

### Current (Single Instance)
- SQLite database (file-based)
- Local filesystem storage
- Single backend process

### Phase 1: Stateless (Easy)
- Keep SQLite, but move to shared volume
- Or switch to PostgreSQL
- Run multiple backend instances behind Nginx

### Phase 2: Cloud Storage (Medium)
- Implement S3Storage trait
- Use Backblaze B2 (cheaper than AWS S3)
- Database stays PostgreSQL

### Phase 3: CDN (Future)
- CloudFront/Cloudflare in front
- Videos served from edge locations
- Database read replicas

---

## 🏆 Competitive Advantages to Add

1. **HLS Transcoding** - Most impressive technical feature
2. **Docker Compose** - Shows DevOps knowledge
3. **Comprehensive Tests** - Shows code quality focus
4. **Architecture Diagrams** - Visual documentation
5. **README with GIFs** - Polished presentation

---

## ⚠️ Common Pitfalls to Avoid

1. Don't over-engineer (microservices too early)
2. Don't forget error handling (failed uploads, missing files)
3. Don't hardcode config (use env vars)
4. Don't skip validation (file types, sizes)
5. Don't ignore security (even though auth not required)

---

*Last updated: March 6, 2026*
