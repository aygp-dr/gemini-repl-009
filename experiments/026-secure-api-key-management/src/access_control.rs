//! Role-Based Access Control (RBAC) with Least-Privilege Principles

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::audit_logger::{AuditLogger, AuditEvent, Principal as AuditPrincipal, SecurityLevel};
use chrono::Utc;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // API Key Management
    CreateApiKey,
    ReadApiKey,
    UseApiKey,
    RotateApiKey,
    RevokeApiKey,
    
    // Data Access
    ReadData,
    WriteData,
    DeleteData,
    
    // Audit
    ViewAuditLogs,
    ExportAuditLogs,
    VerifyAuditIntegrity,
    
    // Administration
    ManageUsers,
    ManageRoles,
    ManagePermissions,
    
    // Security
    SecurityIncidentResponse,
    EmergencyAccess,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    // Predefined roles
    Admin,
    SecurityOfficer,
    Developer,
    Auditor,
    ReadOnly,
    
    // Custom role
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct AccessControl {
    role_permissions: Arc<RwLock<HashMap<Role, HashSet<Permission>>>>,
    user_roles: Arc<RwLock<HashMap<String, HashSet<Role>>>>,
    audit_logger: Arc<AuditLogger>,
}

impl AccessControl {
    pub fn new(audit_logger: Arc<AuditLogger>) -> Self {
        let mut role_permissions = HashMap::new();
        
        // Define default role permissions (principle of least privilege)
        
        // Admin: Full access
        role_permissions.insert(
            Role::Admin,
            vec![
                Permission::CreateApiKey,
                Permission::ReadApiKey,
                Permission::UseApiKey,
                Permission::RotateApiKey,
                Permission::RevokeApiKey,
                Permission::ReadData,
                Permission::WriteData,
                Permission::DeleteData,
                Permission::ViewAuditLogs,
                Permission::ExportAuditLogs,
                Permission::VerifyAuditIntegrity,
                Permission::ManageUsers,
                Permission::ManageRoles,
                Permission::ManagePermissions,
                Permission::SecurityIncidentResponse,
                Permission::EmergencyAccess,
            ].into_iter().collect()
        );
        
        // Security Officer: Security-focused permissions
        role_permissions.insert(
            Role::SecurityOfficer,
            vec![
                Permission::ReadApiKey,
                Permission::RotateApiKey,
                Permission::RevokeApiKey,
                Permission::ViewAuditLogs,
                Permission::ExportAuditLogs,
                Permission::VerifyAuditIntegrity,
                Permission::SecurityIncidentResponse,
                Permission::EmergencyAccess,
            ].into_iter().collect()
        );
        
        // Developer: Standard development permissions
        role_permissions.insert(
            Role::Developer,
            vec![
                Permission::CreateApiKey,
                Permission::ReadApiKey,
                Permission::UseApiKey,
                Permission::RotateApiKey,
                Permission::ReadData,
                Permission::WriteData,
            ].into_iter().collect()
        );
        
        // Auditor: Read-only audit access
        role_permissions.insert(
            Role::Auditor,
            vec![
                Permission::ViewAuditLogs,
                Permission::ExportAuditLogs,
                Permission::VerifyAuditIntegrity,
            ].into_iter().collect()
        );
        
        // ReadOnly: Minimal permissions
        role_permissions.insert(
            Role::ReadOnly,
            vec![
                Permission::ReadData,
            ].into_iter().collect()
        );
        
        Self {
            role_permissions: Arc::new(RwLock::new(role_permissions)),
            user_roles: Arc::new(RwLock::new(HashMap::new())),
            audit_logger,
        }
    }
    
    /// Check if a principal has a specific permission
    pub async fn check_permission(
        &self,
        principal: &Principal,
        permission: &Permission,
    ) -> Result<()> {
        let has_permission = self.has_permission(principal, permission).await?;
        
        // Audit the permission check
        let outcome = if has_permission { "granted" } else { "denied" };
        let security_level = match permission {
            Permission::EmergencyAccess | 
            Permission::SecurityIncidentResponse |
            Permission::RevokeApiKey => SecurityLevel::High,
            
            Permission::ManageUsers |
            Permission::ManageRoles |
            Permission::ManagePermissions => SecurityLevel::Medium,
            
            _ => SecurityLevel::Low,
        };
        
        self.audit_logger.log(AuditEvent {
            timestamp: Utc::now(),
            principal: self.principal_to_audit(principal),
            action: "permission_check".to_string(),
            resource: format!("permission:{:?}", permission),
            outcome: outcome.to_string(),
            security_level,
            details: serde_json::json!({
                "permission": permission,
                "granted": has_permission,
            }),
        }).await?;
        
        if has_permission {
            Ok(())
        } else {
            Err(anyhow!("Permission denied: {:?}", permission))
        }
    }
    
