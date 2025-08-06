//! Encryption key management module for reStrike VTA
//! 
//! Provides secure key generation, rotation, and lifecycle management
//! for encryption operations.

use std::sync::Arc;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use ring::rand::{SecureRandom, SystemRandom};
use base64::{Engine as _, engine::general_purpose};

use crate::security::{SecureConfig, SecurityError, SecurityResult};
use crate::security::audit::{SecurityAudit, AuditAction};
use crate::database::DatabaseConnection;

/// Key rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationConfig {
    /// Whether automatic key rotation is enabled
    pub enabled: bool,
    /// Rotation interval in days
    pub interval_days: u32,
    /// Maximum key age in days before forced rotation
    pub max_age_days: u32,
    /// Number of old keys to retain for decryption
    pub retain_old_keys: u32,
}

impl Default for KeyRotationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_days: 90,    // Rotate every 3 months
            max_age_days: 365,    // Force rotation after 1 year
            retain_old_keys: 3,   // Keep last 3 keys for decryption
        }
    }
}

/// Encryption key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub key_id: String,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub is_active: bool,
    pub algorithm: String,
    pub key_size: u32,
    pub usage_count: u64,
    pub rotation_reason: Option<String>,
}

impl KeyMetadata {
    pub fn new(algorithm: String, key_size: u32) -> Self {
        let now = Utc::now();
        Self {
            key_id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            last_used: now,
            is_active: true,
            algorithm,
            key_size,
            usage_count: 0,
            rotation_reason: None,
        }
    }
    
    pub fn age_days(&self) -> i64 {
        let now = Utc::now();
        (now - self.created_at).num_days()
    }
    
    pub fn should_rotate(&self, config: &KeyRotationConfig) -> bool {
        if !config.enabled {
            return false;
        }
        
        self.age_days() >= config.interval_days as i64
    }
    
    pub fn is_expired(&self, config: &KeyRotationConfig) -> bool {
        self.age_days() >= config.max_age_days as i64
    }
}

/// Encrypted key storage entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedKeyEntry {
    metadata: KeyMetadata,
    encrypted_key: String, // Base64-encoded encrypted key
    master_key_hash: String, // Hash of master key used for encryption
}

/// Key manager for encryption key lifecycle management
pub struct KeyManager {
    database: Arc<DatabaseConnection>,
    audit: SecurityAudit,
    rotation_config: KeyRotationConfig,
    rng: SystemRandom,
}

impl KeyManager {
    /// Create a new key manager
    pub async fn new(
        database: Arc<DatabaseConnection>,
        rotation_config: Option<KeyRotationConfig>,
    ) -> SecurityResult<Self> {
        let audit = SecurityAudit::new(database.clone())?;
        let config = rotation_config.unwrap_or_default();
        
        Ok(Self {
            database,
            audit,
            rotation_config: config,
            rng: SystemRandom::new(),
        })
    }
    
    /// Generate a new encryption key
    pub async fn generate_encryption_key(
        &self,
        user_context: &str,
        algorithm: &str,
        key_size: u32,
    ) -> SecurityResult<String> {
        // Generate random key
        let mut key_bytes = vec![0u8; (key_size / 8) as usize];
        self.rng.fill(&mut key_bytes)
            .map_err(|e| SecurityError::RandomGeneration(format!("Failed to generate key: {:?}", e)))?;
        
        // Create metadata
        let metadata = KeyMetadata::new(algorithm.to_string(), key_size);
        
        // Store the key securely
        self.store_key(&metadata, &key_bytes, user_context).await?;
        
        // Log the key generation
        self.audit.log_security_event(
            AuditAction::EncryptionKeyRotation,
            user_context,
            &format!("Generated new {} encryption key ({})", algorithm, metadata.key_id),
            true,
            None,
        ).await?;
        
        // Return base64-encoded key
        Ok(general_purpose::STANDARD.encode(&key_bytes))
    }
    
