use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_i18n::t;

use crate::{
    error::{AppError, Result},
    middlewares::auth::AuthUser,
    models::{ApiKey, NewApiKey},
    repository::apikey::ApiKeyRepository,
    state::AppState,
};

#[derive(Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

#[derive(Serialize)]
pub struct ApiKeyResponse {
    pub id: i64,
    pub name: String,
    pub api_key: String,
    pub created_at: String,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(key: ApiKey) -> Self {
        Self {
            id: key.id,
            name: key.name,
            api_key: key.api_key,
            created_at: key.created_at.to_rfc3339(),
        }
    }
}

/// Create a new API key for the authenticated user.
pub async fn create_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<ApiKeyResponse>)> {
    let api_key_string = Uuid::new_v4().to_string();

    let new_key = NewApiKey {
        user_id: user.user_id,
        name: payload.name,
        api_key: api_key_string,
    };

    let created_key = ApiKeyRepository::create(&state.pool, new_key).await?;

    Ok((StatusCode::CREATED, Json(created_key.into())))
}

/// List all API keys for the authenticated user.
pub async fn list_api_keys(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<ApiKeyResponse>>> {
    let keys = ApiKeyRepository::list_by_user(&state.pool, user.user_id).await?;
    let response = keys.into_iter().map(ApiKeyResponse::from).collect();
    Ok(Json(response))
}

/// Delete an API key.
pub async fn delete_api_key(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<StatusCode> {
    // Verify ownership
    let keys = ApiKeyRepository::list_by_user(&state.pool, user.user_id).await?;
    if !keys.iter().any(|k| k.id == id) {
        return Err(AppError::NotFound(t!("auth.api_key_not_found").to_string()));
    }

    ApiKeyRepository::delete(&state.pool, id).await?;

    Ok(StatusCode::NO_CONTENT)
}