    /// Check if principal has permission (internal, no audit)
    async fn has_permission(
        &self,
        principal: &Principal,
        permission: &Permission,
    ) -> Result<bool> {
        match principal {
            Principal::System => Ok(true), // System has all permissions
            
            Principal::User(user_id) => {
                let user_roles = self.user_roles.read().await;
                let role_permissions = self.role_permissions.read().await;
                
                if let Some(roles) = user_roles.get(user_id) {
                    for role in roles {
                        if let Some(perms) = role_permissions.get(role) {
                            if perms.contains(permission) {
                                return Ok(true);
                            }
                        }
                    }
                }
                Ok(false)
            }
            
            Principal::Service(service_id) => {
                // Services have limited, predefined permissions
                match (service_id.as_str(), permission) {
                    ("monitoring", Permission::ViewAuditLogs) => Ok(true),
                    ("backup", Permission::ExportAuditLogs) => Ok(true),
                    _ => Ok(false),
                }
            }
            
            Principal::Anonymous => Ok(false), // Anonymous has no permissions
        }
    }
    
    /// Grant a role to a user
    pub async fn grant_role(
        &self,
        granter: &Principal,
        user_id: &str,
        role: Role,
    ) -> Result<()> {
        // Check if granter has permission to manage roles
        self.check_permission(granter, &Permission::ManageRoles).await?;
        
        // Add role to user
        {
            let mut user_roles = self.user_roles.write().await;
            user_roles.entry(user_id.to_string())
                .or_insert_with(HashSet::new)
                .insert(role.clone());
        }
        
        // Audit log
        self.audit_logger.log(AuditEvent {
            timestamp: Utc::now(),
            principal: self.principal_to_audit(granter),
            action: "grant_role".to_string(),
            resource: format!("user:{}", user_id),
            outcome: "success".to_string(),
            security_level: SecurityLevel::Medium,
            details: serde_json::json!({
                "role": role,
                "user": user_id,
            }),
        }).await?;
        
        Ok(())
    }
    
    /// Revoke a role from a user
    pub async fn revoke_role(
        &self,
        revoker: &Principal,
        user_id: &str,
        role: &Role,
    ) -> Result<()> {
        // Check if revoker has permission to manage roles
        self.check_permission(revoker, &Permission::ManageRoles).await?;
        
        // Remove role from user
        {
            let mut user_roles = self.user_roles.write().await;
            if let Some(roles) = user_roles.get_mut(user_id) {
                roles.remove(role);
            }
        }
        
        // Audit log
        self.audit_logger.log(AuditEvent {
            timestamp: Utc::now(),
            principal: self.principal_to_audit(revoker),
            action: "revoke_role".to_string(),
            resource: format!("user:{}", user_id),
            outcome: "success".to_string(),
            security_level: SecurityLevel::Medium,
            details: serde_json::json!({
                "role": role,
                "user": user_id,
            }),
        }).await?;
        
        Ok(())
    }
    
    /// Create a custom role with specific permissions
    pub async fn create_custom_role(
        &self,
        creator: &Principal,
        role_name: String,
        permissions: HashSet<Permission>,
    ) -> Result<()> {
        // Check if creator has permission to manage roles
        self.check_permission(creator, &Permission::ManageRoles).await?;
        
        // Validate role name
        if role_name.is_empty() || role_name.len() > 50 {
            return Err(anyhow!("Invalid role name"));
        }
        
        // Add custom role
        {
            let mut role_permissions = self.role_permissions.write().await;
            role_permissions.insert(Role::Custom(role_name.clone()), permissions.clone());
        }
        
        // Audit log
        self.audit_logger.log(AuditEvent {
            timestamp: Utc::now(),
            principal: self.principal_to_audit(creator),
            action: "create_custom_role".to_string(),
            resource: format!("role:{}", role_name),
            outcome: "success".to_string(),
            security_level: SecurityLevel::High,
            details: serde_json::json!({
                "role_name": role_name,
                "permissions": permissions,
            }),
        }).await?;
        
        Ok(())
    }
    
