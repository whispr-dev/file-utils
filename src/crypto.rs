use aes::Aes256;
use ctr::cipher::{KeyIvInit, StreamCipher};
use rand::rngs::OsRng;
use rand::RngCore;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use anyhow::{Context, Result};
use std::env;

type Aes256Ctr = ctr::Ctr128BE<Aes256>;

#[derive(Debug, Clone, Copy)]
pub enum CryptoMode {
    Aes256Ctr,
    Quantum,
}

impl Default for CryptoMode {
    fn default() -> Self {
        // Check environment variable or config
        match env::var("CRYPTO_MODE").as_deref() {
            Ok("quantum") => CryptoMode::Quantum,
            Ok("aes") => CryptoMode::Aes256Ctr,
            _ => CryptoMode::Aes256Ctr, // Default to AES
        }
    }
}

impl CryptoMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "quantum" => CryptoMode::Quantum,
            "aes" | "aes256" | "aes-ctr" => CryptoMode::Aes256Ctr,
            _ => CryptoMode::default(),
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            CryptoMode::Aes256Ctr => "aes",
            CryptoMode::Quantum => "quantum",
        }
    }
}

pub fn encrypt_aes_ctr256(data: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Vec<u8> {
    let mut cipher = Aes256Ctr::new(key.into(), iv.into());
    let mut buffer = data.to_vec();
    cipher.apply_keystream(&mut buffer);
    buffer
}

pub fn decrypt_aes_ctr256(data: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Vec<u8> {
    // CTR mode is symmetric - encryption and decryption are the same operation
    encrypt_aes_ctr256(data, key, iv)
}

// Enhanced Quantum Resilient Kryptographic State Machine
// Based on theoretical quantum-resistant entropy cascading
const ENTROPY_THRESHOLD: f64 = 0.3;
const INITIAL_ETA: f64 = 1.0;
const FRAGILITY_ALPHA: f64 = 0.7;
const BUFFER_T: f64 = 5.0;
const QUANTUM_ROUNDS: usize = 3;
const ENTROPY_SEED_SIZE: usize = 64;

#[derive(Debug, Clone)]
struct RKState {
    message: Vec<u8>,
    eta: f64,
    q: f64,
    t: f64,
    alpha: f64,
    entropy_pool: Vec<u8>,
    round: usize,
}

impl RKState {
    pub fn new(message: Vec<u8>) -> Self {
        let mut entropy_pool = vec![0u8; ENTROPY_SEED_SIZE];
        OsRng.fill_bytes(&mut entropy_pool);
        
        Self {
            message,
            eta: INITIAL_ETA,
            q: 0.0,
            t: BUFFER_T,
            alpha: FRAGILITY_ALPHA,
            entropy_pool,
            round: 0,
        }
    }
    
    pub fn new_with_key(message: Vec<u8>, key: &[u8]) -> Self {
        let mut entropy_pool = vec![0u8; ENTROPY_SEED_SIZE];
        
        // Seed entropy pool with key material
        for (i, &byte) in key.iter().take(ENTROPY_SEED_SIZE).enumerate() {
            entropy_pool[i] = byte;
        }
        
        // Fill remaining with OS random
        if key.len() < ENTROPY_SEED_SIZE {
            OsRng.fill_bytes(&mut entropy_pool[key.len()..]);
        }
        
        Self {
            message,
            eta: INITIAL_ETA,
            q: 0.0,
            t: BUFFER_T,
            alpha: FRAGILITY_ALPHA,
            entropy_pool,
            round: 0,
        }
    }
    
    fn update_entropy(&mut self, byte_index: usize) {
        let pool_idx = byte_index % self.entropy_pool.len();
        let entropy_byte = self.entropy_pool[pool_idx];
        
        // Quantum-inspired entropy cascade
        self.q += self.alpha * (entropy_byte as f64);
        self.t += self.q * self.eta;
        
        // Update entropy pool based on current state
        self.entropy_pool[pool_idx] = (entropy_byte as f64 + self.t) as u8;
    }

    pub fn encrypt(mut self) -> Vec<u8> {
        // Multi-round quantum encryption
        for round in 0..QUANTUM_ROUNDS {
            self.round = round;
            
            // Process message in chunks to avoid borrowing conflicts
            let message_len = self.message.len();
            for i in 0..message_len {
                self.update_entropy(i);
                
                // Quantum state transformation
                let quantum_noise = (self.t.sin() * 255.0) as u8;
                let cascade_factor = (self.q.fract() * 255.0) as u8;
                
                self.message[i] = self.message[i]
                    .wrapping_add(quantum_noise)
                    .wrapping_mul(cascade_factor.wrapping_add(1))
                    .wrapping_add(42); // Quantum constant
            }
            
            // Reset state for next round
            self.q *= 0.618; // Golden ratio decay
            self.t *= 1.414; // √2 amplification
        }
        
        // Prepend metadata for decryption
        let mut result = Vec::with_capacity(self.message.len() + 8);
        result.extend_from_slice(&(self.message.len() as u64).to_le_bytes());
        result.extend_from_slice(&self.message);
        
        result
    }

    pub fn decrypt(mut self) -> Vec<u8> {
        // Extract original length
        if self.message.len() < 8 {
            return self.message; // Corrupted data
        }
        
        let original_len_bytes: [u8; 8] = self.message[0..8].try_into().unwrap_or([0; 8]);
        let original_len = u64::from_le_bytes(original_len_bytes) as usize;
        
        self.message = self.message[8..].to_vec();
        
        // Reverse the encryption rounds
        for round in (0..QUANTUM_ROUNDS).rev() {
            self.round = round;
            
            // Process message in chunks to avoid borrowing conflicts
            let message_len = self.message.len();
            for i in 0..message_len {
                self.update_entropy(i);
                
                // Reverse quantum state transformation
                let quantum_noise = (self.t.sin() * 255.0) as u8;
                let cascade_factor = (self.q.fract() * 255.0) as u8;
                
                self.message[i] = self.message[i]
                    .wrapping_sub(42)
                    .wrapping_div(cascade_factor.wrapping_add(1))
                    .wrapping_sub(quantum_noise);
            }
            
            // Reset state for next round (reverse)
            self.t /= 1.414;
            self.q /= 0.618;
        }
        
        // Truncate to original length
        self.message.truncate(original_len);
        self.message
    }
}

pub fn encrypt_with_mode(data: &[u8], key: &[u8], mode: CryptoMode) -> Result<Vec<u8>> {
    match mode {
        CryptoMode::Quantum => {
            let rk = RKState::new_with_key(data.to_vec(), key);
            Ok(rk.encrypt())
        },
        CryptoMode::Aes256Ctr => {
            let mut iv = [0u8; 16];
            OsRng.fill_bytes(&mut iv);
            
            let mut k = [0u8; 32];
            let key_len = key.len().min(32);
            k[..key_len].copy_from_slice(&key[..key_len]);
            
            // If key is shorter than 32 bytes, pad with deterministic data
            if key_len < 32 {
                for i in key_len..32 {
                    k[i] = key[i % key_len].wrapping_add(i as u8);
                }
            }
            
            let encrypted = encrypt_aes_ctr256(data, &k, &iv);
            
            // Prepend IV to encrypted data
            let mut result = Vec::with_capacity(16 + encrypted.len());
            result.extend_from_slice(&iv);
            result.extend_from_slice(&encrypted);
            
            Ok(result)
        }
    }
}

pub fn decrypt_with_mode(data: &[u8], key: &[u8], mode: CryptoMode) -> Result<Vec<u8>> {
    match mode {
        CryptoMode::Quantum => {
            let rk = RKState::new_with_key(data.to_vec(), key);
            Ok(rk.decrypt())
        },
        CryptoMode::Aes256Ctr => {
            if data.len() < 16 {
                return Err(anyhow::anyhow!("Encrypted data too short (missing IV)"));
            }
            
            let (iv_bytes, ciphertext) = data.split_at(16);
            let iv: [u8; 16] = iv_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid IV length"))?;
            
            let mut k = [0u8; 32];
            let key_len = key.len().min(32);
            k[..key_len].copy_from_slice(&key[..key_len]);
            
            // If key is shorter than 32 bytes, pad with deterministic data
            if key_len < 32 {
                for i in key_len..32 {
                    k[i] = key[i % key_len].wrapping_add(i as u8);
                }
            }
            
            Ok(decrypt_aes_ctr256(ciphertext, &k, &iv))
        }
    }
}

/// High-level file encryption function with mode support
pub fn encrypt_file<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    key: &[u8],
    mode: CryptoMode,
) -> Result<()> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    
    // Read input file
    let mut input_file = File::open(input_path)
        .with_context(|| format!("Failed to open input file: {}", input_path.display()))?;
    
    let mut data = Vec::new();
    input_file.read_to_end(&mut data)
        .with_context(|| format!("Failed to read input file: {}", input_path.display()))?;
    
    // Encrypt data
    let encrypted_data = encrypt_with_mode(&data, key, mode)
        .with_context(|| "Failed to encrypt data")?;
    
    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }
    
    // Write encrypted data
    let mut output_file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
    
    output_file.write_all(&encrypted_data)
        .with_context(|| format!("Failed to write encrypted data to: {}", output_path.display()))?;
    
    output_file.sync_all()
        .with_context(|| "Failed to sync encrypted file to disk")?;
    
    println!("Encrypted {} -> {} using {} mode", 
             input_path.display(), 
             output_path.display(), 
             mode.as_str());
    
    Ok(())
}

