//! Audit log repository

use crate::error::Result;
use crate::models::common::StringUuid;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, MySqlPool, Row};
use uuid::Uuid;

mod impl_repo;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: i64,
    pub actor_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Audit log entry with actor information (for API responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogWithActor {
    pub id: i64,
    pub actor_id: Option<String>,
    pub actor_email: Option<String>,
    pub actor_display_name: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl<'r> FromRow<'r, MySqlRow> for AuditLogWithActor {
    fn from_row(row: &'r MySqlRow) -> sqlx::Result<Self> {
        let id: i64 = row.try_get("id")?;
        let actor_id: Option<String> = row.try_get("actor_id")?;
        let actor_email: Option<String> = row.try_get("actor_email")?;
        let actor_display_name: Option<String> = row.try_get("actor_display_name")?;
        let action: String = row.try_get("action")?;
        let resource_type: String = row.try_get("resource_type")?;
        let resource_id: Option<String> = row.try_get("resource_id")?;

        let old_value_wrapper: Option<sqlx::types::Json<serde_json::Value>> =
            row.try_get("old_value")?;
        let old_value = old_value_wrapper.map(|w| w.0);

        let new_value_wrapper: Option<sqlx::types::Json<serde_json::Value>> =
            row.try_get("new_value")?;
        let new_value = new_value_wrapper.map(|w| w.0);

        let ip_address: Option<String> = row.try_get("ip_address")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;

        Ok(AuditLogWithActor {
            id,
            actor_id,
            actor_email,
            actor_display_name,
            action,
            resource_type,
            resource_id,
            old_value,
            new_value,
            ip_address,
            created_at,
        })
    }
}

impl<'r> FromRow<'r, MySqlRow> for AuditLog {
    fn from_row(row: &'r MySqlRow) -> sqlx::Result<Self> {
        let id: i64 = row.try_get("id")?;
        let actor_id: Option<String> = row.try_get("actor_id")?;
        let action: String = row.try_get("action")?;
        let resource_type: String = row.try_get("resource_type")?;
        let resource_id: Option<String> = row.try_get("resource_id")?;

        // Handle JSON fields that might be NULL
        // We read them as Option<String> or Option<serde_json::Value> explicitly
        // If the column is JSON type in MySQL, sqlx treats it as Value if valid.
        // But the issue was UnexpectedNullError when it was NULL and mapped to Option<Value>.
        // Let's try reading as Option<serde_json::Value> directly but without the macro magic first?
        // Actually, the macro magic IS what caused the issue (likely strictness).
        // A safer way is to read as Option<sqlx::types::Json<serde_json::Value>> and unwrap
        // OR read as Option<serde_json::Value> manually.

        let old_value_wrapper: Option<sqlx::types::Json<serde_json::Value>> =
            row.try_get("old_value")?;
        let old_value = old_value_wrapper.map(|w| w.0);

        let new_value_wrapper: Option<sqlx::types::Json<serde_json::Value>> =
            row.try_get("new_value")?;
        let new_value = new_value_wrapper.map(|w| w.0);

        let ip_address: Option<String> = row.try_get("ip_address")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;

        Ok(AuditLog {
            id,
            actor_id,
            action,
            resource_type,
            resource_id,
            old_value,
            new_value,
            ip_address,
            created_at,
        })
    }
}

/// Input for creating an audit log entry
#[derive(Debug, Clone)]
pub struct CreateAuditLogInput {
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub ip_address: Option<String>,
}

/// Audit log query parameters
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AuditLogQuery {
    pub actor_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub action: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn create(&self, input: &CreateAuditLogInput) -> Result<()>;
    async fn find(&self, query: &AuditLogQuery) -> Result<Vec<AuditLog>>;
    /// Find audit logs with actor information (email, display_name) for API responses
    async fn find_with_actor(&self, query: &AuditLogQuery) -> Result<Vec<AuditLogWithActor>>;
    async fn count(&self, query: &AuditLogQuery) -> Result<i64>;

    /// Nullify actor_id for audit logs (preserve audit trail when user is deleted)
    async fn nullify_actor_id(&self, actor_id: StringUuid) -> Result<u64>;
}

pub struct AuditRepositoryImpl {
    pool: MySqlPool,
}

impl AuditRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}
