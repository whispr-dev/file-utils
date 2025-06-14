use anyhow::{Context, Result};
use console::style;
use std::{
    fs,
    path::Path,
};

/// Ensure a directory exists, creating it if necessary
pub fn ensure_directory(dir_path: &Path) -> Result<()> {
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)
            .with_context(|| format!("Failed to create directory: {}", dir_path.display()))?;
        
        println!(
            "{} Created directory: {}",
            style("[INFO]").blue().bold(),
            dir_path.display()
        );
    }
    
    Ok(())
}

/// Display a file processing summary
pub fn display_summary(
    operation: &str,
    total_files: usize,
    processed_files: usize,
    skipped_files: usize,
    failed_files: usize,
) {
    println!("\n{}", style(format!("{} Summary", operation)).bold().underlined());
    println!("Total files: {}", total_files);
    println!("{} {}", style("Successfully processed:").green(), processed_files);
    println!("{} {}", style("Skipped:").yellow(), skipped_files);
    println!("{} {}", style("Failed:").red(), failed_files);
}

/// Display progress information during processing
pub fn display_progress(
    current: usize,
    total: usize,
    file_path: &Path,
    operation: &str,
) {
    println!(
        "{} [{}/{}] {} {}",
        style("[PROGRESS]").blue().bold(),
        current,
        total,
        operation,
        file_path.display()
    );
}

/// Display application header
pub fn display_header() {
    let version = env!("CARGO_PKG_VERSION");
    
    println!("{}", style("════════════════════════════════════════").cyan());
    println!(
        "{} {} {}",
        style("SecureCrypt").cyan().bold(),
        style("v").cyan(),
        style(version).cyan().bold()
    );
    println!("{}", style("Secure File Encryption & Shredding Tool").cyan().italic());
    println!("{}", style("════════════════════════════════════════").cyan());
    println!();
}

/// Display error message
pub fn display_error(message: &str) {
    eprintln!("{} {}", style("[ERROR]").red().bold(), message);
}

/// Display warning message
pub fn display_warning(message: &str) {
    println!("{} {}", style("[WARNING]").yellow().bold(), message);
}

/// Display info message
pub fn display_info(message: &str) {
    println!("{} {}", style("[INFO]").blue().bold(), message);
}

/// Display success message
pub fn display_success(message: &str) {
    println!("{} {}", style("[SUCCESS]").green().bold(), message);
}