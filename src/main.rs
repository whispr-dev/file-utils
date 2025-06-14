mod config;
mod crypto;
mod secure_delete;
mod file_utils;
mod ui;

use anyhow::Result;
use clap::{Arg, ArgAction, Command};
use std::path::Path;

use crate::crypto::{encrypt_file, decrypt_file, CryptoMode};
use crate::secure_delete::secure_delete;

fn main() -> Result<()> {
    let matches = Command::new("wofl_obs-defuscrypt")
        .about("Encrypt, decrypt, or securely delete files")
        .arg(Arg::new("secure")
            .short('s')
            .long("secure")
            .help("Securely delete a file directly (standalone)")
            .num_args(1)
            .value_name("PATH"))
        .subcommand(
            Command::new("encrypt")
                .about("Encrypt a file")
                .arg(Arg::new("source")
                    .help("Path to the source file to encrypt")
                    .required(true))
                .arg(Arg::new("output")
                    .short('o')
                    .long("output")
                    .help("Output file path (optional)")
                    .num_args(1)
                    .value_name("PATH"))
                .arg(Arg::new("key")
                    .short('k')
                    .long("key")
                    .help("Encryption key (optional, will prompt if not provided)")
                    .num_args(1)
                    .value_name("KEY"))
                .arg(Arg::new("mode")
                    .short('m')
                    .long("mode")
                    .help("Encryption mode: aes or quantum")
                    .num_args(1)
                    .value_name("MODE")
                    .default_value("aes"))
                .arg(Arg::new("secure")
                    .short('s')
                    .long("secure")
                    .help("Securely delete original after encryption")
                    .action(ArgAction::SetTrue))
        )
        .subcommand(
            Command::new("decrypt")
                .about("Decrypt a file")
                .arg(Arg::new("source")
                    .help("Path to the source file to decrypt")
                    .required(true))
                .arg(Arg::new("output")
                    .short('o')
                    .long("output")
                    .help("Output file path (optional)")
                    .num_args(1)
                    .value_name("PATH"))
                .arg(Arg::new("key")
                    .short('k')
                    .long("key")
                    .help("Decryption key (optional, will prompt if not provided)")
                    .num_args(1)
                    .value_name("KEY"))
                .arg(Arg::new("mode")
                    .short('m')
                    .long("mode")
                    .help("Decryption mode: aes or quantum")
                    .num_args(1)
                    .value_name("MODE")
                    .default_value("aes"))
                .arg(Arg::new("secure")
                    .short('s')
                    .long("secure")
                    .help("Securely delete original after decryption")
                    .action(ArgAction::SetTrue))
        )
        .get_matches();

    // CASE: Standalone secure delete mode
    if let Some(path) = matches.get_one::<String>("secure") {
        secure_delete(Path::new(path))?;
        println!("✅ Securely deleted: {}", path);
        return Ok(());
    }

    // CASE: Subcommands (encrypt, decrypt)
    match matches.subcommand() {
        Some(("encrypt", encrypt_matches)) => {
            let source_file = encrypt_matches.get_one::<String>("source").unwrap();
            let source_path = Path::new(source_file);
            
            // Determine output path
            let output_path = match encrypt_matches.get_one::<String>("output") {
                Some(output) => output.clone(),
                None => {
                    // Auto-generate output filename by adding .enc extension
                    format!("{}.enc", source_file)
                }
            };
            
            // Get encryption key
            let key = get_encryption_key(encrypt_matches.get_one::<String>("key"))?;
            
            // Get encryption mode
            let mode_str = encrypt_matches.get_one::<String>("mode").unwrap();
            let mode = CryptoMode::from_str(mode_str);
            
            // Perform encryption
            encrypt_file(source_path, Path::new(&output_path), &key, mode)?;
            
            // Securely delete original if requested
            if encrypt_matches.get_flag("secure") {
                secure_delete(source_path)?;
                println!("✅ Original file securely deleted");
            }
        }
        Some(("decrypt", decrypt_matches)) => {
            let source_file = decrypt_matches.get_one::<String>("source").unwrap();
            let source_path = Path::new(source_file);
            
            // Determine output path
            let output_path = match decrypt_matches.get_one::<String>("output") {
                Some(output) => output.clone(),
                None => {
                    // Auto-generate output filename by removing .enc extension
                    if source_file.ends_with(".enc") {
                        source_file[..source_file.len() - 4].to_string()
                    } else {
                        format!("{}.decrypted", source_file)
                    }
                }
            };
            
            // Get decryption key
            let key = get_encryption_key(decrypt_matches.get_one::<String>("key"))?;
            
            // Get decryption mode
            let mode_str = decrypt_matches.get_one::<String>("mode").unwrap();
            let mode = CryptoMode::from_str(mode_str);
            
            // Perform decryption
            decrypt_file(source_path, Path::new(&output_path), &key, mode)?;
            
            // Securely delete original if requested
            if decrypt_matches.get_flag("secure") {
                secure_delete(source_path)?;
                println!("✅ Original file securely deleted");
            }
        }
        _ => {
            println!("Usage:");
            println!("  Encrypt: wofl_obs-defuscrypt.exe encrypt <path> [-o output] [-k key] [-m mode] [-s]");
            println!("  Decrypt: wofl_obs-defuscrypt.exe decrypt <path> [-o output] [-k key] [-m mode] [-s]");
            println!("  Shred:   wofl_obs-defuscrypt.exe -s <path>");
            println!("");
            println!("Modes: aes (default), quantum");
            println!("If no key is provided, you'll be prompted to enter one.");
        }
    }

    Ok(())
}

/// Get encryption key from command line argument or prompt user
fn get_encryption_key(key_arg: Option<&String>) -> Result<Vec<u8>> {
    match key_arg {
        Some(key_str) => {
            // Use provided key, convert to bytes
            Ok(key_str.as_bytes().to_vec())
        }
        None => {
            // Prompt user for key
            use std::io::{self, Write};
            
            print!("Enter encryption key: ");
            io::stdout().flush()?;
            
            let mut key = String::new();
            io::stdin().read_line(&mut key)?;
            
            // Remove trailing newline
            let key = key.trim();
            
            if key.is_empty() {
                return Err(anyhow::anyhow!("Empty key not allowed"));
            }
            
            Ok(key.as_bytes().to_vec())
        }
    }
}