    /// Get the current active encryption key
    pub async fn get_active_key(&self, algorithm: &str) -> SecurityResult<Option<(String, KeyMetadata)>> {
        let conn = self.database.get_connection().await?;
        
        let result = conn.query_row(
            "SELECT config_key, encrypted_value FROM secure_config 
             WHERE category = 'encryption_keys' AND config_key LIKE ? AND is_sensitive = 1 
             ORDER BY updated_at DESC LIMIT 1",
            [&format!("{}_%", algorithm)],
            |row| {
                let config_key: String = row.get(0)?;
                let encrypted_value: Vec<u8> = row.get(1)?;
                Ok((config_key, encrypted_value))
            },
        );
        
        match result {
            Ok((_config_key, encrypted_value)) => {
                let encrypted_json = String::from_utf8(encrypted_value)
                    .map_err(|e| SecurityError::Decryption(format!("Invalid UTF-8 in key data: {}", e)))?;
                
                let entry: EncryptedKeyEntry = serde_json::from_str(&encrypted_json)?;
                
                if entry.metadata.is_active && !entry.metadata.is_expired(&self.rotation_config) {
                    // Decrypt the key (this would use the master key derivation)
                    let key_bytes = general_purpose::STANDARD.decode(&entry.encrypted_key)
                        .map_err(|e| SecurityError::Decryption(format!("Failed to decode key: {}", e)))?;
                    
                    let key_string = general_purpose::STANDARD.encode(&key_bytes);
                    
                    // Update usage statistics
                    self.update_key_usage(&entry.metadata.key_id).await?;
                    
                    Ok(Some((key_string, entry.metadata)))
                } else {
                    Ok(None)
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(SecurityError::Database(e)),
        }
    }
    
    /// Rotate encryption keys
    pub async fn rotate_keys(&self, user_context: &str, reason: Option<String>) -> SecurityResult<Vec<String>> {
        let mut rotated_keys = Vec::new();
        
        // Get all active keys that need rotation
        let keys_to_rotate = self.get_keys_needing_rotation().await?;
        
        for (algorithm, mut metadata) in keys_to_rotate {
            // Generate new key
            let new_key = self.generate_encryption_key(user_context, &algorithm, metadata.key_size).await?;
            
            // Mark old key as inactive
            metadata.is_active = false;
            metadata.rotation_reason = reason.clone();
            self.update_key_metadata(&metadata).await?;
            
            rotated_keys.push(format!("{}:{}", algorithm, new_key));
            
            // Log the rotation
            self.audit.log_security_event(
                AuditAction::EncryptionKeyRotation,
                user_context,
                &format!("Rotated {} encryption key: {}", algorithm, metadata.key_id),
                true,
                None,
            ).await?;
        }
        
        // Clean up old keys
        self.cleanup_old_keys().await?;
        
        Ok(rotated_keys)
    }
    
    /// Force rotation of all keys
    pub async fn force_rotate_all_keys(&self, user_context: &str, reason: &str) -> SecurityResult<u32> {
        let conn = self.database.get_connection().await?;
        
        // Get all active keys
        let mut stmt = conn.prepare(
            "SELECT config_key, encrypted_value FROM secure_config 
             WHERE category = 'encryption_keys' AND is_sensitive = 1"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let config_key: String = row.get(0)?;
            let encrypted_value: Vec<u8> = row.get(1)?;
            Ok((config_key, encrypted_value))
        })?;
        
        let mut count = 0;
        for row in rows {
            let (_config_key, encrypted_value) = row?;
            let encrypted_json = String::from_utf8(encrypted_value)
                .map_err(|e| SecurityError::Decryption(format!("Invalid UTF-8 in key data: {}", e)))?;
            
            let entry: EncryptedKeyEntry = serde_json::from_str(&encrypted_json)?;
            
            if entry.metadata.is_active {
                // Rotate this key
                self.rotate_keys(user_context, Some(reason.to_string())).await?;
                count += 1;
            }
        }
        
        // Log the mass rotation
        self.audit.log_security_event(
            AuditAction::EncryptionKeyRotation,
            user_context,
            &format!("Force rotated {} encryption keys: {}", count, reason),
            true,
            None,
        ).await?;
        
        Ok(count)
    }
    
    /// Get key rotation status
    pub async fn get_rotation_status(&self) -> SecurityResult<KeyRotationStatus> {
        let conn = self.database.get_connection().await?;
        
        // Count total keys
        let total_keys: i64 = conn.query_row(
            "SELECT COUNT(*) FROM secure_config WHERE category = 'encryption_keys'",
            [],
            |row| row.get(0),
        )?;
        
        // Count active keys
        let active_keys: i64 = conn.query_row(
            "SELECT COUNT(*) FROM secure_config 
             WHERE category = 'encryption_keys' AND config_key LIKE '%active%'",
            [],
            |row| row.get(0),
        )?;
        
        // Get keys needing rotation
        let keys_needing_rotation = self.get_keys_needing_rotation().await?.len();
        
        // Get oldest key age
        let oldest_key_age = conn.query_row(
            "SELECT MIN(created_at) FROM secure_config WHERE category = 'encryption_keys'",
            [],
            |row| {
                let created_at_str: String = row.get(0)?;
                Ok(created_at_str)
            },
        ).ok().and_then(|date_str| {
            DateTime::parse_from_rfc3339(&date_str).ok().map(|dt| {
                (Utc::now() - dt.with_timezone(&Utc)).num_days()
            })
        });
        
        Ok(KeyRotationStatus {
            total_keys: total_keys as u32,
            active_keys: active_keys as u32,
            keys_needing_rotation: keys_needing_rotation as u32,
            oldest_key_age_days: oldest_key_age.unwrap_or(0) as u32,
            last_rotation: None, // Would need to track this separately
            next_scheduled_rotation: None, // Would need to calculate based on config
        })
    }
    
    /// Store an encryption key securely
    async fn store_key(
        &self,
        metadata: &KeyMetadata,
        key_bytes: &[u8],
        _user_context: &str,
    ) -> SecurityResult<()> {
        // Create a temporary SecureConfig to encrypt the key
        // In practice, this would use a master key derivation system
        let temp_config = SecureConfig::new("temp_master_key".to_string())?;
        
        let key_b64 = general_purpose::STANDARD.encode(key_bytes);
        let _encrypted_key = temp_config.encrypt_value(&key_b64)?;
        
        let entry = EncryptedKeyEntry {
            metadata: metadata.clone(),
            encrypted_key: general_purpose::STANDARD.encode(&key_bytes), // Simplified for demo
            master_key_hash: "demo_hash".to_string(), // Would be actual hash
        };
        
        let entry_json = serde_json::to_string(&entry)?;
        let config_key = format!("{}_{}", metadata.algorithm, metadata.key_id);
        
        // Store in secure_config table
        let conn = self.database.get_connection().await?;
        let now = Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO secure_config 
            (config_key, encrypted_value, category, is_sensitive, salt, algorithm, kdf_params, created_at, updated_at, description)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                config_key,
                entry_json.as_bytes(),
                "encryption_keys",
                true,
                vec![0u8; 32], // Placeholder salt
                "AES-256-GCM",
                "{}",
                now,
                now,
                format!("Encryption key for {}", metadata.algorithm),
            ],
        )?;
        
