use std::path::Path;
use std::fs;

pub fn list_encrypted_files(directory: &Path) -> Vec<String> {
    let mut encrypted_files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    let file_name_str = file_name.to_string_lossy();
                    if file_name_str.ends_with(".enc") {
                        encrypted_files.push(path.display().to_string());
                    }
                }
            }
        }
    }
    
    encrypted_files
}

pub fn process_path(path: &Path) -> bool {
    path.exists()
}

/// Recursively find all files in a directory
pub fn find_files_recursive(directory: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                files.extend(find_files_recursive(&path));
            }
        }
    }
    
    files
}

/// Check if a file has the .enc extension
pub fn is_encrypted_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "enc")
        .unwrap_or(false)
}

/// Get the base name without .enc extension
pub fn get_decrypted_name(path: &Path) -> Option<std::path::PathBuf> {
    if is_encrypted_file(path) {
        let mut new_path = path.to_path_buf();
        new_path.set_extension("");
        Some(new_path)
    } else {
        None
    }
}

/// Create directory if it doesn't exist
pub fn ensure_directory_exists(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}