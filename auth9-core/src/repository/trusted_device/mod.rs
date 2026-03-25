//! Trusted device repository

use crate::domains::identity::service::trusted_device::TrustedDevice;
use crate::error::Result;
use crate::models::common::StringUuid;
use async_trait::async_trait;
use sqlx::MySqlPool;

mod impl_repo;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TrustedDeviceRepository: Send + Sync {
    async fn find_by_user_and_fingerprint(
        &self,
        user_id: StringUuid,
        fingerprint: &str,
    ) -> Result<Option<TrustedDevice>>;
    async fn create(&self, device: &TrustedDevice) -> Result<()>;
    async fn list_by_user(&self, user_id: StringUuid) -> Result<Vec<TrustedDevice>>;
    async fn revoke(&self, device_id: StringUuid) -> Result<()>;
    async fn revoke_all_by_user(&self, user_id: StringUuid) -> Result<u64>;
    async fn update_last_used(&self, device_id: StringUuid) -> Result<()>;
    async fn delete_expired(&self) -> Result<u64>;
    async fn delete_by_user(&self, user_id: StringUuid) -> Result<u64>;
}

pub struct TrustedDeviceRepositoryImpl {
    pool: MySqlPool,
}

impl TrustedDeviceRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}
