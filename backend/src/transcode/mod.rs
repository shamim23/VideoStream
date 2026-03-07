use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Quality presets for HLS transcoding
#[derive(Debug, Clone, Copy)]
pub struct QualityLevel {
    pub name: &'static str,
    pub width: i32,
    pub height: i32,
    pub video_bitrate: &'static str,
    pub audio_bitrate: &'static str,
}

/// 1080p quality
pub const QUALITY_1080P: QualityLevel = QualityLevel {
    name: "1080p",
    width: 1920,
    height: 1080,
    video_bitrate: "5000k",
    audio_bitrate: "192k",
};

/// 720p quality
pub const QUALITY_720P: QualityLevel = QualityLevel {
    name: "720p",
    width: 1280,
    height: 720,
    video_bitrate: "2500k",
    audio_bitrate: "128k",
};

/// 480p quality
pub const QUALITY_480P: QualityLevel = QualityLevel {
    name: "480p",
    width: 854,
    height: 480,
    video_bitrate: "1000k",
    audio_bitrate: "96k",
};

/// 240p quality (mobile)
pub const QUALITY_240P: QualityLevel = QualityLevel {
    name: "240p",
    width: 426,
    height: 240,
    video_bitrate: "400k",
    audio_bitrate: "64k",
};

pub const ALL_QUALITIES: &[QualityLevel] = &[QUALITY_1080P, QUALITY_720P, QUALITY_480P, QUALITY_240P];

pub struct TranscodingService;

impl TranscodingService {
    /// Transcode video to HLS format with multiple quality levels
    pub async fn transcode_to_hls(
        input_path: &Path,
        output_dir: &Path,
    ) -> anyhow::Result<()> {
        // Create HLS output directory
        let hls_dir = output_dir.join("hls");
        tokio::fs::create_dir_all(&hls_dir).await?;
        
        println!("Starting HLS transcoding for: {:?}", input_path);
        println!("Output directory: {:?}", hls_dir);
        
        // Transcode each quality level
        for quality in ALL_QUALITIES {
            Self::transcode_quality(input_path, &hls_dir, quality).await?;
        }
        
        // Create master playlist
        Self::create_master_playlist(&hls_dir).await?;
        
        println!("HLS transcoding complete!");
        Ok(())
    }
    
    /// Transcode a single quality level
    async fn transcode_quality(
        input_path: &Path,
        hls_dir: &Path,
        quality: &QualityLevel,
    ) -> anyhow::Result<()> {
        let scale_filter = format!(
            "scale=w={}:h={}:force_original_aspect_ratio=decrease",
            quality.width, quality.height
        );
        
        let output_m3u8 = hls_dir.join(format!("{}.m3u8", quality.name));
        let segment_pattern = hls_dir.join(format!("{}_%03d.ts", quality.name));
        
        println!(
            "Transcoding {}: {}x{} @ {}",
            quality.name, quality.width, quality.height, quality.video_bitrate
        );
        
        let status = Command::new("ffmpeg")
            .arg("-i").arg(input_path)
            .arg("-vf").arg(&scale_filter)
            .arg("-c:v").arg("libx264")
            .arg("-b:v").arg(quality.video_bitrate)
            .arg("-maxrate").arg(format!("{}", 
                Self::calculate_maxrate(quality.video_bitrate)))
            .arg("-bufsize").arg(format!("{}", 
                Self::calculate_bufsize(quality.video_bitrate)))
            .arg("-c:a").arg("aac")
            .arg("-b:a").arg(quality.audio_bitrate)
            .arg("-ar").arg("48000")  // Audio sample rate
            .arg("-hls_time").arg("4")  // 4 second segments
            .arg("-hls_playlist_type").arg("vod")  // Video on demand
            .arg("-hls_segment_filename").arg(&segment_pattern)
            .arg("-f").arg("hls")
            .arg(&output_m3u8)
            .status()
            .await?;
            
        if !status.success() {
            return Err(anyhow::anyhow!(
                "FFmpeg failed for quality {}", quality.name
            ));
        }
        
        println!("Completed {}", quality.name);
        Ok(())
    }
    
    /// Create master playlist that references all quality levels
    async fn create_master_playlist(hls_dir: &Path) -> anyhow::Result<()> {
        let mut master_content = String::from("#EXTM3U\n#EXT-X-VERSION:3\n\n");
        
        // Add each quality level to master playlist
        // Ordered from highest to lowest bandwidth
        for quality in ALL_QUALITIES {
            let bandwidth = Self::bitrate_to_bandwidth(quality.video_bitrate);
            let resolution = format!("{}x{}", quality.width, quality.height);
            
            master_content.push_str(&format!(
                "#EXT-X-STREAM-INF:BANDWIDTH={},RESOLUTION={}\n",
                bandwidth, resolution
            ));
            master_content.push_str(&format!("{}.m3u8\n\n", quality.name));
        }
        
        let master_path = hls_dir.join("playlist.m3u8");
        tokio::fs::write(&master_path, master_content).await?;
        
        println!("Created master playlist: {:?}", master_path);
        Ok(())
    }
    
    /// Calculate max bitrate (107% of target)
    fn calculate_maxrate(bitrate: &str) -> String {
        let num: i32 = bitrate.trim_end_matches('k').parse().unwrap_or(5000);
        format!("{}k", (num as f32 * 1.07) as i32)
    }
    
    /// Calculate buffer size (150% of target)
    fn calculate_bufsize(bitrate: &str) -> String {
        let num: i32 = bitrate.trim_end_matches('k').parse().unwrap_or(5000);
        format!("{}k", (num as f32 * 1.5) as i32)
    }
    
    /// Convert bitrate string to bandwidth number (bits per second)
    fn bitrate_to_bandwidth(bitrate: &str) -> u32 {
        bitrate.trim_end_matches('k')
            .parse::<u32>()
            .unwrap_or(5000)
            * 1000
    }
    
    /// Get the path to the HLS playlist for a video
    pub fn get_hls_path(storage_path: &Path, video_id: &str) -> PathBuf {
        // Video is stored in videos/ab/cd/uuid/ format
        let video_dir = storage_path
            .join("videos")
            .join(&video_id[0..2])
            .join(&video_id[2..4])
            .join(video_id)
            .join("hls")
            .join("playlist.m3u8");
        video_dir
    }
    
    /// Check if HLS files exist for a video
    pub async fn hls_exists(storage_path: &Path, video_id: &str) -> bool {
        let hls_path = Self::get_hls_path(storage_path, video_id);
        tokio::fs::try_exists(&hls_path).await.unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bitrate_calculations() {
        assert_eq!(TranscodingService::calculate_maxrate("5000k"), "5350k");
        assert_eq!(TranscodingService::calculate_bufsize("5000k"), "7500k");
        assert_eq!(TranscodingService::bitrate_to_bandwidth("5000k"), 5000000);
    }
}
