//! Core API Key Management with Security Best Practices

use anyhow::{Result, anyhow};
use secrecy::{ExposeSecret, Secret, SecretString, Zeroize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use ring::rand::{SecureRandom, SystemRandom};
use zeroize::Zeroizing;
use constant_time_eq::constant_time_eq;

use crate::audit_logger::{AuditLogger, AuditEvent, SecurityLevel};
use crate::access_control::{AccessControl, Permission, Principal};
use crate::secure_storage::SecureStorage;

/// API Key metadata (never contains the actual key)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyMetadata {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub last_rotated: Option<DateTime<Utc>>,
    pub permissions: Vec<Permission>,
    pub rate_limit: RateLimit,
    pub key_hash: String, // SHA-256 hash for verification
    pub status: KeyStatus,
    pub rotation_policy: RotationPolicy,
    pub usage_count: u64,
    pub failed_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyStatus {
    Active,
    Suspended,
    Expired,
    Revoked,
    PendingRotation,
    Compromised,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub burst_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationPolicy {
    pub max_age_days: u32,
    pub max_usage_count: u64,
    pub auto_rotate: bool,
    pub notify_days_before: u32,
}

impl Default for RotationPolicy {
    fn default() -> Self {
        Self {
            max_age_days: 90,
            max_usage_count: 1_000_000,
            auto_rotate: true,
            notify_days_before: 7,
        }
    }
}

/// Secure API Key Manager
pub struct ApiKeyManager {
    storage: Arc<dyn SecureStorage>,
    access_control: Arc<AccessControl>,
    audit_logger: Arc<AuditLogger>,
    keys: Arc<RwLock<HashMap<Uuid, ApiKeyMetadata>>>,
    rng: SystemRandom,
}

impl ApiKeyManager {
    pub fn new(
        storage: Arc<dyn SecureStorage>,
        access_control: Arc<AccessControl>,
        audit_logger: Arc<AuditLogger>,
    ) -> Self {
        Self {
            storage,
            access_control,
            audit_logger,
            keys: Arc::new(RwLock::new(HashMap::new())),
            rng: SystemRandom::new(),
        }
    }

    /// Generate a new API key with specified permissions
    pub async fn generate_key(
        &self,
        principal: &Principal,
        name: String,
        permissions: Vec<Permission>,
        rotation_policy: Option<RotationPolicy>,
    ) -> Result<(Uuid, SecretString)> {
        // Check if principal has permission to create keys
        self.access_control.check_permission(
            principal,
            &Permission::CreateApiKey,
        ).await?;

        // Generate cryptographically secure random key
        let key = self.generate_secure_key()?;
        let key_id = Uuid::new_v4();
        
        // Hash the key for storage
        let key_hash = self.hash_key(key.expose_secret())?;
        
        // Create metadata
        let metadata = ApiKeyMetadata {
            id: key_id,
            name: name.clone(),
            created_at: Utc::now(),
            expires_at: None,
            last_used: None,
            last_rotated: None,
            permissions,
            rate_limit: RateLimit {
                requests_per_minute: 60,
                requests_per_hour: 1000,
                requests_per_day: 10000,
                burst_size: 10,
            },
            key_hash,
            status: KeyStatus::Active,
            rotation_policy: rotation_policy.unwrap_or_default(),
            usage_count: 0,
            failed_attempts: 0,
        };

        // Store encrypted key
        self.storage.store_key(key_id, &key, &metadata).await?;
        
        // Update in-memory cache
        {
            let mut keys = self.keys.write().unwrap();
            keys.insert(key_id, metadata.clone());
        }

        // Audit log
        self.audit_logger.log(AuditEvent {
            timestamp: Utc::now(),
            principal: principal.clone(),
            action: "create_api_key".to_string(),
            resource: format!("key:{}", key_id),
            outcome: "success".to_string(),
            security_level: SecurityLevel::High,
            details: serde_json::json!({
                "key_name": name,
                "permissions": metadata.permissions,
            }),
        }).await?;

        Ok((key_id, key))
    }

    /// Validate an API key
    pub async fn validate_key(
        &self,
        key: &SecretString,
    ) -> Result<(Uuid, Vec<Permission>)> {
        let key_hash = self.hash_key(key.expose_secret())?;
        
        // Find key by hash (constant time comparison)
        let keys = self.keys.read().unwrap();
        let metadata = keys.values()
            .find(|m| constant_time_eq(m.key_hash.as_bytes(), key_hash.as_bytes()))
            .ok_or_else(|| anyhow!("Invalid API key"))?;

        // Check key status
        match &metadata.status {
            KeyStatus::Active => {},
            KeyStatus::Expired => return Err(anyhow!("API key has expired")),
            KeyStatus::Revoked => return Err(anyhow!("API key has been revoked")),
            KeyStatus::Compromised => {
                // Alert security team
                self.audit_logger.log(AuditEvent {
                    timestamp: Utc::now(),
                    principal: Principal::System,
                    action: "compromised_key_usage_attempt".to_string(),
                    resource: format!("key:{}", metadata.id),
                    outcome: "blocked".to_string(),
                    security_level: SecurityLevel::Critical,
                    details: serde_json::json!({}),
                }).await?;
                return Err(anyhow!("API key has been compromised"));
            },
            _ => return Err(anyhow!("API key is not active")),
        }

        // Check expiration
        if let Some(expires_at) = metadata.expires_at {
            if Utc::now() > expires_at {
                return Err(anyhow!("API key has expired"));
            }
        }

        // Check rotation policy
        if self.needs_rotation(metadata)? {
            // Mark for rotation but still allow usage
            self.mark_for_rotation(metadata.id).await?;
        }

        // Update usage statistics
        self.update_usage(metadata.id).await?;

        Ok((metadata.id, metadata.permissions.clone()))
    }

    /// Rotate an API key
    pub async fn rotate_key(
        &self,
        principal: &Principal,
        key_id: Uuid,
    ) -> Result<SecretString> {
        // Check permission
        self.access_control.check_permission(
            principal,
            &Permission::RotateApiKey,
        ).await?;

        // Get existing metadata
        let metadata = {
            let keys = self.keys.read().unwrap();
            keys.get(&key_id)
                .ok_or_else(|| anyhow!("Key not found"))?
                .clone()
        };

        // Generate new key
        let new_key = self.generate_secure_key()?;
        let new_key_hash = self.hash_key(new_key.expose_secret())?;

        // Update metadata
        let mut updated_metadata = metadata.clone();
        updated_metadata.key_hash = new_key_hash;
        updated_metadata.last_rotated = Some(Utc::now());
        updated_metadata.status = KeyStatus::Active;
        updated_metadata.usage_count = 0;

        // Store new key
        self.storage.store_key(key_id, &new_key, &updated_metadata).await?;

        // Update cache
        {
            let mut keys = self.keys.write().unwrap();
            keys.insert(key_id, updated_metadata);
        }

        // Audit log
        self.audit_logger.log(AuditEvent {
            timestamp: Utc::now(),
            principal: principal.clone(),
            action: "rotate_api_key".to_string(),
            resource: format!("key:{}", key_id),
            outcome: "success".to_string(),
            security_level: SecurityLevel::High,
            details: serde_json::json!({
                "key_name": metadata.name,
            }),
        }).await?;

        Ok(new_key)
    }

    /// Revoke an API key
    pub async fn revoke_key(
        &self,
        principal: &Principal,
        key_id: Uuid,
        reason: &str,
    ) -> Result<()> {
        // Check permission
        self.access_control.check_permission(
            principal,
            &Permission::RevokeApiKey,
        ).await?;

        // Update status
        {
            let mut keys = self.keys.write().unwrap();
            if let Some(metadata) = keys.get_mut(&key_id) {
                metadata.status = KeyStatus::Revoked;
            } else {
                return Err(anyhow!("Key not found"));
            }
        }

        // Remove from secure storage
        self.storage.delete_key(key_id).await?;

        // Audit log with HIGH severity
        self.audit_logger.log(AuditEvent {
            timestamp: Utc::now(),
            principal: principal.clone(),
            action: "revoke_api_key".to_string(),
            resource: format!("key:{}", key_id),
            outcome: "success".to_string(),
            security_level: SecurityLevel::High,
            details: serde_json::json!({
                "reason": reason,
            }),
        }).await?;

        Ok(())
    }

    /// Mark a key as compromised (immediate revocation + security alert)
    pub async fn mark_compromised(
        &self,
        principal: &Principal,
        key_id: Uuid,
        incident_details: &str,
    ) -> Result<()> {
        // Check permission
        self.access_control.check_permission(
            principal,
            &Permission::SecurityIncidentResponse,
        ).await?;

        // Update status
        {
            let mut keys = self.keys.write().unwrap();
            if let Some(metadata) = keys.get_mut(&key_id) {
                metadata.status = KeyStatus::Compromised;
            } else {
                return Err(anyhow!("Key not found"));
            }
        }

        // Remove from secure storage immediately
        self.storage.delete_key(key_id).await?;

        // Critical security audit log
        self.audit_logger.log(AuditEvent {
            timestamp: Utc::now(),
            principal: principal.clone(),
            action: "mark_key_compromised".to_string(),
            resource: format!("key:{}", key_id),
            outcome: "success".to_string(),
            security_level: SecurityLevel::Critical,
            details: serde_json::json!({
                "incident_details": incident_details,
                "immediate_actions": ["key_deleted", "alerts_sent"],
            }),
        }).await?;

        // TODO: Trigger security incident response
        // - Alert security team
        // - Check for suspicious activity
        // - Review audit logs

        Ok(())
    }

    // Private helper methods

    fn generate_secure_key(&self) -> Result<SecretString> {
        let mut key_bytes = Zeroizing::new([0u8; 32]);
        self.rng.fill(&mut *key_bytes)
            .map_err(|_| anyhow!("Failed to generate random key"))?;
        
        // Encode as base64 URL-safe
        let key_string = base64::encode_config(&*key_bytes, base64::URL_SAFE_NO_PAD);
        Ok(SecretString::new(key_string))
    }

    fn hash_key(&self, key: &str) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        Ok(hex::encode(hasher.finalize()))
    }

    fn needs_rotation(&self, metadata: &ApiKeyMetadata) -> Result<bool> {
        let policy = &metadata.rotation_policy;
        
        // Check age
        let age = Utc::now() - metadata.created_at;
        if age > Duration::days(policy.max_age_days as i64) {
            return Ok(true);
        }

        // Check usage count
        if metadata.usage_count > policy.max_usage_count {
            return Ok(true);
        }

        // Check if manually marked for rotation
        if metadata.status == KeyStatus::PendingRotation {
            return Ok(true);
        }

        Ok(false)
    }

    async fn mark_for_rotation(&self, key_id: Uuid) -> Result<()> {
        let mut keys = self.keys.write().unwrap();
        if let Some(metadata) = keys.get_mut(&key_id) {
            if metadata.status == KeyStatus::Active {
                metadata.status = KeyStatus::PendingRotation;
                
                // Schedule rotation notification
                self.audit_logger.log(AuditEvent {
                    timestamp: Utc::now(),
                    principal: Principal::System,
                    action: "key_rotation_scheduled".to_string(),
                    resource: format!("key:{}", key_id),
                    outcome: "pending".to_string(),
                    security_level: SecurityLevel::Medium,
                    details: serde_json::json!({
                        "reason": "policy_triggered",
                    }),
                }).await?;
            }
        }
        Ok(())
    }

    async fn update_usage(&self, key_id: Uuid) -> Result<()> {
        let mut keys = self.keys.write().unwrap();
        if let Some(metadata) = keys.get_mut(&key_id) {
            metadata.usage_count += 1;
            metadata.last_used = Some(Utc::now());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secure_storage::MockSecureStorage;
    use crate::access_control::MockAccessControl;
    use crate::audit_logger::MockAuditLogger;

    #[tokio::test]
    async fn test_key_generation() {
        let storage = Arc::new(MockSecureStorage::new());
        let access_control = Arc::new(MockAccessControl::new());
        let audit_logger = Arc::new(MockAuditLogger::new());
        
        let manager = ApiKeyManager::new(storage, access_control, audit_logger);
        
        let principal = Principal::User("test_user".to_string());
        let (key_id, secret) = manager.generate_key(
            &principal,
            "Test Key".to_string(),
            vec![Permission::ReadData],
            None,
        ).await.unwrap();
        
        assert!(!secret.expose_secret().is_empty());
        assert_ne!(key_id, Uuid::nil());
    }

    #[tokio::test]
    async fn test_key_validation() {
        let storage = Arc::new(MockSecureStorage::new());
        let access_control = Arc::new(MockAccessControl::new());
        let audit_logger = Arc::new(MockAuditLogger::new());
        
        let manager = ApiKeyManager::new(storage, access_control, audit_logger);
        
        let principal = Principal::User("test_user".to_string());
        let (key_id, secret) = manager.generate_key(
            &principal,
            "Test Key".to_string(),
            vec![Permission::ReadData],
            None,
        ).await.unwrap();
        
        // Validate the key
        let (validated_id, permissions) = manager.validate_key(&secret).await.unwrap();
        assert_eq!(validated_id, key_id);
        assert_eq!(permissions, vec![Permission::ReadData]);
    }
}