// src/secure_delete/file_operations.rs - File attribute and reboot operations
use std::path::Path;
use std::fs::OpenOptions;
use anyhow::Result;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

// Windows-specific constants
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
        format!("\\\\?\\{}", path.display())
    } else {
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
        .chain(std::iter::once(0))
        .collect();
    
    wide
}

/// Remove Windows file attributes that prevent deletion
#[cfg(windows)]
pub fn remove_file_attributes(path: &Path) -> Result<()> {
    let wide_path = path_to_wide_string(path);
    
    let result = unsafe {
        SetFileAttributesW(wide_path.as_ptr(), 0) // 0 = FILE_ATTRIBUTE_NORMAL
    };
    
    if result == 0 {
        let error = unsafe { GetLastError() };
        eprintln!("Warning: Failed to remove file attributes for {}: Error {}", 
                  path.display(), error);
    }
    
    Ok(())
}

/// Schedule file for deletion on next reboot (Windows)
#[cfg(windows)]
pub fn schedule_deletion_on_reboot(path: &Path) -> Result<()> {
    let wide_path = path_to_wide_string(path);
    
    let result = unsafe {
        MoveFileExW(
            wide_path.as_ptr(),
            std::ptr::null(),
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

/// Test if we can access the file for writing (simple lock test)
pub fn test_file_access(path: &Path) -> bool {
    match OpenOptions::new().write(true).open(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

// Non-Windows stubs
#[cfg(not(windows))]
pub fn remove_file_attributes(_path: &Path) -> Result<()> {
    Ok(()) // No-op on non-Windows
}

#[cfg(not(windows))]
pub fn schedule_deletion_on_reboot(_path: &Path) -> Result<()> {
    Err(anyhow::anyhow!("Reboot deletion not supported on this platform"))
}