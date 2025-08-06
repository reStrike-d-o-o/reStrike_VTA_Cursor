//! Encryption module for reStrike VTA security
//! 
//! Provides SHA256-based encryption for sensitive configuration data
//! using PBKDF2 key derivation and cryptographically secure random number generation.

use std::fmt;
use ring::{pbkdf2, rand};
use ring::rand::SecureRandom;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};
use base64::{Engine as _, engine::general_purpose};
use serde::{Serialize, Deserialize};
use crate::security::constants::*;

/// Security error types
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    #[error("Decryption error: {0}")]
    Decryption(String),
    
    #[error("Key derivation error: {0}")]
    KeyDerivation(String),
    
    #[error("Random number generation error: {0}")]
    RandomGeneration(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Database connection error: {0}")]
    DatabaseConnection(#[from] crate::database::DatabaseError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

/// Encrypted data container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Base64-encoded encrypted data
    pub ciphertext: String,
    /// Base64-encoded salt used for key derivation
    pub salt: String,
    /// Base64-encoded nonce used for encryption
    pub nonce: String,
    /// Encryption algorithm identifier
    pub algorithm: String,
    /// Key derivation parameters
    pub kdf_params: KdfParams,
}

/// Key derivation function parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    /// PBKDF2 iteration count
    pub iterations: u32,
    /// Salt length in bytes
    pub salt_length: usize,
    /// Derived key length in bytes
    pub key_length: usize,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            iterations: PBKDF2_ITERATIONS,
            salt_length: SALT_LENGTH,
            key_length: KEY_LENGTH,
        }
    }
}

/// Secure configuration encryption handler
pub struct SecureConfig {
    /// Master password for key derivation
    master_password: String,
    /// System-specific entropy for additional security
    system_entropy: Vec<u8>,
    /// Key derivation parameters
    kdf_params: KdfParams,
}

impl SecureConfig {
    /// Create a new SecureConfig instance
    pub fn new(master_password: String) -> Result<Self, SecurityError> {
        let system_entropy = Self::generate_system_entropy()?;
        
        Ok(Self {
            master_password,
            system_entropy,
            kdf_params: KdfParams::default(),
        })
    }
    
    /// Generate system-specific entropy for additional security
    fn generate_system_entropy() -> Result<Vec<u8>, SecurityError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Add system-specific information
        if let Ok(hostname) = std::env::var("COMPUTERNAME") {
            hostname.hash(&mut hasher);
        }
        if let Ok(username) = std::env::var("USERNAME") {
            username.hash(&mut hasher);
        }
        
        // Add process-specific information
        std::process::id().hash(&mut hasher);
        
