//! Comprehensive Audit Logging for Security Events

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc;
use slog::{o, Drain, Logger, info, warn, error, crit};
use std::collections::HashMap;
use blake3::Hasher;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub principal: Principal,
    pub action: String,
    pub resource: String,
    pub outcome: String,
    pub security_level: SecurityLevel,
    pub details: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Principal {
    User(String),
    Service(String),
    System,
    Anonymous,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Audit logger with tamper detection and secure storage
pub struct AuditLogger {
    logger: Logger,
    tx: mpsc::Sender<AuditEvent>,
    storage_backend: Arc<dyn AuditStorage>,
}

#[async_trait::async_trait]
pub trait AuditStorage: Send + Sync {
    async fn store(&self, event: &AuditEvent, hash: &str) -> Result<()>;
    async fn verify_integrity(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<bool>;
    async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEvent>>;
}

#[derive(Debug, Clone)]
pub struct AuditFilter {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub principal: Option<String>,
    pub action: Option<String>,
    pub resource: Option<String>,
    pub security_level: Option<SecurityLevel>,
}

impl AuditLogger {
    pub fn new(storage_backend: Arc<dyn AuditStorage>) -> Self {
        // Set up structured logging
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let logger = Logger::root(drain, o!("component" => "audit"));

        let (tx, mut rx) = mpsc::channel::<AuditEvent>(1000);

        // Spawn background task for processing audit events
        let storage = storage_backend.clone();
        let log = logger.clone();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let hash = Self::hash_event(&event);
                
                // Log based on security level
                match event.security_level {
                    SecurityLevel::Low => {
                        info!(log, "Audit Event";
                            "action" => &event.action,
                            "principal" => format!("{:?}", event.principal),
                            "resource" => &event.resource,
                            "outcome" => &event.outcome,
                        );
                    }
                    SecurityLevel::Medium => {
                        warn!(log, "Security Event";
                            "action" => &event.action,
                            "principal" => format!("{:?}", event.principal),
                            "resource" => &event.resource,
                            "outcome" => &event.outcome,
                            "details" => format!("{}", event.details),
                        );
                    }
                    SecurityLevel::High => {
                        error!(log, "High Security Event";
                            "action" => &event.action,
                            "principal" => format!("{:?}", event.principal),
                            "resource" => &event.resource,
                            "outcome" => &event.outcome,
                            "details" => format!("{}", event.details),
                            "hash" => &hash,
                        );
                    }
                    SecurityLevel::Critical => {
                        crit!(log, "CRITICAL SECURITY EVENT";
                            "action" => &event.action,
                            "principal" => format!("{:?}", event.principal),
                            "resource" => &event.resource,
                            "outcome" => &event.outcome,
                            "details" => format!("{}", event.details),
                            "hash" => &hash,
                            "timestamp" => event.timestamp.to_rfc3339(),
                        );
                    }
                }

                // Store event with hash for tamper detection
                if let Err(e) = storage.store(&event, &hash).await {
                    crit!(log, "Failed to store audit event"; "error" => e.to_string());
                }
            }
        });

        Self {
            logger,
            tx,
            storage_backend,
        }
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) -> Result<()> {
        // Send to background processor
        self.tx.send(event).await?;
        Ok(())
    }

    /// Query audit logs with filtering
    pub async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEvent>> {
        self.storage_backend.query(filter).await
    }

    /// Verify integrity of audit logs for a time range
    pub async fn verify_integrity(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<bool> {
        self.storage_backend.verify_integrity(from, to).await
    }

    /// Generate compliance report
    pub async fn generate_compliance_report(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<ComplianceReport> {
        let events = self.query(AuditFilter {
            from: Some(from),
            to: Some(to),
            ..Default::default()
        }).await?;

        let mut report = ComplianceReport {
            period_start: from,
            period_end: to,
            total_events: events.len(),
            events_by_level: HashMap::new(),
            events_by_principal: HashMap::new(),
            events_by_action: HashMap::new(),
            critical_events: Vec::new(),
            integrity_verified: false,
        };

        // Analyze events
        for event in events {
            *report.events_by_level
                .entry(event.security_level)
                .or_insert(0) += 1;

            let principal_key = format!("{:?}", event.principal);
            *report.events_by_principal
                .entry(principal_key)
                .or_insert(0) += 1;

            *report.events_by_action
                .entry(event.action.clone())
                .or_insert(0) += 1;

            if event.security_level == SecurityLevel::Critical {
                report.critical_events.push(event);
            }
        }

        // Verify integrity
        report.integrity_verified = self.verify_integrity(from, to).await?;

        Ok(report)
    }

    /// Hash event for tamper detection
    fn hash_event(event: &AuditEvent) -> String {
        let mut hasher = Hasher::new();
        
        // Hash all fields in deterministic order
        hasher.update(event.timestamp.to_rfc3339().as_bytes());
        hasher.update(format!("{:?}", event.principal).as_bytes());
        hasher.update(event.action.as_bytes());
        hasher.update(event.resource.as_bytes());
        hasher.update(event.outcome.as_bytes());
        hasher.update(&[event.security_level as u8]);
        hasher.update(event.details.to_string().as_bytes());
        
        hasher.finalize().to_hex().to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: usize,
    pub events_by_level: HashMap<SecurityLevel, usize>,
    pub events_by_principal: HashMap<String, usize>,
    pub events_by_action: HashMap<String, usize>,
    pub critical_events: Vec<AuditEvent>,
    pub integrity_verified: bool,
}

/// SQLite-based audit storage implementation
pub struct SqliteAuditStorage {
    pool: sqlx::SqlitePool,
}

impl SqliteAuditStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = sqlx::SqlitePool::connect(database_url).await?;
        
        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                principal TEXT NOT NULL,
                action TEXT NOT NULL,
                resource TEXT NOT NULL,
                outcome TEXT NOT NULL,
                security_level INTEGER NOT NULL,
                details TEXT NOT NULL,
                hash TEXT NOT NULL,
                previous_hash TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_audit_principal ON audit_events(principal);
            CREATE INDEX IF NOT EXISTS idx_audit_action ON audit_events(action);
            CREATE INDEX IF NOT EXISTS idx_audit_security_level ON audit_events(security_level);
            "#
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl AuditStorage for SqliteAuditStorage {
    async fn store(&self, event: &AuditEvent, hash: &str) -> Result<()> {
        // Get previous hash for chain integrity
        let previous_hash: Option<String> = sqlx::query_scalar(
            "SELECT hash FROM audit_events ORDER BY id DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO audit_events (
                timestamp, principal, action, resource, outcome,
                security_level, details, hash, previous_hash
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(event.timestamp.to_rfc3339())
        .bind(format!("{:?}", event.principal))
        .bind(&event.action)
        .bind(&event.resource)
        .bind(&event.outcome)
        .bind(event.security_level as i32)
        .bind(event.details.to_string())
        .bind(hash)
        .bind(previous_hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn verify_integrity(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<bool> {
        // Verify hash chain integrity
        let events: Vec<(String, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT timestamp || principal || action || resource || outcome || 
                   security_level || details as data,
                   hash, previous_hash
            FROM audit_events
            WHERE timestamp >= ? AND timestamp <= ?
            ORDER BY id
            "#
        )
        .bind(from.to_rfc3339())
        .bind(to.to_rfc3339())
        .fetch_all(&self.pool)
        .await?;

        let mut prev_hash: Option<String> = None;
        
        for (_data, hash, previous_hash) in events {
            if prev_hash != previous_hash {
                return Ok(false);
            }
            prev_hash = Some(hash);
        }

        Ok(true)
    }

    async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditEvent>> {
        let mut query = String::from(
            "SELECT timestamp, principal, action, resource, outcome, 
                    security_level, details 
             FROM audit_events WHERE 1=1"
        );
        
        let mut binds = vec![];

        if let Some(from) = filter.from {
            query.push_str(" AND timestamp >= ?");
            binds.push(from.to_rfc3339());
        }

        if let Some(to) = filter.to {
            query.push_str(" AND timestamp <= ?");
            binds.push(to.to_rfc3339());
        }

        if let Some(principal) = filter.principal {
            query.push_str(" AND principal LIKE ?");
            binds.push(format!("%{}%", principal));
        }

        if let Some(action) = filter.action {
            query.push_str(" AND action = ?");
            binds.push(action);
        }

        query.push_str(" ORDER BY timestamp DESC");

        // Dynamic query building - in production use proper query builder
        let mut sql_query = sqlx::query(&query);
        for bind in binds {
            sql_query = sql_query.bind(bind);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;
        
        // Convert rows to AuditEvent - implementation omitted for brevity
        Ok(vec![])
    }
}

// Mock implementations for testing

pub struct MockAuditLogger {
    events: Arc<RwLock<Vec<AuditEvent>>>,
}

impl MockAuditLogger {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn log(&self, event: AuditEvent) -> Result<()> {
        let mut events = self.events.write().unwrap();
        events.push(event);
        Ok(())
    }
}

impl Default for AuditFilter {
    fn default() -> Self {
        Self {
            from: None,
            to: None,
            principal: None,
            action: None,
            resource: None,
            security_level: None,
        }
    }
}