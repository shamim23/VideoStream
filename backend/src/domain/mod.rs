use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

impl Video {
    pub fn new(filename: String, content_type: String, size_bytes: i64) -> Self {
        let id = Uuid::new_v4().to_string();
        let storage_path = format!("videos/{}/{}/{}", 
            &id[0..2], 
            &id[2..4], 
            id
        );
        
        Self {
            id,
            filename,
            content_type,
            size_bytes,
            storage_path,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ShareResponse {
    pub share_url: String,
}
