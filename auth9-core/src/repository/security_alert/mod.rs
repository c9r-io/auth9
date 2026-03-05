//! Security alert repository

#[allow(unused_imports)]
use crate::domain::{
    AlertSeverity, CreateSecurityAlertInput, SecurityAlert, SecurityAlertType, StringUuid,
};
use crate::error::Result;
use async_trait::async_trait;
use sqlx::MySqlPool;

mod impl_repo;

#[cfg(test)]
mod tests;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SecurityAlertRepository: Send + Sync {
    async fn create(&self, input: &CreateSecurityAlertInput) -> Result<SecurityAlert>;
    async fn find_by_id(&self, id: StringUuid) -> Result<Option<SecurityAlert>>;
    async fn list(&self, offset: i64, limit: i64) -> Result<Vec<SecurityAlert>>;
    async fn list_unresolved(&self, offset: i64, limit: i64) -> Result<Vec<SecurityAlert>>;
    async fn list_by_user(
        &self,
        user_id: StringUuid,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<SecurityAlert>>;
    async fn list_by_severity(
        &self,
        severity: AlertSeverity,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<SecurityAlert>>;
    async fn list_filtered(
        &self,
        offset: i64,
        limit: i64,
        unresolved_only: bool,
        severity: Option<AlertSeverity>,
        alert_type: Option<SecurityAlertType>,
    ) -> Result<Vec<SecurityAlert>>;
    async fn count_filtered(
        &self,
        unresolved_only: bool,
        severity: Option<AlertSeverity>,
        alert_type: Option<SecurityAlertType>,
    ) -> Result<i64>;
    async fn count(&self) -> Result<i64>;
    async fn count_unresolved(&self) -> Result<i64>;
    async fn resolve(&self, id: StringUuid, resolved_by: StringUuid) -> Result<SecurityAlert>;
    async fn delete_old(&self, days: i64) -> Result<u64>;

    /// Nullify user_id for security alerts (preserve audit trail when user is deleted)
    async fn nullify_user_id(&self, user_id: StringUuid) -> Result<u64>;

    /// Delete all security alerts for a tenant (when tenant is deleted)
    async fn delete_by_tenant(&self, tenant_id: StringUuid) -> Result<u64>;
}

pub struct SecurityAlertRepositoryImpl {
    pool: MySqlPool,
}

impl SecurityAlertRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}
