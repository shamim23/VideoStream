  📋 Clarifying Questions for the Hiring Team

  1. Deployment & Infrastructure

   Question                                 Why It Matters
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   "Is a live deployed demo required, or    Determines if you need to set up Rende
   is local development sufficient?"        r/Vercel/AWS accounts
   "Do you want to see Infrastructure as    Shows extra initiative if you include
   Code (Terraform/CloudFormation) even t   it
   hough guidelines say it's not required
   ?"
   "Should the backend and frontend be de   Affects architecture decisions
   ployed as separate services, or as a m
   onolithic deployment?"

  2. Storage Strategy

   Question                                 Why It Matters
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   "Is local filesystem storage acceptabl   Cloud storage adds complexity; current
   e for this assessment, or do you expec   impl uses local FS
   t a cloud storage implementation (S3,
   Backblaze B2)?"
   "If cloud storage is expected, should    Determines scope of work
   I implement the abstraction trait only
   , or include actual cloud provider int
   egration?"
   "Should videos be transcoded to multip   Major architectural decision for the "
   le resolutions (HLS/DASH) for the bonu   consistent playback" bonus
   s 'consistent playback' requirement?"

  3. Architecture & Design Document

   Question                                 Why It Matters
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   "What format do you prefer for the arc   Ensures deliverable meets expectations
   hitecture document - Markdown, PDF, or
   diagrams (Draw.io/Excalidraw)?"
   "Should the document focus on the curr   The bonus mentions horizontal scaling
   ent implementation, or also include th
   e 'scaling path' for horizontal scalin
   g?"
   "Do you want to see database schema di   Shows thoroughness
   agrams or API endpoint documentation?"

  4. Functional Requirements

   Question                                 Why It Matters
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   "For the 'consistent playback' bonus,    HLS requires ffmpeg - significant addi
   is HTTP range request streaming suffic   tional work
   ient, or do you expect adaptive bitrat
   e streaming (HLS)?"
   "Should the shareable links expire aft   Affects database schema
   er a certain time, or are they permane
   nt?"
   "Is there a requirement for video dele   Not mentioned but good to clarify
   tion/cleanup?"

  5. Assessment Scope

   Question                                 Why It Matters
  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   "How much time should I spend on the U   "Functional UI is sufficient" suggests
   I vs the backend architecture?"          backend focus
   "Should I include tests (unit/integrat   Shows code quality awareness
   ion)?"
   "Is the 1-week deadline flexible if I    Clarifies if polish is expected vs cor
   implement additional features?"          e features

  ─────────────────────────────────────────────────────────────────────────────────
  🎯 My Recommendations

  Based on the requirements, here's what I'd prioritize:

  Must Have (Core Requirements)

  1. ✅ Upload with 1GB limit
  2. ✅ Streaming with shareable links
  3. ✅ Architecture document
  4. ✅ Clean, maintainable Rust + Svelte code

  Should Have (Shows Extra Effort)

  1. Add HLS transcoding - Use ffmpeg-next crate to generate .m3u8 playlist after u
     ad
  2. Add tests - Unit tests for service layer, integration tests for API
  3. Docker setup - Dockerfile + docker-compose.yml for easy local setup

  Nice to Have (If Time Permits)

  1. Deploy to Render (backend) + Vercel (frontend)
  2. Implement S3 storage trait (even if not connected)
  3. Add rate limiting

  ─────────────────────────────────────────────────────────────────────────────────
  Want me to:

  1. Add HLS transcoding for the "consistent playback" bonus?
  2. Create the architecture document based on what we've built?
  3. Add Docker setup for easy deployment?

💫 by asking 2. The system should be able to scale horizontally.
does it mean to add a loadbalancer via aws? or something else?
• Good question from the user. They're asking what "scale horizontally" means in th
  e context of this assessment. Let me explain the different levels of horizontal s
  caling and what they could implement to demonstrate this understanding.

  Horizontal scaling can mean different things:

  1. Application level - stateless app behind a load balancer
  2. Database level - read replicas, sharding
  3. Storage level - distributed storage like S3
  4. Full microservices architecture

  For this assessment, I should explain the spectrum of options from simple to comp
  lex, and what would be most appropriate to demonstrate.
