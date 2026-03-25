//! User login profile repository

use crate::domains::security_observability::service::user_profile::UserLoginProfile;
use crate::error::Result;
use crate::models::common::StringUuid;
use async_trait::async_trait;
use sqlx::MySqlPool;

mod impl_repo;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserLoginProfileRepository: Send + Sync {
    async fn find_by_user_id(&self, user_id: StringUuid) -> Result<Option<UserLoginProfile>>;
    async fn upsert(&self, profile: &UserLoginProfile) -> Result<()>;
    async fn delete_by_user_id(&self, user_id: StringUuid) -> Result<u64>;
}

pub struct UserLoginProfileRepositoryImpl {
    pool: MySqlPool,
}

impl UserLoginProfileRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}
