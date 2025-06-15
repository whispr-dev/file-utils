// src/process_hunter.rs - PROCWOLF process hunting and termination
use std::path::Path;
use anyhow::Result;
use std::io::{self, Write};

#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use std::mem;

use crate::file_operations::test_file_access;

// Windows-specific constants and types
#[cfg(windows)]
const PROCESS_TERMINATE: u32 = 0x0001;
#[cfg(windows)]
const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
#[cfg(windows)]
const PROCESS_VM_READ: u32 = 0x0010;
#[cfg(windows)]
const TH32CS_SNAPPROCESS: u32 = 0x00000002;
#[cfg(windows)]
const TH32CS_SNAPTHREAD: u32 = 0x00000004;
#[cfg(windows)]
const THREAD_SUSPEND_RESUME: u32 = 0x0002;

#[cfg(windows)]
#[repr(C)]
struct ProcessEntry32W {
    dw_size: u32,
    cnt_usage: u32,
    th32_process_id: u32,
    th32_default_heap_id: usize,
    th32_module_id: u32,
    cnt_threads: u32,
    th32_parent_process_id: u32,
    pc_pri_class_base: i32,
    dw_flags: u32,
    sz_exe_file: [u16; 260],
}

#[cfg(windows)]
#[repr(C)]
struct ThreadEntry32 {
    dw_size: u32,
    cnt_usage: u32,
    th32_thread_id: u32,
    th32_owner_process_id: u32,
    tpri_base: i32,
    tpri_delta: i32,
    dw_flags: u32,
}

#[cfg(windows)]
type Handle = *mut std::ffi::c_void;

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn CreateToolhelp32Snapshot(dwFlags: u32, th32ProcessID: u32) -> Handle;
    fn Process32FirstW(hSnapshot: Handle, lppe: *mut ProcessEntry32W) -> i32;
    fn Process32NextW(hSnapshot: Handle, lppe: *mut ProcessEntry32W) -> i32;
    fn Thread32First(hSnapshot: Handle, lpte: *mut ThreadEntry32) -> i32;
    fn Thread32Next(hSnapshot: Handle, lpte: *mut ThreadEntry32) -> i32;
    fn CloseHandle(hObject: Handle) -> i32;
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> Handle;
    fn TerminateProcess(hProcess: Handle, uExitCode: u32) -> i32;
    fn GetProcessImageFileNameW(hProcess: Handle, lpImageFileName: *mut u16, nSize: u32) -> u32;
    fn OpenThread(dwDesiredAccess: u32, bInheritHandle: i32, dwThreadId: u32) -> Handle;
    fn SuspendThread(hThread: Handle) -> u32;
    fn ResumeThread(hThread: Handle) -> u32;
    fn GetCurrentProcessId() -> u32;
    fn GetLastError() -> u32;
}

#[cfg(windows)]
const INVALID_HANDLE_VALUE: Handle = (-1isize) as Handle;

/// Process information structure
#[cfg(windows)]
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub parent_pid: u32,
}

/// Convert wide string to regular string
#[cfg(windows)]
fn wide_string_to_string(wide: &[u16]) -> String {
    let null_pos = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..null_pos])
}

/// Enumerate all running processes - PROCWOLF style
#[cfg(windows)]
fn enumerate_processes() -> Result<Vec<ProcessInfo>> {
    let mut processes = Vec::new();
    
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(anyhow::anyhow!("Failed to create process snapshot"));
    }
    
    let mut process_entry = ProcessEntry32W {
        dw_size: mem::size_of::<ProcessEntry32W>() as u32,
        cnt_usage: 0,
        th32_process_id: 0,
        th32_default_heap_id: 0,
        th32_module_id: 0,
        cnt_threads: 0,
        th32_parent_process_id: 0,
        pc_pri_class_base: 0,
        dw_flags: 0,
        sz_exe_file: [0; 260],
    };
    
    let mut result = unsafe { Process32FirstW(snapshot, &mut process_entry) };
    
    while result != 0 {
        let name = wide_string_to_string(&process_entry.sz_exe_file);
        let path = get_process_path(process_entry.th32_process_id);
        
        processes.push(ProcessInfo {
            pid: process_entry.th32_process_id,
            name,
            path,
            parent_pid: process_entry.th32_parent_process_id,
        });
        
        result = unsafe { Process32NextW(snapshot, &mut process_entry) };
    }
    
    unsafe { CloseHandle(snapshot) };
    Ok(processes)
}

