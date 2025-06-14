use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Configuration for processing files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Output directory path (if None, use the same directory as input)
    pub output_path: Option<PathBuf>,
    
    /// Whether to process directories recursively
    pub recursive: bool,
    
    /// Whether to overwrite existing files without prompting
    pub force: bool,
    
    /// Whether to securely delete original files after processing
    pub secure_delete: bool,
    
    /// Number of passes for secure deletion
    pub shred_passes: u8,
    
    /// Whether to clean up empty folders after secure deletion
    pub clean_empty_folders: bool,
}

impl Config {
    /// Get effective output path for a given input path
    pub fn get_output_path(&self, input_path: &PathBuf, is_encrypting: bool) -> PathBuf {
        match &self.output_path {
            Some(output_dir) => {
                if output_dir.is_dir() {
                    // If output is a directory, keep the filename but in the output directory
                    let filename = input_path.file_name()
                        .unwrap_or_default();
                    
                    let mut new_path = output_dir.clone();
                    
                    // Add appropriate extension
                    if is_encrypting {
                        // For encryption, append .enc
                        let mut new_filename = filename.to_os_string();
                        new_filename.push(".enc");
                        new_path.push(new_filename);
                    } else {
                        // For decryption, remove .enc extension if present
                        let filename_str = filename.to_string_lossy();
                        if filename_str.ends_with(".enc") {
                            let original_name = &filename_str[..filename_str.len() - 4];
                            new_path.push(original_name);
                        } else {
                            // If no .enc extension, keep as is
                            new_path.push(filename);
                        }
                    }
                    
                    new_path
                } else {
                    // If output path is specified as a file, use it directly
                    output_dir.clone()
                }
            },
            None => {
                // No output path, use input directory
                let mut new_path = input_path.clone();
                
                if is_encrypting {
                    // For encryption, append .enc
                    new_path.set_extension("enc");
                } else {
                    // For decryption, remove .enc extension
                    if let Some(ext) = new_path.extension() {
                        if ext == "enc" {
                            // Remove the .enc extension
                            new_path.set_extension("");
                        }
                    }
                }
                
                new_path
            }
        }
    }
    
    /// Get the relative path for a file within a directory structure
    pub fn get_relative_output_path(
        &self, 
        file_path: &PathBuf, 
        base_path: &PathBuf, 
        is_encrypting: bool
    ) -> PathBuf {
        // Get relative path from base directory
        let relative_path = file_path.strip_prefix(base_path)
            .unwrap_or(file_path.as_path());
        
        match &self.output_path {
            Some(output_dir) => {
                let mut new_path = output_dir.clone();
                new_path.push(relative_path);
                
                // Add appropriate extension
                if is_encrypting {
                    new_path.set_extension("enc");
                } else if let Some(ext) = new_path.extension() {
                    if ext == "enc" {
                        new_path.set_extension("");
                    }
                }
                
                new_path
            },
            None => {
                // No output path, use input directory structure
                let mut new_path = file_path.clone();
                
                if is_encrypting {
                    new_path.set_extension("enc");
                } else if let Some(ext) = new_path.extension() {
                    if ext == "enc" {
                        new_path.set_extension("");
                    }
                }
                
                new_path
            }
        }
    }
}
