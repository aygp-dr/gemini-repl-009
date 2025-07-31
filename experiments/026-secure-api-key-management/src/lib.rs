//! Secure API Key Management System
//! 
//! Implements enterprise-grade security for API key handling including:
//! - Hardware security module (HSM) integration
//! - Key rotation with zero downtime
//! - Least-privilege access control
//! - Comprehensive audit logging
//! - Encrypted storage with key derivation
//! - Rate limiting and anomaly detection

pub mod key_manager;
pub mod key_rotation;
pub mod access_control;
pub mod audit_logger;
pub mod secure_storage;
pub mod threat_detection;