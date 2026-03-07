# HLS Transcoding Implementation - Complexity Analysis

## 🎯 Overview
**Time Estimate:** 3-4 hours for basic implementation
**Difficulty:** Medium (mostly integration work)
**Value:** Very High (industry-standard, shows serious engineering)

---

## 📋 What HLS Actually Does

```
User uploads 1080p 1GB video
           ↓
    [Upload Complete]
           ↓
    Spawn ffmpeg job (async)
           ↓
    Creates multiple versions:
    ├─ 1080p/  (high bitrate)
    ├─ 720p/   (medium bitrate)  
    ├─ 480p/   (low bitrate)
    └─ 240p/   (mobile bitrate)
           ↓
    Creates playlist.m3u8
           ↓
    Player auto-selects quality
    based on bandwidth
```

---

## 🔧 Implementation Steps

### Step 1: Add Dependencies (~10 min)

**Cargo.toml:**
```toml
[dependencies]
# Existing...
ffmpeg-next = "7.0"  # FFmpeg bindings
tokio = { version = "1", features = ["full", "process"] }
```

**System dependency:**
```bash
# macOS
brew install ffmpeg

# Ubuntu/Debian
apt-get install ffmpeg libffmpeg-dev

# Or use Docker with ffmpeg pre-installed
```

**Complexity:** LOW - Just add crate, install ffmpeg

---

### Step 2: Create Transcoding Service (~45 min)

**New file: `backend/src/transcode/mod.rs`**

```rust
use std::path::Path;
use tokio::process::Command;

pub struct TranscodingService;

impl TranscodingService {
    pub async fn transcode_to_hls(
        input_path: &Path,
        output_dir: &Path,
    ) -> anyhow::Result<()> {
        // Create output directory
        tokio::fs::create_dir_all(output_dir).await?;
        
        // Run ffmpeg to create HLS segments
        let status = Command::new("ffmpeg")
            .arg("-i").arg(input_path)
            .arg("-vf").arg("scale=w=1920:h=1080:force_original_aspect_ratio=decrease")
            .arg("-c:v").arg("libx264")
            .arg("-b:v").arg("5000k")
            .arg("-maxrate").arg("5350k")
            .arg("-bufsize").arg("7500k")
            .arg("-c:a").arg("aac")
            .arg("-b:a").arg("192k")
            .arg("-hls_time").arg("4")
            .arg("-hls_playlist_type").arg("vod")
            .arg("-hls_segment_filename")
            .arg(output_dir.join("1080p_%03d.ts"))
            .arg(output_dir.join("1080p.m3u8"))
            .status()
            .await?;
            
        if !status.success() {
            return Err(anyhow::anyhow!("FFmpeg failed"));
        }
        
        // Create master playlist
        Self::create_master_playlist(output_dir).await?;
        
        Ok(())
    }
    
    async fn create_master_playlist(output_dir: &Path) -> anyhow::Result<()> {
        let master = r#"#EXTM3U
#EXT-X-VERSION:3
#EXT-X-STREAM-INF:BANDWIDTH=5000000,RESOLUTION=1920x1080
1080p.m3u8
"#;
        
        tokio::fs::write(output_dir.join("playlist.m3u8"), master).await?;
        Ok(())
    }
}
```

**Complexity:** MEDIUM - Understanding ffmpeg CLI args

---

### Step 3: Add Async Job Processing (~30 min)

**Update `service/mod.rs`:**

```rust
use tokio::task;

pub async fn upload_video(...) -> Result<String, ServiceError> {
    // ... existing save code ...
    
    // Spawn transcoding in background
    let video_id = video.id.clone();
    let storage_path = video.storage_path.clone();
    task::spawn(async move {
        if let Err(e) = Self::transcode_video(&video_id, &storage_path).await {
            eprintln!("Transcoding failed for {}: {}", video_id, e);
        }
    });
    
    Ok(video.id)
}

async fn transcode_video(id: &str, path: &str) -> anyhow::Result<()> {
    let input = storage_path.join(path).join("original.mp4");
    let output = storage_path.join(path).join("hls");
    
    TranscodingService::transcode_to_hls(&input, &output).await?;
    
    // Update DB to mark transcoding complete
    sqlx::query("UPDATE videos SET hls_ready = true WHERE id = ?")
        .bind(id)
        .execute(&self.db)
        .await?;
        
    Ok(())
}
```

**Complexity:** LOW - Just spawn a background task

---

### Step 4: Add HLS Streaming Endpoint (~20 min)

**Update `api/mod.rs`:**

