//! Integration tests for the security system
//! 
//! Tests the complete security functionality end-to-end

use std::sync::Arc;
use re_strike_vta::security::{
    SecureConfigManager, ConfigCategory, AccessLevel,
    ConfigMigrationTool, MigrationConfig,
    SecurityAudit, AuditAction,
    KeyManager
};
use re_strike_vta::database::DatabaseConnection;

#[tokio::test]
async fn test_complete_security_workflow() {
    // Initialize database
    let database = Arc::new(DatabaseConnection::new().unwrap());
    
    // Test 1: Create security session
    let config_manager = SecureConfigManager::new("test_master_password".to_string(), database.clone())
        .await.unwrap();
    
    let session = config_manager.create_session(
        "test_user".to_string(),
        AccessLevel::Administrator,
        Some("127.0.0.1".to_string()),
        Some("Integration Test".to_string()),
    ).await.unwrap();
    
    assert!(session.is_active);
    assert!(!session.is_expired());
    
    // Test 2: Store and retrieve encrypted configuration
    config_manager.set_config(
        &session.session_id,
        "test.password",
        "super_secret_password_123",
        ConfigCategory::ObsCredentials,
        Some("Test OBS password"),
    ).await.unwrap();
    
    let retrieved = config_manager.get_config(&session.session_id, "test.password")
        .await.unwrap().unwrap();
    
    assert_eq!(retrieved, "super_secret_password_123");
    
    // Test 3: Test key management
    let key_manager = KeyManager::new(database.clone(), None).await.unwrap();
    
    let generated_key = key_manager.generate_encryption_key("test_user", "AES-256", 256)
        .await.unwrap();
    
    assert!(!generated_key.is_empty());
    
    // Test 4: Test audit logging
    let audit = SecurityAudit::new(database.clone()).unwrap();
    
    audit.log_security_event(
        AuditAction::ConfigCreate,
        "test_user",
        "Integration test configuration created",
        true,
        None,
    ).await.unwrap();
    
    let recent_events = audit.get_recent_events(10).await.unwrap();
    assert!(!recent_events.is_empty());
    
    // Test 5: Test configuration migration
    let migration_config = MigrationConfig {
        master_password: "test_master_password".to_string(),
        backup_original: false,
        verify_after_migration: true,
    };
    
    let migration_tool = ConfigMigrationTool::new(config_manager, database.clone(), migration_config)
        .await.unwrap();
    
    // Create test JSON config
    let test_config = serde_json::json!({
        "obs": {
            "connections": [{
                "name": "test_obs",
                "host": "localhost",
                "port": 4455,
                "password": "test_obs_password"
            }]
        }
    });
    
    let stats = migration_tool.migrate_from_json_config(&session.session_id, &test_config)
        .await.unwrap();
    
    assert!(stats.credentials_migrated > 0);
    
    // Test 6: Verify migrated configuration can be retrieved
    let migrated_password = config_manager.get_config(&session.session_id, "obs.test_obs.password")
        .await.unwrap().unwrap();
    
    assert_eq!(migrated_password, "test_obs_password");
    
    // Test 7: Test session invalidation
    config_manager.invalidate_session(&session.session_id).await.unwrap();
    
    let invalid_session = config_manager.get_session(&session.session_id).await.unwrap();
    assert!(invalid_session.is_none() || !invalid_session.unwrap().is_active);
    
    println!("✅ All security integration tests passed!");
}

#[tokio::test]
async fn test_security_access_control() {
    let database = Arc::new(DatabaseConnection::new().unwrap());
    let config_manager = SecureConfigManager::new("test_master_password".to_string(), database.clone())
        .await.unwrap();
    
    // Create read-only session
    let readonly_session = config_manager.create_session(
        "readonly_user".to_string(),
        AccessLevel::ReadOnly,
        None, None,
    ).await.unwrap();
    
    // Create admin session
    let admin_session = config_manager.create_session(
        "admin_user".to_string(),
        AccessLevel::Administrator,
        None, None,
    ).await.unwrap();
    
    // Admin should be able to store sensitive config
    let result = config_manager.set_config(
        &admin_session.session_id,
        "admin.secret",
        "admin_secret_value",
        ConfigCategory::ObsCredentials,
        Some("Admin secret"),
    ).await;
    
    assert!(result.is_ok());
    
    // ReadOnly user should be able to read config
    let retrieved = config_manager.get_config(&readonly_session.session_id, "admin.secret")
        .await.unwrap();
    
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), "admin_secret_value");
    
    // ReadOnly user should NOT be able to modify config
    let result = config_manager.set_config(
        &readonly_session.session_id,
        "readonly.attempt",
        "should_fail",
        ConfigCategory::ObsCredentials,
        Some("Should fail"),
    ).await;
    
    assert!(result.is_err());
    
    println!("✅ Security access control tests passed!");
}

#[tokio::test]
async fn test_encryption_integrity() {
    let database = Arc::new(DatabaseConnection::new().unwrap());
    let config_manager = SecureConfigManager::new("test_master_password".to_string(), database.clone())
        .await.unwrap();
    
    let session = config_manager.create_session(
        "test_user".to_string(),
        AccessLevel::Administrator,
        None, None,
    ).await.unwrap();
    
    // Store various types of sensitive data
    let test_data = vec![
        ("password", "my_super_secret_password_!@#$%^&*()"),
        ("api_key", "sk-abcdef1234567890abcdef1234567890"),
        ("token", "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"),
        ("database_url", "postgresql://user:pass@localhost:5432/db"),
        ("special_chars", "áéíóúñüÇ€¥£$¢™©®°±×÷≠"),
    ];
    
    // Store all test data
    for (key, value) in &test_data {
        config_manager.set_config(
            &session.session_id,
            &format!("integrity.{}", key),
            value,
            ConfigCategory::ApiKeys,
            Some(&format!("Test {}", key)),
        ).await.unwrap();
    }
    
    // Retrieve and verify all test data
    for (key, expected_value) in &test_data {
        let retrieved = config_manager.get_config(&session.session_id, &format!("integrity.{}", key))
            .await.unwrap().unwrap();
        
        assert_eq!(retrieved, *expected_value, "Data integrity failed for key: {}", key);
    }
    
    println!("✅ Encryption integrity tests passed!");
}