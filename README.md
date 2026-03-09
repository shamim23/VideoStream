# Video Streaming Service

A minimal private video streaming service with clean architecture. Upload videos anonymously, get a shareable link, and stream with HTTP Range request support for smooth seeking.

## Features

- **Anonymous upload** - No account required
- **1GB file limit** - Configurable size cap
- **Shareable links** - UUID-based private URLs
- **Smooth streaming** - HTTP Range requests for seeking/scrubbing
- **Fast time-to-stream** - No transcoding delay, immediate playback

## Quick Start

### Option 1: Docker (Recommended)

**Prerequisites:** [Docker](https://docs.docker.com/get-docker/) & Docker Compose

**1. Clone and enter the project:**
```bash
cd video-app
```

**2. Start everything:**
```bash
docker-compose up --build
```

**3. Open the app:**
Navigate to `http://localhost:5173` in your browser.

**To scale horizontally** (run 3 backend instances):
```bash
docker-compose up --scale backend=3
```

### Option 2: Local Development

**Prerequisites:**
- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) 18+ and npm

> **Note:** If you just installed Rust and `cargo` is not found, either restart your terminal or run:
> ```bash
> source "$HOME/.cargo/env"
> ```

**1. Start the backend:**
```bash
cd backend
cargo run
```
The backend will start on `http://127.0.0.1:3000`.

**2. Start the frontend** (in a new terminal):
```bash
cd frontend
npm install
npm run dev
```
The frontend will start on `http://localhost:5173`.

**3. Open the app:**
Navigate to `http://localhost:5173` in your browser.

---

## Usage

1. **Upload a video** - Drag & drop or select a video file (MP4, WebM, MOV, MKV, OGV)
2. **Wait for upload** - Progress bar shows upload status with speed and ETA
3. **Get share link** - Copy the generated link or use the preview player
4. **Share & watch** - Anyone with the link can watch and seek through the video

---

## Project Structure

```
video-app/
├── backend/           # Rust (Axum) - API & streaming server
│   ├── src/
│   │   ├── main.rs        # Server entry point
│   │   ├── api/           # HTTP handlers
│   │   ├── service/       # Business logic
│   │   ├── storage/       # Storage trait & LocalStorage impl
│   │   └── domain/        # Video entity & models
│   └── Cargo.toml
│
├── frontend/          # SvelteKit - Web UI
│   ├── src/routes/
│   │   ├── +page.svelte       # Upload page
│   │   └── watch/[id]/        # Video player page
│   └── package.json
│
├── architecture.md         # High-level system design
├── SYSTEM_DESIGN.md        # Detailed design with scaling path
└── README.md               # This file
```

---

## Docker Commands

```bash
# Start all services
docker-compose up

# Start with 3 backend instances (horizontal scaling demo)
docker-compose up --scale backend=3

# Rebuild after code changes
docker-compose up --build

# Run in background
docker-compose up -d

# View logs
docker-compose logs -f

# Stop everything
docker-compose down

# Clean up volumes (removes uploaded videos)
docker-compose down -v
```

## Development

### Backend Commands

```bash
cd backend

# Run in development mode
cargo run

# Run tests
cargo test

# Build release binary
cargo build --release
```

### Frontend Commands

```bash
cd frontend

# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

---

## API Reference

### Upload Video
```bash
curl -X POST -F "video=@/path/to/video.mp4" http://127.0.0.1:3000/api/upload
```

**Response:**
```json
{
  "share_url": "/api/watch/abc123..."
}
```

### Stream Video
```bash
# Full video
curl http://127.0.0.1:3000/api/watch/<video_id>

# Specific byte range (for seeking)
curl -H "Range: bytes=0-1023" http://127.0.0.1:3000/api/watch/<video_id>
```

**Response:**
- `200 OK` - Full video
- `206 Partial Content` - Byte range response with `Content-Range` header

---

## Architecture

This project demonstrates clean architecture principles:

- **Layered design** - API → Service → Domain → Storage
- **Trait-based storage** - Swap LocalStorage → S3 without changing business logic
- **Stateless backend** - Ready for horizontal scaling
- **HTTP Range streaming** - Efficient seeking without transcoding

For detailed architecture documentation, see:
- [`SYSTEM_DESIGN.md`](./SYSTEM_DESIGN.md) - Complete system design with scaling path
- [`architecture.md`](./architecture.md) - Architecture overview

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | SvelteKit |
| Backend | Rust (Axum, Tokio) |
| Database | SQLite (zero config) |
| Storage | Local filesystem (S3-ready) |
| Deployment | Docker & Docker Compose |
| Load Balancer | Nginx |

---

## Troubleshooting

### `cargo: command not found`

If you just installed Rust, the cargo environment may not be loaded in your current terminal session. Either:

1. **Restart your terminal**, or
2. **Run this command** to load cargo for the current session:
   ```bash
   source "$HOME/.cargo/env"
   ```

This happens because Rustup adds cargo to your shell's configuration file (`.bashrc`, `.zshrc`, etc.), but the current terminal session hasn't reloaded it yet.

---

## License

MIT
