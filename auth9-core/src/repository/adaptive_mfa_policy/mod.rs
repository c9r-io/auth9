//! Adaptive MFA policy repository

use crate::error::Result;
use crate::models::common::StringUuid;
use async_trait::async_trait;
use sqlx::MySqlPool;

mod impl_repo;

/// Database row for adaptive MFA policy
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdaptiveMfaPolicyRow {
    pub id: StringUuid,
    pub tenant_id: StringUuid,
    pub mode: String,
    pub risk_threshold: u8,
    pub always_require_for_admins: bool,
    pub trust_device_days: u16,
    #[sqlx(json)]
    pub step_up_operations: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AdaptiveMfaPolicyRepository: Send + Sync {
    async fn find_by_tenant_id(
        &self,
        tenant_id: StringUuid,
    ) -> Result<Option<AdaptiveMfaPolicyRow>>;
    async fn upsert(&self, row: &AdaptiveMfaPolicyRow) -> Result<()>;
    async fn delete_by_tenant_id(&self, tenant_id: StringUuid) -> Result<u64>;
}

pub struct AdaptiveMfaPolicyRepositoryImpl {
    pool: MySqlPool,
}

impl AdaptiveMfaPolicyRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}
