use crate::error::{AppError, Result};

pub async fn download_image(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to download image from {}: {}", url, e)))?;
    let content = response.bytes().await.map_err(|e| {
        AppError::Internal(format!("Failed to read image bytes from {}: {}", url, e))
    })?;
    Ok(content.to_vec())
}

pub fn init_i18n() {
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"));
    rust_i18n::set_locale(&locale);
}
