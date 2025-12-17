//! komga-related data models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
pub struct NewApiKey {
    pub user_id: i64,
    pub name: String,
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct ApiKey {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_serialization() {
        let api_key = ApiKey {
            id: 1,
            user_id: 42,
            name: "test_key".to_string(),
            api_key: "test_api_key".to_string(),
            created_at: Utc::now(),
        };
        let serialized = serde_json::to_string(&api_key).expect("Failed to serialize");
        let deserialized: ApiKey =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(api_key.id, deserialized.id);
        assert_eq!(api_key.user_id, deserialized.user_id);
        assert_eq!(api_key.name, deserialized.name);
        assert_eq!(api_key.api_key, deserialized.api_key);
        assert_eq!(api_key.created_at, deserialized.created_at);
    }
}
