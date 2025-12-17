//! Bangumi metadata service.
//!
//! This module provides functionality to search and fetch metadata from
//! the Bangumi.tv API for content items.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// Base URL for the Bangumi API.
const BANGUMI_API_BASE: &str = "https://api.bgm.tv";

/// User agent for API requests.
const USER_AGENT: &str = "ryuri/0.1.0 (https://github.com/tnzzzhlp/ryuri)";

/// Search result from Bangumi API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "dev", derive(utoipa::ToSchema))]
pub struct BangumiSearchResult {
    /// Bangumi subject ID.
    pub id: i64,
    /// Original name (usually Japanese/Chinese).
    pub name: String,
    /// Chinese name (if available).
    #[serde(default)]
    pub name_cn: Option<String>,
    /// Summary/description.
    #[serde(default)]
    pub summary: Option<String>,
    /// Cover image URL.
    #[serde(default)]
    pub image: Option<String>,
}

/// Response from Bangumi search API.
#[derive(Debug, Clone, Deserialize)]
struct BangumiSearchResponse {
    /// List of search results.
    #[serde(default)]
    list: Vec<BangumiSearchItem>,
}

/// Individual search result item from Bangumi API.
#[derive(Debug, Clone, Deserialize)]
struct BangumiSearchItem {
    id: i64,
    name: String,
    #[serde(default)]
    name_cn: Option<String>,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    images: Option<BangumiImages>,
}

/// Image URLs from Bangumi API.
#[derive(Debug, Clone, Deserialize)]
struct BangumiImages {
    #[serde(default)]
    large: Option<String>,
    #[serde(default)]
    medium: Option<String>,
    #[serde(default)]
    small: Option<String>,
}

impl From<BangumiSearchItem> for BangumiSearchResult {
    fn from(item: BangumiSearchItem) -> Self {
        let image = item
            .images
            .and_then(|imgs| imgs.large.or(imgs.medium).or(imgs.small));

        Self {
            id: item.id,
            name: item.name,
            name_cn: item.name_cn,
            summary: item.summary,
            image,
        }
    }
}

/// Service for fetching metadata from Bangumi.tv API.
pub struct BangumiService {
    client: Client,
    api_key: Option<String>,
}

impl BangumiService {
    /// Create a new BangumiService instance.
    pub fn new(api_key: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, api_key }
    }

    /// Search for subjects on Bangumi by keyword.
    ///
    /// # Arguments
    /// * `query` - Search keyword (usually the content title)
    ///
    /// # Returns
    /// A list of matching search results.
    ///
    /// Requirements: 8.4
    pub async fn search(&self, query: &str) -> Result<Vec<BangumiSearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Use the v0 search API with type filter for books (type=1) and anime (type=2)
        // We search both to cover manga and light novels
        let url = format!(
            "{}/search/subject/{}",
            BANGUMI_API_BASE,
            urlencoding::encode(query)
        );

        let mut request = self.client.get(&url).query(&[
            ("type", "1"), // Books (manga, novels)
            ("responseGroup", "small"),
            ("max_results", "10"),
        ]);

        // Add API key if available
        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to search Bangumi: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                // No results found
                return Ok(Vec::new());
            }
            return Err(AppError::Internal(format!(
                "Bangumi API returned error: {}",
                status
            )));
        }

        let search_response: BangumiSearchResponse = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse Bangumi response: {}", e)))?;

        Ok(search_response
            .list
            .into_iter()
            .map(BangumiSearchResult::from)
            .collect())
    }

    /// Get detailed subject information from Bangumi.
    ///
    /// # Arguments
    /// * `bangumi_id` - The Bangumi subject ID
    ///
    /// # Returns
    /// The raw JSON response from Bangumi API, stored as-is for flexibility.
    ///
    /// Requirements: 8.4
    pub async fn get_subject(&self, bangumi_id: i64) -> Result<serde_json::Value> {
        let url = format!("{}/v0/subjects/{}", BANGUMI_API_BASE, bangumi_id);

        let mut request = self.client.get(&url);

        // Add API key if available
        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch Bangumi subject: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(AppError::NotFound(format!(
                    "Bangumi subject {} not found",
                    bangumi_id
                )));
            }
            return Err(AppError::Internal(format!(
                "Bangumi API returned error: {}",
                status
            )));
        }

        let subject: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to parse Bangumi subject: {}", e)))?;

        Ok(subject)
    }

    /// Automatically scrape metadata for content based on its title.
    ///
    /// This method searches Bangumi using the content title and fetches
    /// detailed metadata from the first matching result.
    ///
    /// # Arguments
    /// * `title` - The content title to search for
    ///
    /// # Returns
    /// The metadata JSON blob if found, or None if no match was found.
    ///
    /// Requirements: 8.1
    pub async fn auto_scrape(&self, title: &str) -> Result<Option<serde_json::Value>> {
        // Search for the title
        let results = self.search(title).await?;

        // If no results, return None
        if results.is_empty() {
            return Ok(None);
        }

        // Get detailed info for the first result
        let first_result = &results[0];
        let metadata = self.get_subject(first_result.id).await?;

        Ok(Some(metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bangumi_search_result_from_item() {
        let item = BangumiSearchItem {
            id: 12345,
            name: "Test Manga".to_string(),
            name_cn: Some("测试漫画".to_string()),
            summary: Some("A test manga".to_string()),
            images: Some(BangumiImages {
                large: Some("https://example.com/large.jpg".to_string()),
                medium: Some("https://example.com/medium.jpg".to_string()),
                small: None,
            }),
        };

        let result: BangumiSearchResult = item.into();

        assert_eq!(result.id, 12345);
        assert_eq!(result.name, "Test Manga");
        assert_eq!(result.name_cn, Some("测试漫画".to_string()));
        assert_eq!(result.summary, Some("A test manga".to_string()));
        assert_eq!(
            result.image,
            Some("https://example.com/large.jpg".to_string())
        );
    }

    #[test]
    fn test_bangumi_search_result_image_fallback() {
        let item = BangumiSearchItem {
            id: 12345,
            name: "Test".to_string(),
            name_cn: None,
            summary: None,
            images: Some(BangumiImages {
                large: None,
                medium: Some("https://example.com/medium.jpg".to_string()),
                small: None,
            }),
        };

        let result: BangumiSearchResult = item.into();
        assert_eq!(
            result.image,
            Some("https://example.com/medium.jpg".to_string())
        );
    }

    #[test]
    fn test_bangumi_search_result_no_images() {
        let item = BangumiSearchItem {
            id: 12345,
            name: "Test".to_string(),
            name_cn: None,
            summary: None,
            images: None,
        };

        let result: BangumiSearchResult = item.into();
        assert_eq!(result.image, None);
    }

    #[test]
    fn test_bangumi_service_new() {
        let service = BangumiService::new(None);
        assert!(service.api_key.is_none());

        let service_with_key = BangumiService::new(Some("test_key".to_string()));
        assert_eq!(service_with_key.api_key, Some("test_key".to_string()));
    }
}