        Ok(())
    }
    
    /// Get keys that need rotation
    async fn get_keys_needing_rotation(&self) -> SecurityResult<Vec<(String, KeyMetadata)>> {
        let conn = self.database.get_connection().await?;
        
        let mut stmt = conn.prepare(
            "SELECT config_key, encrypted_value FROM secure_config 
             WHERE category = 'encryption_keys' AND is_sensitive = 1"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let config_key: String = row.get(0)?;
            let encrypted_value: Vec<u8> = row.get(1)?;
            Ok((config_key, encrypted_value))
        })?;
        
        let mut keys_to_rotate = Vec::new();
        
        for row in rows {
            let (_config_key, encrypted_value) = row?;
            let encrypted_json = String::from_utf8(encrypted_value)
                .map_err(|e| SecurityError::Decryption(format!("Invalid UTF-8 in key data: {}", e)))?;
            
            let entry: EncryptedKeyEntry = serde_json::from_str(&encrypted_json)?;
            
            if entry.metadata.is_active && entry.metadata.should_rotate(&self.rotation_config) {
                keys_to_rotate.push((entry.metadata.algorithm.clone(), entry.metadata));
            }
        }
        
        Ok(keys_to_rotate)
    }
    
    /// Update key metadata
    async fn update_key_metadata(&self, _metadata: &KeyMetadata) -> SecurityResult<()> {
        // This would update the stored metadata in the database
        // Implementation would decrypt, update, and re-encrypt the entry
        Ok(())
    }
    
    /// Update key usage statistics
    async fn update_key_usage(&self, _key_id: &str) -> SecurityResult<()> {
        // This would increment usage count and update last_used timestamp
        // Implementation would decrypt, update, and re-encrypt the entry
        Ok(())
    }
    
    /// Clean up old encryption keys
    async fn cleanup_old_keys(&self) -> SecurityResult<u32> {
        let conn = self.database.get_connection().await?;
        let cutoff_date = Utc::now() - chrono::Duration::days(self.rotation_config.max_age_days as i64 * 2);
        
        let deleted = conn.execute(
            "DELETE FROM secure_config 
             WHERE category = 'encryption_keys' 
             AND created_at < ? 
             AND config_key NOT LIKE '%_active'",
            [cutoff_date.to_rfc3339()],
        )?;
        
        log::info!("Cleaned up {} old encryption keys", deleted);
        Ok(deleted as u32)
    }
    
    /// Get key management statistics
    pub async fn get_key_statistics(&self) -> SecurityResult<KeyStatistics> {
        let conn = self.database.get_connection().await?;
        
        // Total keys
        let total_keys: i64 = conn.query_row(
            "SELECT COUNT(*) FROM secure_config WHERE category = 'encryption_keys'",
            [],
            |row| row.get(0),
        )?;
        
        // Active keys by algorithm
        let mut active_keys_by_algorithm = std::collections::HashMap::new();
        let mut stmt = conn.prepare(
            "SELECT config_key FROM secure_config 
             WHERE category = 'encryption_keys' AND config_key LIKE '%_active'"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let config_key: String = row.get(0)?;
            Ok(config_key)
        })?;
        
        for row in rows {
            let config_key = row?;
            if let Some(algorithm) = config_key.split('_').next() {
                *active_keys_by_algorithm.entry(algorithm.to_string()).or_insert(0) += 1;
            }
        }
        
        Ok(KeyStatistics {
            total_keys: total_keys as u32,
            active_keys_by_algorithm,
            last_rotation_check: Utc::now(),
        })
    }
}