/// Get the full path of a process by PID
#[cfg(windows)]
fn get_process_path(pid: u32) -> Option<String> {
    let process_handle = unsafe { 
        OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid) 
    };
    
    if process_handle == ptr::null_mut() || process_handle == INVALID_HANDLE_VALUE {
        return None;
    }
    
    let mut path_buffer: [u16; 512] = [0; 512];
    let result = unsafe { 
        GetProcessImageFileNameW(process_handle, path_buffer.as_mut_ptr(), 512) 
    };
    
    unsafe { CloseHandle(process_handle) };
    
    if result > 0 {
        Some(wide_string_to_string(&path_buffer[..result as usize]))
    } else {
        None
    }
}

/// Kill a process by PID
#[cfg(windows)]
fn kill_process_by_pid(pid: u32, force: bool) -> Result<bool> {
    let process_handle = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid) };
    
    if process_handle == ptr::null_mut() || process_handle == INVALID_HANDLE_VALUE {
        let error = unsafe { GetLastError() };
        return Err(anyhow::anyhow!("Failed to open process {}: Error {}", pid, error));
    }
    
    let exit_code = if force { 1 } else { 0 };
    let result = unsafe { TerminateProcess(process_handle, exit_code) };
    
    unsafe { CloseHandle(process_handle) };
    
    if result != 0 {
        Ok(true)
    } else {
        let error = unsafe { GetLastError() };
        Err(anyhow::anyhow!("Failed to terminate process {}: Error {}", pid, error))
    }
}

/// Suspend a process by PID
#[cfg(windows)]
fn suspend_process_by_pid(pid: u32) -> Result<()> {
    let thread_snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if thread_snapshot == INVALID_HANDLE_VALUE {
        return Err(anyhow::anyhow!("Failed to create thread snapshot"));
    }
    
    let mut thread_entry = ThreadEntry32 {
        dw_size: mem::size_of::<ThreadEntry32>() as u32,
        cnt_usage: 0,
        th32_thread_id: 0,
        th32_owner_process_id: 0,
        tpri_base: 0,
        tpri_delta: 0,
        dw_flags: 0,
    };
    
    let mut result = unsafe { Thread32First(thread_snapshot, &mut thread_entry) };
    let mut suspended_count = 0;
    
    while result != 0 {
        if thread_entry.th32_owner_process_id == pid {
            let thread_handle = unsafe { 
                OpenThread(THREAD_SUSPEND_RESUME, 0, thread_entry.th32_thread_id) 
            };
            
            if thread_handle != ptr::null_mut() && thread_handle != INVALID_HANDLE_VALUE {
                unsafe { SuspendThread(thread_handle) };
                unsafe { CloseHandle(thread_handle) };
                suspended_count += 1;
            }
        }
        
        result = unsafe { Thread32Next(thread_snapshot, &mut thread_entry) };
    }
    
    unsafe { CloseHandle(thread_snapshot) };
    
    if suspended_count > 0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!("No threads found or suspended for PID: {}", pid))
    }
}