```rust
pub async fn stream_hls_handler(
    State(state): State<Arc<AppState>>,
    Path((id, file)): Path<(String, String)>,
) -> Result<Response, (StatusCode, String)> {
    // Serve .m3u8 playlist or .ts segments
    let path = state.video_service.get_hls_path(&id, &file).await
        .map_err(map_service_error)?;
        
    let content_type = if file.ends_with(".m3u8") {
        "application/vnd.apple.mpegurl"
    } else {
        "video/mp2t"
    };
    
    // Read and return file...
}
```

**Complexity:** LOW - Similar to existing stream handler

---

### Step 5: Update Frontend Player (~30 min)

**Install hls.js (for browsers without native HLS):**
```bash
cd frontend
npm install hls.js
```

**Update `+page.svelte`:**

```svelte
<script>
  import Hls from 'hls.js';
  
  let videoElement;
  let hls;
  
  onMount(() => {
    if (Hls.isSupported()) {
      hls = new Hls({
        startLevel: -1, // Auto quality
        capLevelToPlayerSize: true,
      });
      hls.loadSource(getHlsUrl());
      hls.attachMedia(videoElement);
    } else if (videoElement.canPlayType('application/vnd.apple.mpegurl')) {
      // Native HLS support (Safari)
      videoElement.src = getHlsUrl();
    }
  });
  
  function getHlsUrl() {
    return `${apiBaseUrl}/api/watch/${videoId}/playlist.m3u8`;
  }
</script>

<video bind:this={videoElement} controls />
```

**Complexity:** LOW - hls.js does the heavy lifting

---

### Step 6: Database Migration (~15 min)

```sql
-- Add column to track transcoding status
ALTER TABLE videos ADD COLUMN hls_ready BOOLEAN DEFAULT FALSE;
ALTER TABLE videos ADD COLUMN transcoding_started_at DATETIME;
ALTER TABLE videos ADD COLUMN transcoding_completed_at DATETIME;
```

**Complexity:** LOW - Single ALTER TABLE

---

## ⏱️ Total Time Breakdown

| Task | Time | Complexity |
|------|------|------------|
| Add dependencies | 10 min | ⭐ Low |
| Transcoding service | 45 min | ⭐⭐⭐ Medium |
| Async job processing | 30 min | ⭐⭐ Low |
| HLS streaming endpoint | 20 min | ⭐⭐ Low |
| Frontend player update | 30 min | ⭐⭐ Low |
| Database migration | 15 min | ⭐ Low |
| Testing & debugging | 60 min | ⭐⭐⭐ Medium |
| **TOTAL** | **~3.5 hours** | **Medium** |

---

## 🎯 Skills Demonstrated

Adding HLS shows you understand:

1. **Video encoding** (codecs, bitrates, containers)
2. **Async job processing** (background tasks)
3. **Adaptive streaming protocols** (HLS/DASH)
4. **External tool integration** (ffmpeg)
5. **Progressive enhancement** (fallback to native HLS)

---

## 🚨 Potential Issues & Solutions

| Issue | Solution |
|-------|----------|
| FFmpeg not installed | Add to Dockerfile, or use `ffmpeg-sidecar` crate |
| Transcoding takes too long | Add progress tracking, show "processing" state |
| Large files crash ffmpeg | Limit concurrent transcodes, add memory limits |
| Safari vs Chrome differences | Use hls.js for all browsers (consistent behavior) |
| Storage doubles (original + HLS) | Delete original after HLS ready, or keep as backup |

---

## 📝 Simpler Alternative (If Short on Time)

If 3.5 hours is too much, implement **partial HLS:**

```rust
// Just create one quality level (720p) instead of 4
// Takes ~1 hour instead of 3.5

Command::new("ffmpeg")
    .arg("-i").arg(input_path)
    .arg("-vf").arg("scale=1280:720")  // Just 720p
    .arg("-c:v").arg("libx264")
    .arg("-preset").arg("fast")         // Faster encoding
    .arg("-crf").arg("23")              // Quality vs speed
    .arg("-hls_time").arg("10")         // Larger segments = faster
    .arg(output_dir.join("playlist.m3u8"))
```

**Result:** Still shows HLS knowledge, but much faster to implement.

---

## ✅ Recommendation

**DO IT if:**
- You have 3-4 hours to spare
- Want to stand out significantly
- Job is for video/media company

**SKIP if:**
- Time is tight
- Job is general backend (not video-focused)
- Other requirements are incomplete

**Compromise:**
- Document HLS architecture without full implementation
- Shows you know what it is and how to add it later

---

*Created for assessment planning*