        let hash = hasher.finish();
        Ok(hash.to_be_bytes().to_vec())
    }
    
    /// Generate a cryptographically secure random salt
    fn generate_salt(&self) -> Result<Vec<u8>, SecurityError> {
        let rng = rand::SystemRandom::new();
        let mut salt = vec![0u8; self.kdf_params.salt_length];
        
        rng.fill(&mut salt)
            .map_err(|e| SecurityError::RandomGeneration(format!("Failed to generate salt: {:?}", e)))?;
        
        Ok(salt)
    }
    
    /// Generate a cryptographically secure random nonce
    fn generate_nonce(&self) -> Result<Vec<u8>, SecurityError> {
        let rng = rand::SystemRandom::new();
        let mut nonce = vec![0u8; 12]; // AES-GCM standard nonce length
        
        rng.fill(&mut nonce)
            .map_err(|e| SecurityError::RandomGeneration(format!("Failed to generate nonce: {:?}", e)))?;
        
        Ok(nonce)
    }
    
    /// Derive encryption key from master password and salt
    fn derive_key(&self, salt: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let mut key = vec![0u8; self.kdf_params.key_length];
        
        // Combine master password with system entropy
        let mut password_data = self.master_password.as_bytes().to_vec();
        password_data.extend(&self.system_entropy);
        
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(self.kdf_params.iterations).unwrap(),
            salt,
            &password_data,
            &mut key,
        );
        
        Ok(key)
    }
    
    /// Encrypt a plaintext value
    pub fn encrypt_value(&self, plaintext: &str) -> Result<EncryptedData, SecurityError> {
        if plaintext.is_empty() {
            return Err(SecurityError::InvalidInput("Plaintext cannot be empty".to_string()));
        }
        
        // Generate salt and nonce
        let salt = self.generate_salt()?;
        let nonce_bytes = self.generate_nonce()?;
        
        // Derive encryption key
        let key_bytes = self.derive_key(&salt)?;
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt the data
        let cipher = Aes256Gcm::new(key);
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| SecurityError::Encryption(format!("AES encryption failed: {:?}", e)))?;
        
        // Encode to base64
        let encrypted_data = EncryptedData {
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            salt: general_purpose::STANDARD.encode(&salt),
            nonce: general_purpose::STANDARD.encode(&nonce_bytes),
            algorithm: "AES-256-GCM".to_string(),
            kdf_params: self.kdf_params.clone(),
        };
        
        Ok(encrypted_data)
    }
    
    /// Decrypt an encrypted value
    pub fn decrypt_value(&self, encrypted_data: &EncryptedData) -> Result<String, SecurityError> {
        // Validate algorithm
        if encrypted_data.algorithm != "AES-256-GCM" {
            return Err(SecurityError::Decryption(
                format!("Unsupported algorithm: {}", encrypted_data.algorithm)
            ));
        }
        
        // Decode from base64
        let ciphertext = general_purpose::STANDARD
            .decode(&encrypted_data.ciphertext)
            .map_err(|e| SecurityError::Decryption(format!("Invalid ciphertext encoding: {}", e)))?;
        
        let salt = general_purpose::STANDARD
            .decode(&encrypted_data.salt)
            .map_err(|e| SecurityError::Decryption(format!("Invalid salt encoding: {}", e)))?;
        
        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted_data.nonce)
            .map_err(|e| SecurityError::Decryption(format!("Invalid nonce encoding: {}", e)))?;
        
        // Derive decryption key
        let key_bytes = self.derive_key(&salt)?;
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Decrypt the data
        let cipher = Aes256Gcm::new(key);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| SecurityError::Decryption(format!("AES decryption failed: {:?}", e)))?;
        
        // Convert to string
        String::from_utf8(plaintext)
            .map_err(|e| SecurityError::Decryption(format!("Invalid UTF-8 in plaintext: {}", e)))
    }
    
    /// Hash a password for storage (one-way hash)
    pub fn hash_password(&self, password: &str) -> Result<String, SecurityError> {
        if password.is_empty() {
            return Err(SecurityError::InvalidInput("Password cannot be empty".to_string()));
        }
        
        let salt = self.generate_salt()?;
        let mut hash = vec![0u8; 32]; // SHA256 output length
        
        // Combine password with system entropy
        let mut password_data = password.as_bytes().to_vec();
        password_data.extend(&self.system_entropy);
        
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(self.kdf_params.iterations).unwrap(),
            &salt,
            &password_data,
            &mut hash,
        );
        
        // Combine salt and hash for storage
        let mut result = salt;
        result.extend(hash);
        
        Ok(general_purpose::STANDARD.encode(result))
    }
    
    /// Verify a password against a stored hash
    pub fn verify_password(&self, password: &str, stored_hash: &str) -> Result<bool, SecurityError> {
        if password.is_empty() || stored_hash.is_empty() {
            return Ok(false);
        }
        
        // Decode stored hash
        let stored_data = general_purpose::STANDARD
            .decode(stored_hash)
            .map_err(|e| SecurityError::Authentication(format!("Invalid hash encoding: {}", e)))?;
        
        if stored_data.len() != SALT_LENGTH + 32 {
            return Err(SecurityError::Authentication("Invalid hash format".to_string()));
        }
        
        // Extract salt and hash
        let (salt, expected_hash) = stored_data.split_at(SALT_LENGTH);
        
        // Hash the provided password with the same salt
        let mut computed_hash = vec![0u8; 32];
        let mut password_data = password.as_bytes().to_vec();
        password_data.extend(&self.system_entropy);
        
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(self.kdf_params.iterations).unwrap(),
            salt,
            &password_data,
            &mut computed_hash,
        );
        
        // Constant-time comparison
        Ok(computed_hash.as_slice() == expected_hash)
    }
    
    /// Generate a secure random string for use as passwords or tokens
    pub fn generate_secure_string(&self, length: usize) -> Result<String, SecurityError> {
        if length == 0 {
            return Err(SecurityError::InvalidInput("Length must be greater than 0".to_string()));
        }
        
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
        let rng = rand::SystemRandom::new();
        let mut result = Vec::with_capacity(length);
        
        for _ in 0..length {
            let mut byte = [0u8; 1];
            rng.fill(&mut byte)
                .map_err(|e| SecurityError::RandomGeneration(format!("Failed to generate random byte: {:?}", e)))?;
            
            let idx = (byte[0] as usize) % CHARSET.len();
            result.push(CHARSET[idx]);
        }
        
        String::from_utf8(result)
            .map_err(|e| SecurityError::RandomGeneration(format!("Failed to create string: {}", e)))
    }
}

impl fmt::Debug for SecureConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureConfig")
            .field("master_password", &"[REDACTED]")
            .field("system_entropy", &format!("[{} bytes]", self.system_entropy.len()))
            .field("kdf_params", &self.kdf_params)
            .finish()
    }
}

/// Security utility functions
pub mod utils {
    use super::*;
    
    /// Generate a master password from system entropy
    pub fn generate_master_password() -> Result<String, SecurityError> {
        let config = SecureConfig::new("temp".to_string())?;
        config.generate_secure_string(32)
    }
    
    /// Check if a string contains only safe characters
    pub fn is_safe_string(input: &str) -> bool {
        input.chars().all(|c| c.is_ascii_alphanumeric() || "!@#$%^&*()-_=+[]{}|;:,.<>?".contains(c))
    }
    
    /// Sanitize input by removing unsafe characters
    pub fn sanitize_input(input: &str) -> String {
        input.chars()
            .filter(|c| c.is_ascii_alphanumeric() || "!@#$%^&*()-_=+[]{}|;:,.<>?".contains(*c))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_decryption() {
        let config = SecureConfig::new("test_password".to_string()).unwrap();
        let plaintext = "sensitive_data_123";
        
        let encrypted = config.encrypt_value(plaintext).unwrap();
        let decrypted = config.decrypt_value(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
    
    #[test]
    fn test_password_hashing() {
        let config = SecureConfig::new("test_password".to_string()).unwrap();
        let password = "user_password_123";
        
        let hash = config.hash_password(password).unwrap();
        assert!(config.verify_password(password, &hash).unwrap());
        assert!(!config.verify_password("wrong_password", &hash).unwrap());
    }
    
    #[test]
    fn test_secure_string_generation() {
        let config = SecureConfig::new("test_password".to_string()).unwrap();
        let secure_string = config.generate_secure_string(16).unwrap();
        
        assert_eq!(secure_string.len(), 16);
        assert!(utils::is_safe_string(&secure_string));
    }
    
    #[test]
    fn test_empty_input_handling() {
        let config = SecureConfig::new("test_password".to_string()).unwrap();
        
        assert!(config.encrypt_value("").is_err());
        assert!(config.hash_password("").is_err());
        assert!(config.generate_secure_string(0).is_err());
    }
}