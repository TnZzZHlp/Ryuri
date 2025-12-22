use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use rust_i18n::t;
use crate::error::{Result, AppError};

#[derive(Debug, Deserialize)]
pub struct ListDirectoriesQuery {
    path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DirectoryEntry {
    name: String,
    path: String,
    parent: Option<String>,
}

pub async fn list_directories(
    Query(query): Query<ListDirectoriesQuery>,
) -> Result<Json<Vec<DirectoryEntry>>> {
    let path_str = query.path.unwrap_or_default();
    
    // Windows logic: if path is empty, list drives
    if cfg!(target_os = "windows") && path_str.is_empty() {
        let mut drives = Vec::new();
        for c in b'A'..=b'Z' {
            let drive_root = format!("{}:\\", c as char);
            if Path::new(&drive_root).exists() {
                drives.push(DirectoryEntry {
                    name: drive_root.clone(),
                    path: drive_root,
                    parent: None,
                });
            }
        }
        return Ok(Json(drives));
    }

    // Determine the path to list
    // If path string is empty (and not Windows root case handled above), assume root "/"
    // This primarily handles the Unix case where empty string -> root
    let path = if path_str.is_empty() {
        Path::new("/")
    } else {
        Path::new(&path_str)
    };

    if !path.exists() {
        return Err(AppError::NotFound(t!("filesystem.path_not_found", path = path.display()).to_string()));
    }

    // Check if it's a directory
    if !path.is_dir() {
        return Err(AppError::BadRequest(t!("filesystem.path_not_dir", path = path.display()).to_string()));
    }

    let parent = path.parent().map(|p| p.to_string_lossy().to_string());

    let mut dirs = Vec::new();

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type()
                    && file_type.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        dirs.push(DirectoryEntry {
                            name,
                            path: entry.path().to_string_lossy().to_string(),
                            parent: parent.clone(),
                        });
                    }
            }
        }
        Err(e) => {
             // If we can't read the directory (permission denied, etc.), just return error
             return Err(AppError::FileSystem(e));
        }
    }
    
    // Sort by name case-insensitively for better UX
    dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(Json(dirs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_list_directories() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path();
        
        // Create subdirectories
        fs::create_dir(dir_path.join("sub1")).unwrap();
        fs::create_dir(dir_path.join("sub2")).unwrap();
        fs::write(dir_path.join("file.txt"), "content").unwrap();

        let query = ListDirectoriesQuery {
            path: Some(dir_path.to_string_lossy().to_string()),
        };

        let result = list_directories(Query(query)).await.unwrap();
        let entries = result.0;

        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|e| e.name == "sub1"));
        assert!(entries.iter().any(|e| e.name == "sub2"));
        // Ensure files are ignored
        assert!(!entries.iter().any(|e| e.name == "file.txt"));
    }
}