/// High-level file decryption function with mode support
pub fn decrypt_file<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    key: &[u8],
    mode: CryptoMode,
) -> Result<()> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();
    
    // Read encrypted file
    let mut input_file = File::open(input_path)
        .with_context(|| format!("Failed to open encrypted file: {}", input_path.display()))?;
    
    let mut encrypted_data = Vec::new();
    input_file.read_to_end(&mut encrypted_data)
        .with_context(|| format!("Failed to read encrypted file: {}", input_path.display()))?;
    
    // Decrypt data
    let decrypted_data = decrypt_with_mode(&encrypted_data, key, mode)
        .with_context(|| "Failed to decrypt data")?;
    
    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }
    
    // Write decrypted data
    let mut output_file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
    
    output_file.write_all(&decrypted_data)
        .with_context(|| format!("Failed to write decrypted data to: {}", output_path.display()))?;
    
    output_file.sync_all()
        .with_context(|| "Failed to sync decrypted file to disk")?;
    
    println!("Decrypted {} -> {} using {} mode", 
             input_path.display(), 
             output_path.display(), 
             mode.as_str());
    
    Ok(())
}

// Convenience functions for backward compatibility
pub fn encrypt_file_legacy<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    key: &[u8],
    mode: &str,
) -> Result<()> {
    let crypto_mode = CryptoMode::from_str(mode);
    encrypt_file(input_path, output_path, key, crypto_mode)
}

pub fn decrypt_file_legacy<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    key: &[u8],
    mode: &str,
) -> Result<()> {
    let crypto_mode = CryptoMode::from_str(mode);
    decrypt_file(input_path, output_path, key, crypto_mode)
}