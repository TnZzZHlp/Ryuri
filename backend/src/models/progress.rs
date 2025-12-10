//! Reading progress data models.
//!
//! Progress is tracked per chapter, allowing users to have independent
//! progress for each chapter they read.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Reading progress for a user on a specific chapter.
///
/// Tracks where a user left off reading a particular chapter.
/// For comics, position represents the page number.
/// For novels, position represents the character offset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReadingProgress {
    /// Unique identifier for the progress record.
    pub id: i64,
    /// ID of the user this progress belongs to.
    pub user_id: i64,
    /// ID of the chapter being read.
    pub chapter_id: i64,
    /// Position within the chapter (page number or character offset).
    pub position: i32,
    /// Reading progress within the chapter as a percentage (0.0 to 100.0).
    pub percentage: f32,
    /// Timestamp when the progress was last updated.
    pub updated_at: DateTime<Utc>,
}

impl ReadingProgress {
    /// Creates a new ReadingProgress instance for insertion.
    pub fn create(user_id: i64, chapter_id: i64, position: i32) -> NewReadingProgress {
        NewReadingProgress {
            user_id,
            chapter_id,
            position,
            percentage: 0.0,
        }
    }

    /// Validates that the progress values are within acceptable ranges.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.position < 0 {
            return Err("Position cannot be negative");
        }
        if self.percentage < 0.0 || self.percentage > 100.0 {
            return Err("Percentage must be between 0 and 100");
        }
        Ok(())
    }
}

/// Data for creating new reading progress.
#[derive(Debug, Clone)]
pub struct NewReadingProgress {
    pub user_id: i64,
    pub chapter_id: i64,
    pub position: i32,
    pub percentage: f32,
}

/// Request to update reading progress for a chapter.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProgressRequest {
    /// Current position within the chapter.
    pub position: i32,
}

/// Response for chapter reading progress API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressResponse {
    pub chapter_id: i64,
    pub position: i32,
    pub percentage: f32,
    pub updated_at: DateTime<Utc>,
}

impl From<ReadingProgress> for ProgressResponse {
    fn from(progress: ReadingProgress) -> Self {
        Self {
            chapter_id: progress.chapter_id,
            position: progress.position,
            percentage: progress.percentage,
            updated_at: progress.updated_at,
        }
    }
}

/// Response for overall content progress (aggregated from chapter progress).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentProgressResponse {
    pub content_id: i64,
    pub total_chapters: i32,
    pub completed_chapters: i32,
    pub current_chapter_id: Option<i64>,
    pub overall_percentage: f32,
}