/// Find processes that might be locking a file
#[cfg(windows)]
fn find_file_lock_owners(path: &Path) -> Vec<u32> {
    let mut lock_owners = Vec::new();
    
    if let Ok(processes) = enumerate_processes() {
        for process in processes {
            let process_name = process.name.to_lowercase();
            
            // Common file-locking processes
            if process_name.contains("explorer") ||
               process_name.contains("notepad") ||
               process_name.contains("wordpad") ||
               process_name.contains("winword") ||
               process_name.contains("excel") ||
               process_name.contains("powerpnt") ||
               process_name.contains("outlook") ||
               process_name.contains("acrobat") ||
               process_name.contains("chrome") ||
               process_name.contains("firefox") ||
               process_name.contains("edge") ||
               process_name.contains("photoshop") ||
               process_name.contains("vlc") ||
               process_name.contains("media") {
                
                lock_owners.push(process.pid);
            }
            
            // Check if process is in same directory
            if let Some(process_path) = process.path {
                let proc_dir = std::path::Path::new(&process_path)
                    .parent()
                    .map(|p| p.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                
                let target_dir = path.parent()
                    .map(|p| p.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                
                if !proc_dir.is_empty() && !target_dir.is_empty() && proc_dir == target_dir {
                    if !lock_owners.contains(&process.pid) {
                        lock_owners.push(process.pid);
                    }
                }
            }
        }
    }
    
    // Filter out critical system processes
    lock_owners.retain(|&pid| {
        if let Ok(processes) = enumerate_processes() {
            if let Some(process) = processes.iter().find(|p| p.pid == pid) {
                let name = process.name.to_lowercase();
                !name.contains("system") &&
                !name.contains("csrss") &&
                !name.contains("winlogon") &&
                !name.contains("services") &&
                !name.contains("svchost") &&
                !name.contains("dwm") &&
                !name.contains("wininit")
            } else {
                false
            }
        } else {
            false
        }
    });
    
    lock_owners
}

/// The PROCWOLF - Attempt to terminate processes that have a file locked
#[cfg(windows)]
pub fn terminate_lock_owners(path: &Path) -> Result<()> {
    println!("🐺 PROCWOLF activated - hunting file lock owners for: {}", path.display());
    
    let pids = find_file_lock_owners(path);
    
    if pids.is_empty() {
        println!("No obvious file lock owners detected");
        return Ok(());
    }
    
    println!("Found {} potential file lock owners:", pids.len());
    
    // Get process details before termination
    let processes = enumerate_processes()?;
    let mut targets = Vec::new();
    
    for pid in pids {
        if let Some(process) = processes.iter().find(|p| p.pid == pid) {
            println!("  - {} (PID: {}) - Path: {:?}", 
                     process.name, process.pid, process.path);
            targets.push(process.clone());
        }
    }
    
    if targets.is_empty() {
        return Ok(());
    }
    
    // Strategy 1: Try to suspend processes first (less aggressive)
    println!("\n🐺 Phase 1: Attempting to suspend lock owners...");
    for process in &targets {
        match suspend_process_by_pid(process.pid) {
            Ok(()) => println!("  ✓ Suspended: {} (PID: {})", process.name, process.pid),
            Err(e) => eprintln!("  ✗ Failed to suspend {} (PID: {}): {}", process.name, process.pid, e),
        }
    }
    
    // Give a moment for file handles to be released
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // Test if file is now accessible
    if test_file_access(path) {
        println!("✓ File is now accessible after suspension - lock owners neutralized!");
        return Ok(());
    }
    
    // Strategy 2: Terminate processes (more aggressive)
    println!("\n🐺 Phase 2: File still locked - initiating termination protocol...");
    
    let mut terminated_count = 0;
    for process in &targets {
        // Skip certain "safer" processes in first pass
        if process.name.to_lowercase().contains("explorer") {
            println!("  ⚠ Skipping explorer.exe (first pass) - PID: {}", process.pid);
            continue;
        }
        
        match kill_process_by_pid(process.pid, false) {
            Ok(true) => {
                println!("  ✓ Terminated: {} (PID: {})", process.name, process.pid);
                terminated_count += 1;
            }
            Ok(false) => {
                eprintln!("  ✗ Termination returned false: {} (PID: {})", process.name, process.pid);
            }
            Err(e) => {
                eprintln!("  ✗ Failed to terminate {} (PID: {}): {}", process.name, process.pid, e);
            }
        }
    }
    
    if terminated_count > 0 {
        // Give time for cleanup
        std::thread::sleep(std::time::Duration::from_millis(1000));
        
        if test_file_access(path) {
            println!("✓ File is now accessible after termination - PROCWOLF successful!");
            return Ok(());
        }
    }
    
    // Strategy 3: Nuclear option - terminate everything including explorer
    println!("\n🐺 Phase 3: Nuclear option - terminating all remaining lock owners...");
    for process in &targets {
        if process.name.to_lowercase().contains("explorer") {
            println!("  ⚠ Terminating explorer.exe - PID: {} (Windows shell will restart)", process.pid);
        }
        
        match kill_process_by_pid(process.pid, true) {
            Ok(true) => println!("  ✓ Force terminated: {} (PID: {})", process.name, process.pid),
            Ok(false) => eprintln!("  ✗ Force termination returned false: {} (PID: {})", process.name, process.pid),
            Err(e) => eprintln!("  ✗ Failed to force terminate {} (PID: {}): {}", process.name, process.pid, e),
        }
    }
    
    // Final test
    std::thread::sleep(std::time::Duration::from_millis(1500));
    if test_file_access(path) {
        println!("✓ File is now accessible - PROCWOLF mission accomplished!");
    } else {
        println!("⚠ File may still be locked - manual intervention may be required");
    }
    
    Ok(())
}

/// Utility function to manually deploy PROCWOLF on a specific file
#[cfg(windows)]
pub fn deploy_procwolf(file_path: &Path) -> Result<()> {
    terminate_lock_owners(file_path)
}

/// Show running processes that might be locking a file - diagnostic function
#[cfg(windows)]
pub fn show_potential_lock_owners(file_path: &Path) -> Result<()> {
    println!("Scanning for potential lock owners of: {}", file_path.display());
    
    let pids = find_file_lock_owners(file_path);
    
    if pids.is_empty() {
        println!("No obvious lock owners detected");
        return Ok(());
    }
    
    let processes = enumerate_processes()?;
    
    println!("Potential lock owners found:");
    for pid in pids {
        if let Some(process) = processes.iter().find(|p| p.pid == pid) {
            println!("  PID: {} | Name: {} | Path: {:?}", 
                     process.pid, process.name, process.path);
        }
    }
    
    Ok(())
}

/// Advanced process hunting by partial name match
#[cfg(windows)]
pub fn hunt_and_terminate(name_pattern: &str, force: bool, dry_run: bool) -> Result<Vec<u32>> {
    println!("🐺 PROCWOLF hunting mode - searching for: '{}'", name_pattern);
    
    let processes = enumerate_processes()?;
    let mut targets = Vec::new();
    
    // Find matching processes
    for process in processes {
        let name_match = process.name.to_lowercase().contains(&name_pattern.to_lowercase());
        let path_match = if let Some(ref path) = process.path {
            path.to_lowercase().contains(&name_pattern.to_lowercase())
        } else {
            false
        };
        
        if name_match || path_match {
            targets.push(process);
        }
    }
    
    if targets.is_empty() {
        println!("No processes found matching pattern: '{}'", name_pattern);
        return Ok(Vec::new());
    }
    
    println!("Found {} matching processes:", targets.len());
    for target in &targets {
        println!("  - {} (PID: {}) - Path: {:?}", target.name, target.pid, target.path);
    }
    
    if dry_run {
        println!("DRY RUN: Would terminate {} processes", targets.len());
        return Ok(targets.iter().map(|p| p.pid).collect());
    }
    
    // Confirm termination for multiple processes
    if targets.len() > 1 {
        print!("Terminate {} processes? [y/N]: ", targets.len());
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Operation cancelled");
            return Ok(Vec::new());
        }
    }
    
    let mut killed_pids = Vec::new();
    
    for target in &targets {
        match kill_process_by_pid(target.pid, force) {
            Ok(true) => {
                println!("✓ Terminated: {} (PID: {})", target.name, target.pid);
                killed_pids.push(target.pid);
            }
            Ok(false) => {
                eprintln!("✗ Failed to terminate {} (PID: {})", target.name, target.pid);
            }
            Err(e) => {
                eprintln!("✗ Failed to terminate {} (PID: {}): {}", target.name, target.pid, e);
            }
        }
    }
    
    println!("PROCWOLF hunt complete - {} processes terminated", killed_pids.len());
    Ok(killed_pids)
}

/// Resume all threads of a previously suspended process
#[cfg(windows)]
pub fn resume_process_by_pid(pid: u32) -> Result<()> {
    let thread_snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if thread_snapshot == INVALID_HANDLE_VALUE {
        return Err(anyhow::anyhow!("Failed to create thread snapshot"));
    }
    
    let mut thread_entry = ThreadEntry32 {
        dw_size: mem::size_of::<ThreadEntry32>() as u32,
        cnt_usage: 0,
        th32_thread_id: 0,
        th32_owner_process_id: 0,
        tpri_base: 0,
        tpri_delta: 0,
        dw_flags: 0,
    };
    
    let mut result = unsafe { Thread32First(thread_snapshot, &mut thread_entry) };
    let mut resumed_count = 0;
    
    while result != 0 {
        if thread_entry.th32_owner_process_id == pid {
            let thread_handle = unsafe { 
                OpenThread(THREAD_SUSPEND_RESUME, 0, thread_entry.th32_thread_id) 
            };
            
            if thread_handle != ptr::null_mut() && thread_handle != INVALID_HANDLE_VALUE {
                unsafe { ResumeThread(thread_handle) };
                unsafe { CloseHandle(thread_handle) };
                resumed_count += 1;
            }
        }
        
        result = unsafe { Thread32Next(thread_snapshot, &mut thread_entry) };
    }
    
    unsafe { CloseHandle(thread_snapshot) };
    
    if resumed_count > 0 {
        println!("Resumed {} threads for process PID: {}", resumed_count, pid);
        Ok(())
    } else {
        Err(anyhow::anyhow!("No threads found or resumed for PID: {}", pid))
    }
}

/// List all running processes with detailed information
#[cfg(windows)]
pub fn list_all_processes(filter: Option<&str>) -> Result<()> {
    let processes = enumerate_processes()?;
    
    let filtered_processes: Vec<_> = if let Some(filter_str) = filter {
        processes.into_iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&filter_str.to_lowercase()) ||
                p.path.as_ref().map_or(false, |path| 
                    path.to_lowercase().contains(&filter_str.to_lowercase()))
            })
            .collect()
    } else {
        processes
    };
    
    if filtered_processes.is_empty() {
        println!("No processes found{}", 
                 filter.map_or(String::new(), |f| format!(" matching '{}'", f)));
        return Ok(());
    }
    
    println!("\n{:-<120}", "");
    println!("{:>8} | {:>8} | {:<30} | {}", "PID", "PPID", "Process Name", "Path");
    println!("{:-<120}", "");
    
    for process in &filtered_processes {
        let path_display = process.path.as_deref().unwrap_or("N/A");
        println!("{:>8} | {:>8} | {:<30} | {}", 
                 process.pid, 
                 process.parent_pid, 
                 &process.name[..process.name.len().min(30)], 
                 path_display);
    }
    
    println!("{:-<120}", "");
    println!("Total: {} processes{}", 
             filtered_processes.len(),
             filter.map_or(String::new(), |f| format!(" (filtered by '{}')", f)));
    
    Ok(())
}