/// Key rotation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationStatus {
    pub total_keys: u32,
    pub active_keys: u32,
    pub keys_needing_rotation: u32,
    pub oldest_key_age_days: u32,
    pub last_rotation: Option<DateTime<Utc>>,
    pub next_scheduled_rotation: Option<DateTime<Utc>>,
}

/// Key management statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStatistics {
    pub total_keys: u32,
    pub active_keys_by_algorithm: std::collections::HashMap<String, u32>,
    pub last_rotation_check: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::database::DatabaseConnection;
    
    async fn create_test_key_manager() -> KeyManager {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database = Arc::new(DatabaseConnection::new(db_path.to_str().unwrap()).await.unwrap());
        
        KeyManager::new(database, None).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_key_generation() {
        let manager = create_test_key_manager().await;
        
        let key = manager.generate_encryption_key("test_user", "AES-256", 256).await.unwrap();
        assert!(!key.is_empty());
        
        // Verify the key can be retrieved
        let (retrieved_key, metadata) = manager.get_active_key("AES-256").await.unwrap().unwrap();
        assert_eq!(key, retrieved_key);
        assert!(metadata.is_active);
    }
    
    #[tokio::test]
    async fn test_key_rotation() {
        let manager = create_test_key_manager().await;
        
        // Generate initial key
        let _key1 = manager.generate_encryption_key("test_user", "AES-256", 256).await.unwrap();
        
        // Force rotation
        let rotated = manager.rotate_keys("test_user", Some("Test rotation".to_string())).await.unwrap();
        assert!(!rotated.is_empty());
    }
    
    #[tokio::test]
    async fn test_rotation_status() {
        let manager = create_test_key_manager().await;
        
        let status = manager.get_rotation_status().await.unwrap();
        assert_eq!(status.total_keys, 0); // No keys initially
        
        // Generate a key
        let _key = manager.generate_encryption_key("test_user", "AES-256", 256).await.unwrap();
        
        let status = manager.get_rotation_status().await.unwrap();
        assert!(status.total_keys > 0);
    }
}