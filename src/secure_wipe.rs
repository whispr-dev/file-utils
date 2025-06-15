// src/secure_delete/secure_wipe.rs - Multi-pass cryptographic overwriting
use std::io::{Write, Seek};
use std::path::Path;
use std::fs::OpenOptions;
use anyhow::{Context, Result};
use rand::RngCore;

/// Perform secure overwrite of file contents with multiple passes
pub fn secure_overwrite(path: &Path, file_size: usize) -> Result<()> {
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

/// Perform a single-pass overwrite with zeros (faster but less secure)
pub fn quick_overwrite(path: &Path, file_size: usize) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(false)
        .open(path)
        .with_context(|| format!("Failed to open file for quick overwrite: {}", path.display()))?;
    
    println!("  Quick overwrite with zeros");
    
    file.seek(std::io::SeekFrom::Start(0))
        .with_context(|| "Failed to seek to start")?;
    
    const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks of zeros
    let zero_chunk = vec![0u8; CHUNK_SIZE];
    let mut remaining = file_size;
    
    while remaining > 0 {
        let chunk_size = remaining.min(CHUNK_SIZE);
        file.write_all(&zero_chunk[..chunk_size])
            .with_context(|| "Failed to write zero data")?;
        remaining -= chunk_size;
    }
    
    file.sync_all()
        .with_context(|| "Failed to sync file after quick overwrite")?;
    
    Ok(())
}

/// Perform a DoD 5220.22-M standard 3-pass overwrite
pub fn dod_overwrite(path: &Path, file_size: usize) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(false)
        .open(path)
        .with_context(|| format!("Failed to open file for DoD overwrite: {}", path.display()))?;
    
    println!("  DoD 5220.22-M standard overwrite");
    
    // DoD standard: Pass 1 = all 0s, Pass 2 = all 1s, Pass 3 = random
    let patterns = [
        ("zeros", vec![0u8; 1024 * 1024]),
        ("ones", vec![0xFFu8; 1024 * 1024]),
    ];
    
    // Passes 1 and 2: Fixed patterns
    for (i, (name, pattern)) in patterns.iter().enumerate() {
        println!("    DoD pass {}: {}", i + 1, name);
        
        file.seek(std::io::SeekFrom::Start(0))
            .with_context(|| format!("Failed to seek to start on DoD pass {}", i + 1))?;
        
        let mut remaining = file_size;
        while remaining > 0 {
            let chunk_size = remaining.min(pattern.len());
            file.write_all(&pattern[..chunk_size])
                .with_context(|| format!("Failed to write {} data", name))?;
            remaining -= chunk_size;
        }
        
        file.sync_all()
            .with_context(|| format!("Failed to sync file on DoD pass {}", i + 1))?;
    }
    
    // Pass 3: Random data
    println!("    DoD pass 3: random");
    let mut rng = rand::thread_rng();
    
    file.seek(std::io::SeekFrom::Start(0))
        .with_context(|| "Failed to seek to start on DoD pass 3")?;
    
    let mut remaining = file_size;
    while remaining > 0 {
        let chunk_size = remaining.min(1024 * 1024);
        let mut random_data = vec![0u8; chunk_size];
        rng.fill_bytes(&mut random_data);
        
        file.write_all(&random_data)
            .with_context(|| "Failed to write random data on DoD pass 3")?;
        remaining -= chunk_size;
    }
    
    file.sync_all()
        .with_context(|| "Failed to sync file on DoD pass 3")?;
    
    Ok(())
}