use std::io::{Write, Seek};
use std::path::Path;
use std::fs::OpenOptions;
use anyhow::{Context, Result};
use rand::RngCore;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

// Windows-specific constants and types
#[cfg(windows)]
const FILE_ATTRIBUTE_READONLY: u32 = 0x00000001;
#[cfg(windows)]
const FILE_ATTRIBUTE_HIDDEN: u32 = 0x00000002;
#[cfg(windows)]
const FILE_ATTRIBUTE_SYSTEM: u32 = 0x00000004;
#[cfg(windows)]
const MOVEFILE_DELAY_UNTIL_REBOOT: u32 = 0x00000004;

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn SetFileAttributesW(lpFileName: *const u16, dwFileAttributes: u32) -> i32;
    fn MoveFileExW(
        lpExistingFileName: *const u16,
        lpNewFileName: *const u16,
        dwFlags: u32,
    ) -> i32;
    fn GetLastError() -> u32;
}

/// Convert a path to Windows wide string format with \\?\ prefix for long paths
#[cfg(windows)]
fn path_to_wide_string(path: &Path) -> Vec<u16> {
    let path_str = if path.is_absolute() {
        // Add \\?\ prefix for long path support
        format!("\\\\?\\{}", path.display())
    } else {
        // Convert relative path to absolute first
        match std::env::current_dir() {
            Ok(current) => {
                let absolute_path = current.join(path);
                format!("\\\\?\\{}", absolute_path.display())
            }
            Err(_) => path.display().to_string(),
        }
    };
    
    let wide: Vec<u16> = OsStr::new(&path_str)
        .encode_wide()
        .chain(std::iter::once(0)) // Null terminator
        .collect();
    
    wide
}

/// Remove Windows file attributes that prevent deletion
#[cfg(windows)]
fn remove_file_attributes(path: &Path) -> Result<()> {
    let wide_path = path_to_wide_string(path);
    
    // Remove READ_ONLY, HIDDEN, and SYSTEM attributes
    let result = unsafe {
        SetFileAttributesW(wide_path.as_ptr(), 0) // 0 = FILE_ATTRIBUTE_NORMAL
    };
    
    if result == 0 {
        let error = unsafe { GetLastError() };
        eprintln!("Warning: Failed to remove file attributes for {}: Error {}", 
                  path.display(), error);
        // Don't fail completely, just warn
    }
    
    Ok(())
}

/// Schedule file for deletion on next reboot (Windows)
#[cfg(windows)]
fn schedule_deletion_on_reboot(path: &Path) -> Result<()> {
    let wide_path = path_to_wide_string(path);
    
    let result = unsafe {
        MoveFileExW(
            wide_path.as_ptr(),
            std::ptr::null(), // Delete on reboot
            MOVEFILE_DELAY_UNTIL_REBOOT,
        )
    };
    
    if result == 0 {
        let error = unsafe { GetLastError() };
        return Err(anyhow::anyhow!(
            "Failed to schedule deletion on reboot: Error {}", error
        ));
    }
    
    println!("Scheduled for deletion on next reboot: {}", path.display());
    Ok(())
}

/// Find processes that have a file open (simplified version)
#[cfg(windows)]
fn find_file_lock_owners(path: &Path) -> Vec<u32> {
    // This is a simplified version. In practice, you'd use:
    // - NtQuerySystemInformation with SystemHandleInformation
    // - NtQueryObject to get handle names
    // - Match against the file path
    // For now, we'll just return an empty vec since this requires
    // more complex Windows API calls
    
    eprintln!("Note: Process detection for {} not implemented in this version", path.display());
    Vec::new()
}

/// Attempt to terminate processes that have the file locked
#[cfg(windows)]
fn terminate_lock_owners(path: &Path) -> Result<()> {
    let pids = find_file_lock_owners(path);
    
    if pids.is_empty() {
        return Ok(());
    }
    
    println!("Found {} processes potentially locking {}", pids.len(), path.display());
    
    // In a real implementation, you'd:
    // 1. Ask user for confirmation
    // 2. Use OpenProcess + TerminateProcess
    // 3. Handle privileges (need SeDebugPrivilege)
    
    eprintln!("Warning: Process termination not implemented. Manual intervention may be required.");
    
    Ok(())
}

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
                    // Step 3: Try to identify and terminate lock owners
                    if let Err(e) = terminate_lock_owners(&path) {
                        eprintln!("Warning: Could not handle file locks: {}", e);
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
                // Step 5: Schedule for deletion on reboot (Windows only)
                println!("Attempting to schedule deletion on next reboot...");
                match schedule_deletion_on_reboot(&path) {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "All deletion methods failed. Last error: {}", e
                        ));
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

/// Perform secure overwrite of file contents
fn secure_overwrite(path: &Path, file_size: usize) -> Result<()> {
    // Open file for writing (don't truncate to preserve size)
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(false)
        .open(path)
        .with_context(|| format!("Failed to open file for overwriting: {}", path.display()))?;
    
    const PASSES: u8 = 3;
    let mut rng = rand::thread_rng();
    
    for pass in 1..=PASSES {
        println!("  Overwrite pass {}/{}", pass, PASSES);
        
        // Seek to beginning
        file.seek(std::io::SeekFrom::Start(0))
            .with_context(|| format!("Failed to seek to start on pass {}", pass))?;
        
        // Generate and write random data in chunks for large files
        const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks
        let mut remaining = file_size;
        
        while remaining > 0 {
            let chunk_size = remaining.min(CHUNK_SIZE);
            let mut random_data = vec![0u8; chunk_size];
            rng.fill_bytes(&mut random_data);
            
            file.write_all(&random_data)
                .with_context(|| format!("Failed to write random data on pass {}", pass))?;
            
            remaining -= chunk_size;
        }
        
        // Force flush to disk
        file.sync_all()
            .with_context(|| format!("Failed to sync file on pass {}", pass))?;
    }
    
    Ok(())
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
