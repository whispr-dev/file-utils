// src/secure_delete.rs - Main module that orchestrates everything
use std::path::Path;
use anyhow::{Context, Result};

// Import our modular components from the same src directory
use crate::file_operations::{remove_file_attributes, schedule_deletion_on_reboot, test_file_access};
use crate::process_hunter::terminate_lock_owners;
use crate::secure_wipe::secure_overwrite;

// Re-export public functions from other modules
pub use crate::process_hunter::{
    deploy_procwolf, 
    show_potential_lock_owners, 
    hunt_and_terminate,
    emergency_terminate,
    list_all_processes,
    resume_process_by_pid,
    get_process_details,
    is_admin,
    procwolf_status
};

/// Cross-platform secure delete with Windows-specific stubborn file handling
pub fn secure_delete(file_path: &Path) -> Result<()> {
    let path = if file_path.is_relative() {
        std::env::current_dir()?.join(file_path)
    } else {
        file_path.to_path_buf()
    };
    
    // Check if file exists
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {}", path.display()));
    }
    
    // Get file metadata
    let metadata = std::fs::metadata(&path)
        .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;
    
    let file_size = metadata.len() as usize;
    
    println!("Attempting secure deletion of: {} ({} bytes)", path.display(), file_size);
    
    #[cfg(windows)]
    {
        // Step 1: Remove restrictive attributes
        if let Err(e) = remove_file_attributes(&path) {
            eprintln!("Warning: Could not remove file attributes: {}", e);
        }
    }
    
    // Step 2: Attempt secure overwrite
    if file_size > 0 {
        match secure_overwrite(&path, file_size) {
            Ok(_) => println!("Successfully overwrote file data"),
            Err(e) => {
                eprintln!("Warning: Could not overwrite file data: {}", e);
                
                #[cfg(windows)]
                {
                    // Step 3: Deploy the PROCWOLF to handle lock owners
                    if let Err(e) = terminate_lock_owners(&path) {
                        eprintln!("Warning: PROCWOLF could not eliminate all lock owners: {}", e);
                    }
                    
                    // Retry overwrite after PROCWOLF
                    match secure_overwrite(&path, file_size) {
                        Ok(_) => println!("Successfully overwrote file data after PROCWOLF intervention"),
                        Err(e2) => eprintln!("Still could not overwrite after PROCWOLF: {}", e2),
                    }
                }
            }
        }
    }
    
    // Step 4: Attempt standard deletion
    match std::fs::remove_file(&path) {
        Ok(_) => {
            println!("Successfully deleted: {}", path.display());
            return Ok(());
        }
        Err(e) => {
            eprintln!("Standard deletion failed: {}", e);
            
            #[cfg(windows)]
            {
                // Step 5: Final PROCWOLF attempt if deletion still fails
                println!("Deploying PROCWOLF for final deletion attempt...");
                if let Err(e) = terminate_lock_owners(&path) {
                    eprintln!("Warning: Final PROCWOLF deployment failed: {}", e);
                }
                
                // Try deletion one more time
                match std::fs::remove_file(&path) {
                    Ok(_) => {
                        println!("Successfully deleted after final PROCWOLF intervention: {}", path.display());
                        return Ok(());
                    }
                    Err(_) => {
                        // Step 6: Schedule for deletion on reboot (Windows only)
                        println!("All active methods failed - scheduling deletion on next reboot...");
                        match schedule_deletion_on_reboot(&path) {
                            Ok(_) => return Ok(()),
                            Err(e) => {
                                return Err(anyhow::anyhow!(
                                    "All deletion methods failed including reboot scheduling. Last error: {}", e
                                ));
                            }
                        }
                    }
                }
            }
            
            #[cfg(not(windows))]
            {
                return Err(anyhow::anyhow!(
                    "Could not delete file: {}. Manual intervention required.", e
                ));
            }
        }
    }
}

/// Secure delete with retry logic and multiple strategies
pub fn secure_delete_with_retry(file_path: &Path, max_retries: u32) -> Result<()> {
    let mut last_error = None;
    
    for attempt in 1..=max_retries {
        match secure_delete(file_path) {
            Ok(_) => return Ok(()),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries {
                    println!("Deletion attempt {} failed, retrying...", attempt);
                    std::thread::sleep(std::time::Duration::from_millis(100 * attempt as u64));
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error")))
}

/// Batch secure delete multiple files
pub fn secure_delete_batch<P: AsRef<Path>>(file_paths: &[P]) -> Result<()> {
    let mut failed_files = Vec::new();
    
    for (i, path) in file_paths.iter().enumerate() {
        let path = path.as_ref();
        println!("Processing file {}/{}: {}", i + 1, file_paths.len(), path.display());
        
        match secure_delete_with_retry(path, 3) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Failed to delete {}: {}", path.display(), e);
                failed_files.push(path.to_path_buf());
            }
        }
    }
    
    if !failed_files.is_empty() {
        return Err(anyhow::anyhow!(
            "Failed to delete {} files: {:?}", 
            failed_files.len(), 
            failed_files
        ));
    }
    
    Ok(())
}

/// Example usage and CLI interface
pub fn main() {
    println!("🐺 PROCWOLF Enhanced Secure Delete System");
    println!("==========================================");
    
    #[cfg(windows)]
    {
        if let Err(e) = procwolf_status() {
            eprintln!("Failed to get PROCWOLF status: {}", e);
        }
    }
    
    // Example usage patterns:
    println!("\nExample usage:");
    println!("  secure_delete(Path::new(\"locked_file.txt\"))");
    println!("  deploy_procwolf(Path::new(\"stubborn_file.exe\"))");
    println!("  hunt_and_terminate(\"malware\", true, false)");
    println!("  emergency_terminate(1234)");
    println!("  list_all_processes(Some(\"chrome\"))");
}