• Great question! "Scale horizontally" can mean different levels. Let me break it d
  own from simplest to most complex:

  ─────────────────────────────────────────────────────────────────────────────────
  🔄 Levels of Horizontal Scaling

  Level 1: Stateless Application (Easiest - Recommended for assessment)

  ┌─────────────┐     ┌─────────────┐
  │   Client    │────▶│  Load Balancer│
  └─────────────┘     │   (Nginx/    │
                      │    AWS ALB)   │
                      └──────┬──────┘
                             │
             ┌───────────────┼───────────────┐
             ▼               ▼               ▼
        ┌─────────┐     ┌─────────┐     ┌─────────┐
        │Backend 1│     │Backend 2│     │Backend N│
        │:3000    │     │:3000    │     │:3000    │
        └────┬────┘     └────┬────┘     └────┬────┘
             │               │               │
             └───────────────┼───────────────┘
                             ▼
                      ┌─────────────┐
                      │  PostgreSQL │  (shared DB)
                      │   (RDS)     │
                      └─────────────┘
                             │
                      ┌─────────────┐
                      │     S3      │  (shared storage)
                      │  (Backblaze)│
                      └─────────────┘

  What to implement:

  • Make backend stateless (✅ already done - no session state)
  • Replace SQLite with PostgreSQL (via sqlx, minimal code change)
  • Replace local storage with S3 trait implementation
  • Add Docker + Docker Compose with Nginx load balancer

  Code changes needed: ~50 lines

  ─────────────────────────────────────────────────────────────────────────────────
  Level 2: Auto-scaling with Health Checks

  Add to Level 1:

  • Health check endpoint (/health)
  • Graceful shutdown handling
  • Kubernetes manifests or AWS ECS config

  For assessment: Probably overkill, but shows DevOps knowledge

  ─────────────────────────────────────────────────────────────────────────────────
  Level 3: Microservices (Overkill for this assessment)

  Split into:

  • Upload service
  • Streaming service
  • Transcoding service (ffmpeg workers)
  • Metadata service

  Verdict: Too complex for 1-week assessment

  ─────────────────────────────────────────────────────────────────────────────────
  🎯 My Recommendation for Assessment

  Implement Level 1 with these specific changes:

  1. Database: SQLite → PostgreSQL
• Used ReadFile (video-app/backend/Cargo.toml)
• I can show them the simple changes needed for PostgreSQL support. The beauty of s
  qlx is that switching from SQLite to PostgreSQL is just a configuration change -
  the queries stay the same.
• 2. Storage: Local FS → S3/Backblaze B2### 3. Add Nginx Load Balancer```nginx

  upstream backend { server backend1:3000; server backend2:3000; server backend3:30
  00; }

  server { listen 80; location / {        proxy_pass http://backend;    } }


  In your architecture document, explain:

  | Aspect | Current | Horizontal Scaling Path |
  |--------|---------|------------------------|
  | **App State** | Stateless ✅ | Can run multiple instances behind LB |
  | **Database** | SQLite (single file) | PostgreSQL with connection pooling |
  | **Storage** | Local filesystem | S3/Backblaze B2 (shared across instances) |
  | **Sessions** | None (UUID tokens) | Stateless - no change needed |
  | **Caching** | None | Could add Redis for metadata caching |

  ## 🤔 What to Ask the Hiring Team

  > **"For the horizontal scaling requirement, would you like me to:**
  >
  > **Option A)** Document the scaling architecture with diagrams showing how to ad
  d a load balancer and switch to PostgreSQL/S3?
  >
  > **Option B)** Actually implement Docker Compose with Nginx load balancer and Po
  stgreSQL so you can run `docker-compose up --scale backend=3`?
  >
  > **Option C)** Just ensure the code is stateless and document the rest?
  >
  > **My recommendation is Option B** - it takes ~30 minutes to add and shows tangi
  ble proof of horizontal scaling capability."

  ---

  Want me to **implement Option B** right now? I can:

  1. Add PostgreSQL support (just change `features = ["sqlite"]` to `features = ["p
  ostgres"]`)
  2. Create S3 storage trait implementation
  3. Add `docker-compose.yml` with Nginx load balancer
  4. Write a script to demonstrate 3 backend instances handling uploads

  This would be a **strong competitive advantage** in your assessment!
