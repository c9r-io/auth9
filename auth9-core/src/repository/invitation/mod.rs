//! Invitation repository

use crate::error::Result;
use crate::models::common::StringUuid;
use crate::models::invitation::{CreateInvitationInput, Invitation, InvitationStatus};
use async_trait::async_trait;
use sqlx::MySqlPool;

mod impl_repo;

#[cfg(test)]
mod tests;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InvitationRepository: Send + Sync {
    /// Create a new invitation
    async fn create(
        &self,
        tenant_id: StringUuid,
        invited_by: StringUuid,
        input: &CreateInvitationInput,
        token_hash: &str,
    ) -> Result<Invitation>;

    /// Find invitation by ID
    async fn find_by_id(&self, id: StringUuid) -> Result<Option<Invitation>>;

    /// Find invitation by email and tenant
    async fn find_by_email_and_tenant(
        &self,
        email: &str,
        tenant_id: StringUuid,
    ) -> Result<Option<Invitation>>;

    /// List invitations for a tenant with optional status filter
    async fn list_by_tenant(
        &self,
        tenant_id: StringUuid,
        status: Option<InvitationStatus>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Invitation>>;

    /// Count invitations for a tenant with optional status filter
    async fn count_by_tenant(
        &self,
        tenant_id: StringUuid,
        status: Option<InvitationStatus>,
    ) -> Result<i64>;

    /// List invitations for token verification (includes non-pending statuses)
    async fn list_pending(&self) -> Result<Vec<Invitation>>;

    /// Update invitation status
    async fn update_status(&self, id: StringUuid, status: InvitationStatus) -> Result<Invitation>;

    /// Mark invitation as accepted
    async fn mark_accepted(&self, id: StringUuid) -> Result<Invitation>;

    /// Update invitation token hash and updated_at timestamp
    async fn update_token_hash(&self, id: StringUuid, token_hash: &str) -> Result<Invitation>;

    /// Update invitation updated_at timestamp
    async fn touch_updated_at(&self, id: StringUuid) -> Result<Invitation>;

    /// Delete an invitation
    async fn delete(&self, id: StringUuid) -> Result<()>;

    /// Expire all pending invitations that have passed their expiration date
    async fn expire_pending(&self) -> Result<u64>;

    /// Delete all invitations for a tenant (for cascade delete)
    async fn delete_by_tenant(&self, tenant_id: StringUuid) -> Result<u64>;
}

pub struct InvitationRepositoryImpl {
    pool: MySqlPool,
}

impl InvitationRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}