    /// Get effective permissions for a principal
    pub async fn get_effective_permissions(
        &self,
        principal: &Principal,
    ) -> Result<HashSet<Permission>> {
        match principal {
            Principal::System => {
                // System has all permissions
                Ok(vec![
                    Permission::CreateApiKey,
                    Permission::ReadApiKey,
                    Permission::UseApiKey,
                    Permission::RotateApiKey,
                    Permission::RevokeApiKey,
                    Permission::ReadData,
                    Permission::WriteData,
                    Permission::DeleteData,
                    Permission::ViewAuditLogs,
                    Permission::ExportAuditLogs,
                    Permission::VerifyAuditIntegrity,
                    Permission::ManageUsers,
                    Permission::ManageRoles,
                    Permission::ManagePermissions,
                    Permission::SecurityIncidentResponse,
                    Permission::EmergencyAccess,
                ].into_iter().collect())
            }
            
            Principal::User(user_id) => {
                let mut permissions = HashSet::new();
                let user_roles = self.user_roles.read().await;
                let role_permissions = self.role_permissions.read().await;
                
                if let Some(roles) = user_roles.get(user_id) {
                    for role in roles {
                        if let Some(perms) = role_permissions.get(role) {
                            permissions.extend(perms.iter().cloned());
                        }
                    }
                }
                
                Ok(permissions)
            }
            
            Principal::Service(service_id) => {
                // Return predefined service permissions
                let permissions = match service_id.as_str() {
                    "monitoring" => vec![Permission::ViewAuditLogs],
                    "backup" => vec![Permission::ExportAuditLogs],
                    _ => vec![],
                };
                Ok(permissions.into_iter().collect())
            }
            
            Principal::Anonymous => Ok(HashSet::new()),
        }
    }
    
    fn principal_to_audit(&self, principal: &Principal) -> AuditPrincipal {
        match principal {
            Principal::User(id) => AuditPrincipal::User(id.clone()),
            Principal::Service(id) => AuditPrincipal::Service(id.clone()),
            Principal::System => AuditPrincipal::System,
            Principal::Anonymous => AuditPrincipal::Anonymous,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Principal {
    User(String),
    Service(String),
    System,
    Anonymous,
}

// Mock implementations for testing

pub struct MockAccessControl;

impl MockAccessControl {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn check_permission(
        &self,
        _principal: &Principal,
        _permission: &Permission,
    ) -> Result<()> {
        Ok(()) // Always grant in mock
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit_logger::MockAuditLogger;
    
    #[tokio::test]
    async fn test_role_based_permissions() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let ac = AccessControl::new(audit_logger);
        
        // Grant developer role to user
        let admin = Principal::System;
        let user_id = "test_user";
        
        ac.grant_role(&admin, user_id, Role::Developer).await.unwrap();
        
        // Check developer permissions
        let user = Principal::User(user_id.to_string());
        assert!(ac.check_permission(&user, &Permission::ReadData).await.is_ok());
        assert!(ac.check_permission(&user, &Permission::WriteData).await.is_ok());
        
        // Check permission user doesn't have
        assert!(ac.check_permission(&user, &Permission::ManageUsers).await.is_err());
    }
    
    #[tokio::test]
    async fn test_custom_roles() {
        let audit_logger = Arc::new(MockAuditLogger::new());
        let ac = AccessControl::new(audit_logger);
        
        // Create custom role
        let admin = Principal::System;
        let custom_perms = vec![
            Permission::ReadData,
            Permission::ViewAuditLogs,
        ].into_iter().collect();
        
        ac.create_custom_role(
            &admin,
            "CustomReader".to_string(),
            custom_perms,
        ).await.unwrap();
        
        // Grant custom role
        ac.grant_role(&admin, "test_user", Role::Custom("CustomReader".to_string()))
            .await.unwrap();
        
        // Verify permissions
        let user = Principal::User("test_user".to_string());
        assert!(ac.check_permission(&user, &Permission::ReadData).await.is_ok());
        assert!(ac.check_permission(&user, &Permission::ViewAuditLogs).await.is_ok());
        assert!(ac.check_permission(&user, &Permission::WriteData).await.is_err());
    }
}