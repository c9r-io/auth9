//! Tenant risk policy repository

use crate::error::Result;
use crate::models::common::StringUuid;
use async_trait::async_trait;
use sqlx::MySqlPool;

mod impl_repo;

/// Tenant risk policy database entity
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TenantRiskPolicyRow {
    pub id: StringUuid,
    pub tenant_id: StringUuid,
    pub mfa_threshold: u8,
    pub block_threshold: u8,
    pub notify_admin: bool,
    pub auto_lock_account: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TenantRiskPolicyRepository: Send + Sync {
    async fn find_by_tenant_id(&self, tenant_id: StringUuid)
        -> Result<Option<TenantRiskPolicyRow>>;
    async fn upsert(&self, row: &TenantRiskPolicyRow) -> Result<()>;
    async fn delete_by_tenant_id(&self, tenant_id: StringUuid) -> Result<u64>;
}

pub struct TenantRiskPolicyRepositoryImpl {
    pool: MySqlPool,
}

impl TenantRiskPolicyRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}