/// Emergency process termination - kill by PID with maximum force
#[cfg(windows)]
pub fn emergency_terminate(pid: u32) -> Result<()> {
    println!("🚨 EMERGENCY TERMINATION for PID: {}", pid);
    
    // First try to get process info
    if let Ok(processes) = enumerate_processes() {
        if let Some(process) = processes.iter().find(|p| p.pid == pid) {
            println!("Target: {} - Path: {:?}", process.name, process.path);
            
            // Warn about critical processes
            let name_lower = process.name.to_lowercase();
            if name_lower.contains("system") || 
               name_lower.contains("csrss") || 
               name_lower.contains("winlogon") ||
               name_lower.contains("services") {
                println!("⚠️  WARNING: This appears to be a critical system process!");
                print!("Continue with termination? [y/N]: ");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Emergency termination cancelled");
                    return Ok(());
                }
            }
        }
    }
    
    // Try progressive termination methods
    println!("Phase 1: Attempting graceful termination...");
    match kill_process_by_pid(pid, false) {
        Ok(true) => {
            println!("✓ Process terminated gracefully");
            return Ok(());
        }
        Ok(false) => {
            eprintln!("Graceful termination returned false");
        }
        Err(e) => {
            eprintln!("Graceful termination failed: {}", e);
        }
    }
    
    println!("Phase 2: Attempting forced termination...");
    match kill_process_by_pid(pid, true) {
        Ok(true) => {
            println!("✓ Process force terminated");
            return Ok(());
        }
        Ok(false) => {
            eprintln!("Force termination returned false");
        }
        Err(e) => {
            eprintln!("Force termination failed: {}", e);
        }
    }
    
    println!("Phase 3: Attempting thread suspension...");
    match suspend_process_by_pid(pid) {
        Ok(()) => {
            println!("✓ Process threads suspended");
            
            // Wait a moment then try termination again
            std::thread::sleep(std::time::Duration::from_millis(1000));
            
            match kill_process_by_pid(pid, true) {
                Ok(true) => {
                    println!("✓ Suspended process terminated");
                    return Ok(());
                }
                Ok(false) => {
                    eprintln!("Suspended process termination returned false");
                }
                Err(e) => {
                    eprintln!("Failed to terminate suspended process: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Thread suspension failed: {}", e);
        }
    }
    
    Err(anyhow::anyhow!("All emergency termination methods failed for PID: {}", pid))
}

/// Get detailed information about a specific process
#[cfg(windows)]
pub fn get_process_details(pid: u32) -> Result<ProcessInfo> {
    let processes = enumerate_processes()?;
    
    processes.into_iter()
        .find(|p| p.pid == pid)
        .ok_or_else(|| anyhow::anyhow!("Process with PID {} not found", pid))
}

/// Check if current process has admin privileges
#[cfg(windows)]
pub fn is_admin() -> bool {
    use std::ptr;
    
    #[repr(C)]
    struct SidIdentifierAuthority {
        value: [u8; 6],
    }
    
    #[link(name = "advapi32")]
    extern "system" {
        fn CheckTokenMembership(
            TokenHandle: *mut std::ffi::c_void,
            SidToCheck: *mut std::ffi::c_void,
            IsMember: *mut i32,
        ) -> i32;
        
        fn AllocateAndInitializeSid(
            pIdentifierAuthority: *const SidIdentifierAuthority,
            nSubAuthorityCount: u8,
            nSubAuthority0: u32,
            nSubAuthority1: u32,
            nSubAuthority2: u32,
            nSubAuthority3: u32,
            nSubAuthority4: u32,
            nSubAuthority5: u32,
            nSubAuthority6: u32,
            nSubAuthority7: u32,
            pSid: *mut *mut std::ffi::c_void,
        ) -> i32;
        
        fn FreeSid(pSid: *mut std::ffi::c_void);
    }
    
    const SECURITY_NT_AUTHORITY: SidIdentifierAuthority = SidIdentifierAuthority {
        value: [0, 0, 0, 0, 0, 5],
    };
    const SECURITY_BUILTIN_DOMAIN_RID: u32 = 0x00000020;
    const DOMAIN_ALIAS_RID_ADMINS: u32 = 0x00000220;
    
    let mut admin_group: *mut std::ffi::c_void = ptr::null_mut();
    let mut is_member: i32 = 0;
    
    let result = unsafe {
        AllocateAndInitializeSid(
            &SECURITY_NT_AUTHORITY,
            2,
            SECURITY_BUILTIN_DOMAIN_RID,
            DOMAIN_ALIAS_RID_ADMINS,
            0, 0, 0, 0, 0, 0,
            &mut admin_group,
        )
    };
    
    if result != 0 {
        let check_result = unsafe {
            CheckTokenMembership(ptr::null_mut(), admin_group, &mut is_member)
        };
        
        unsafe { FreeSid(admin_group) };
        
        check_result != 0 && is_member != 0
    } else {
        false
    }
}

/// PROCWOLF status and system information
#[cfg(windows)]
pub fn procwolf_status() -> Result<()> {
    println!("🐺 PROCWOLF System Status");
    println!("{:-<50}", "");
    
    // Check admin privileges
    let is_admin_user = is_admin();
    println!("Administrator privileges: {}", if is_admin_user { "✓ YES" } else { "✗ NO" });
    
    if !is_admin_user {
        println!("⚠️  Some PROCWOLF functions require administrator privileges");
    }
    
    // Get current process info
    let current_pid = unsafe { GetCurrentProcessId() };
    println!("PROCWOLF PID: {}", current_pid);
    
    // Count total processes
    match enumerate_processes() {
        Ok(processes) => {
            println!("Total processes visible: {}", processes.len());
            
            // Count by type
            let mut system_processes = 0;
            let mut user_processes = 0;
            
            for process in &processes {
                if process.name.to_lowercase().contains("system") ||
                   process.name.to_lowercase().contains("svchost") ||
                   process.name.to_lowercase().contains("csrss") {
                    system_processes += 1;
                } else {
                    user_processes += 1;
                }
            }
            
            println!("  - System processes: ~{}", system_processes);
            println!("  - User processes: ~{}", user_processes);
        }
        Err(e) => {
            eprintln!("Failed to enumerate processes: {}", e);
        }
    }
    
    println!("{:-<50}", "");
    println!("PROCWOLF ready for deployment 🐺");
    
    Ok(())
}

// Non-Windows stubs to make the code compile on other platforms
#[cfg(not(windows))]
pub fn terminate_lock_owners(_path: &Path) -> Result<()> {
    eprintln!("Process termination is only implemented for Windows");
    Ok(())
}

#[cfg(not(windows))]
pub fn deploy_procwolf(_file_path: &Path) -> Result<()> {
    eprintln!("PROCWOLF is only available on Windows");
    Err(anyhow::anyhow!("PROCWOLF not supported on this platform"))
}

#[cfg(not(windows))]
pub fn show_potential_lock_owners(_file_path: &Path) -> Result<()> {
    eprintln!("Lock owner detection is only available on Windows");
    Ok(())
}

#[cfg(not(windows))]
pub fn hunt_and_terminate(_name_pattern: &str, _force: bool, _dry_run: bool) -> Result<Vec<u32>> {
    eprintln!("Process hunting is only available on Windows");
    Ok(Vec::new())
}

#[cfg(not(windows))]
pub fn list_all_processes(_filter: Option<&str>) -> Result<()> {
    eprintln!("Detailed process listing is only available on Windows");
    Ok(())
}

#[cfg(not(windows))]
pub fn emergency_terminate(_pid: u32) -> Result<()> {
    eprintln!("Emergency termination is only available on Windows");
    Err(anyhow::anyhow!("Emergency termination not supported on this platform"))
}

#[cfg(not(windows))]
pub fn resume_process_by_pid(_pid: u32) -> Result<()> {
    eprintln!("Process resume is only available on Windows");
    Err(anyhow::anyhow!("Process resume not supported on this platform"))
}

#[cfg(not(windows))]
pub fn get_process_details(_pid: u32) -> Result<ProcessInfo> {
    eprintln!("Process details are only available on Windows");
    Err(anyhow::anyhow!("Process details not supported on this platform"))
}

#[cfg(not(windows))]
pub fn is_admin() -> bool {
    false // Always false on non-Windows
}

#[cfg(not(windows))]
pub fn procwolf_status() -> Result<()> {
    println!("PROCWOLF is only available on Windows");
    Ok(())
}

// Need to define ProcessInfo for non-Windows too
#[cfg(not(windows))]
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub parent_pid: u32,
}