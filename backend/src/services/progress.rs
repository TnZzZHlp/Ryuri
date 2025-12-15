//! Reading progress service.
//!
//! This module provides business logic for tracking user reading progress
//! on chapters and calculating overall content progress.

use sqlx::{Pool, Sqlite};

use crate::error::{AppError, Result};
use crate::models::{
    ContentProgressResponse, NewReadingProgress, ProgressResponse, ReadingProgress,
};
use crate::repository::content::ChapterRepository;
use crate::repository::progress::ProgressRepository;

/// Service for reading progress operations.
///
/// Handles chapter-based progress tracking and overall content progress calculation.
pub struct ProgressService {
    pool: Pool<Sqlite>,
}

impl ProgressService {
    /// Create a new progress service.
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Get reading progress for a specific chapter.
    ///
    /// Requirements: 5.1, 5.2
    pub async fn get_chapter_progress(
        &self,
        user_id: i64,
        chapter_id: i64,
    ) -> Result<Option<ReadingProgress>> {
        // Verify chapter exists
        ChapterRepository::find_by_id(&self.pool, chapter_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Chapter with id {} not found", chapter_id))
            })?;

        ProgressRepository::find_by_user_and_chapter(&self.pool, user_id, chapter_id).await
    }

    /// Get all chapter progress for a content.
    ///
    /// Returns progress for all chapters of the content that the user has read.
    /// Requirements: 5.1
    pub async fn get_content_progress(
        &self,
        user_id: i64,
        content_id: i64,
    ) -> Result<Vec<ReadingProgress>> {
        ProgressRepository::find_by_user_and_content(&self.pool, user_id, content_id).await
    }

    /// Update reading progress for a chapter.
    ///
    /// Validates the position and calculates the percentage based on chapter content.
    /// Requirements: 5.2, 5.3
    pub async fn update_progress(
        &self,
        user_id: i64,
        chapter_id: i64,
        position: i32,
    ) -> Result<ReadingProgress> {
        // Validate position
        if position < 0 {
            return Err(AppError::BadRequest(
                "Position cannot be negative".to_string(),
            ));
        }

        // Verify chapter exists
        let chapter = ChapterRepository::find_by_id(&self.pool, chapter_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Chapter with id {} not found", chapter_id))
            })?;

        // Calculate percentage based on position
        // For now, we'll use a simple calculation - the caller should provide
        // the total pages/characters to calculate accurate percentage
        // We'll store the position and let the frontend calculate display percentage
        // or we can enhance this later with total page count from the chapter
        let percentage = self.calculate_percentage(&chapter, position).await?;

        let new_progress = NewReadingProgress {
            user_id,
            chapter_id,
            position,
            percentage,
        };

        ProgressRepository::upsert(&self.pool, new_progress).await
    }

    /// Calculate percentage based on position within a chapter.
    ///
    /// For comics, this would be page_number / total_pages * 100
    /// For novels, this would be character_position / total_characters * 100
    async fn calculate_percentage(
        &self,
        _chapter: &crate::models::Chapter,
        _position: i32,
    ) -> Result<f32> {
        // For now, we return 0.0 and let the caller provide the percentage
        // In a full implementation, we would:
        // 1. For comics: count images in the archive and calculate page/total
        // 2. For novels: get text length and calculate position/total
        // This requires accessing the archive which is expensive, so we'll
        // let the frontend track and send the percentage with updates
        Ok(0.0)
    }

    /// Update reading progress with explicit percentage.
    ///
    /// Used when the caller knows the total and can calculate the percentage.
    /// Requirements: 5.2, 5.3
    pub async fn update_progress_with_percentage(
        &self,
        user_id: i64,
        chapter_id: i64,
        position: i32,
        percentage: f32,
    ) -> Result<ReadingProgress> {
        // Validate inputs
        if position < 0 {
            return Err(AppError::BadRequest(
                "Position cannot be negative".to_string(),
            ));
        }
        if !(0.0..=100.0).contains(&percentage) {
            return Err(AppError::BadRequest(
                "Percentage must be between 0 and 100".to_string(),
            ));
        }

        // Verify chapter exists
        ChapterRepository::find_by_id(&self.pool, chapter_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Chapter with id {} not found", chapter_id))
            })?;

        let new_progress = NewReadingProgress {
            user_id,
            chapter_id,
            position,
            percentage,
        };

        ProgressRepository::upsert(&self.pool, new_progress).await
    }

    /// Get aggregated content progress.
    ///
    /// Calculates overall progress across all chapters of a content.
    /// Requirements: 5.4
    pub async fn get_aggregated_content_progress(
        &self,
        user_id: i64,
        content_id: i64,
    ) -> Result<ContentProgressResponse> {
        // Get total chapter count
        let total_chapters =
            ChapterRepository::count_by_content(&self.pool, content_id).await? as i32;

        // Get completed chapters count
        let completed_chapters =
            ProgressRepository::count_completed_chapters(&self.pool, user_id, content_id).await?
                as i32;

        // Get the most recently read chapter
        let latest_progress =
            ProgressRepository::find_latest_by_user_and_content(&self.pool, user_id, content_id)
                .await?;

        let current_chapter_id = latest_progress.as_ref().map(|p| p.chapter_id);

        // Calculate overall percentage
        let overall_percentage = if total_chapters > 0 {
            // Weight: completed chapters + current chapter progress
            let base_percentage = (completed_chapters as f32 / total_chapters as f32) * 100.0;

            // Add partial progress from current chapter if not completed
            if let Some(ref progress) = latest_progress {
                if progress.percentage < 100.0 {
                    // Add fractional chapter progress
                    let chapter_contribution = progress.percentage / total_chapters as f32;
                    (base_percentage + chapter_contribution).min(100.0)
                } else {
                    base_percentage
                }
            } else {
                base_percentage
            }
        } else {
            0.0
        };

        Ok(ContentProgressResponse {
            content_id,
            total_chapters,
            completed_chapters,
            current_chapter_id,
            overall_percentage,
        })
    }

    /// Get chapter progress as a response DTO.
    pub async fn get_chapter_progress_response(
        &self,
        user_id: i64,
        chapter_id: i64,
    ) -> Result<Option<ProgressResponse>> {
        let progress = self.get_chapter_progress(user_id, chapter_id).await?;
        Ok(progress.map(ProgressResponse::from))
    }

    /// Get the most recently read contents for a user.
    ///
    /// Returns the contents that have the most recently updated reading progress.
    pub async fn get_recent_contents(
        &self,
        user_id: i64,
        limit: i64,
    ) -> Result<Vec<crate::models::ContentResponse>> {
        let contents = ProgressRepository::find_recent_contents_by_user(&self.pool, user_id, limit).await?;
        Ok(contents
            .into_iter()
            .map(crate::models::ContentResponse::from)
            .collect())
    }
}

/// Utility functions for progress percentage calculation.
impl ProgressService {
    /// Calculate percentage from position and total.
    ///
    /// Returns a percentage value between 0.0 and 100.0.
    /// Requirements: 5.4
    pub fn calculate_percentage_from_total(position: i32, total: i32) -> f32 {
        if total <= 0 {
            return 0.0;
        }
        let percentage = (position as f32 / total as f32) * 100.0;
        percentage.clamp(0.0, 100.0)
    }

    /// Calculate overall content percentage from chapter progress.
    ///
    /// Takes into account completed chapters and partial progress on current chapter.
    /// Requirements: 5.4
    pub fn calculate_overall_percentage(
        completed_chapters: i32,
        total_chapters: i32,
        current_chapter_percentage: f32,
    ) -> f32 {
        if total_chapters <= 0 {
            return 0.0;
        }

        // Base percentage from completed chapters
        let base = (completed_chapters as f32 / total_chapters as f32) * 100.0;

        // Add partial progress from current chapter
        let partial = current_chapter_percentage / total_chapters as f32;

        (base + partial).clamp(0.0, 100.0)
    }
}
