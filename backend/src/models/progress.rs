//! Reading progress data models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Reading progress for a user on a specific content.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReadingProgress {
    pub id: i64,
    pub user_id: i64,
    pub content_id: i64,
    pub chapter_id: i64,
    pub position: i32,
    pub percentage: f32,
    pub updated_at: DateTime<Utc>,
}

/// Request to update reading progress.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProgressRequest {
    pub chapter_id: i64,
    pub position: i32,
}
