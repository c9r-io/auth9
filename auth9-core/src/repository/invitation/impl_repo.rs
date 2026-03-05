//! impl InvitationRepository for InvitationRepositoryImpl

use super::{InvitationRepository, InvitationRepositoryImpl};
use crate::domain::{CreateInvitationInput, Invitation, InvitationStatus, StringUuid};
use crate::error::{AppError, Result};
use async_trait::async_trait;
use chrono::{Duration, Utc};

#[async_trait]
impl InvitationRepository for InvitationRepositoryImpl {
    async fn create(
        &self,
        tenant_id: StringUuid,
        invited_by: StringUuid,
        input: &CreateInvitationInput,
        token_hash: &str,
    ) -> Result<Invitation> {
        let id = StringUuid::new_v4();
        let expires_in = input.expires_in_hours.unwrap_or(72);
        let expires_at = Utc::now() + Duration::hours(expires_in);
        let role_ids_json =
            serde_json::to_string(&input.role_ids).map_err(|e| AppError::Internal(e.into()))?;

        sqlx::query(
            r#"
            INSERT INTO invitations (id, tenant_id, email, role_ids, invited_by, token_hash, status, expires_at, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, 'pending', ?, NOW(), NOW())
            "#,
        )
        .bind(id)
        .bind(tenant_id)
        .bind(&input.email)
        .bind(&role_ids_json)
        .bind(invited_by)
        .bind(token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Failed to create invitation")))
    }

    async fn find_by_id(&self, id: StringUuid) -> Result<Option<Invitation>> {
        let invitation = sqlx::query_as::<_, Invitation>(
            r#"
            SELECT id, tenant_id, email, role_ids, invited_by, token_hash, status, expires_at, accepted_at, created_at, updated_at
            FROM invitations
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(invitation)
    }

    async fn find_by_email_and_tenant(
        &self,
        email: &str,
        tenant_id: StringUuid,
    ) -> Result<Option<Invitation>> {
        let invitation = sqlx::query_as::<_, Invitation>(
            r#"
            SELECT id, tenant_id, email, role_ids, invited_by, token_hash, status, expires_at, accepted_at, created_at, updated_at
            FROM invitations
            WHERE email = ? AND tenant_id = ? AND status = 'pending'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(email)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(invitation)
    }

    async fn list_by_tenant(
        &self,
        tenant_id: StringUuid,
        status: Option<InvitationStatus>,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Invitation>> {
        let invitations = match status {
            Some(InvitationStatus::Expired) => {
                // "Expired" = pending in DB but past expires_at, OR explicitly marked expired
                sqlx::query_as::<_, Invitation>(
                    r#"
                    SELECT id, tenant_id, email, role_ids, invited_by, token_hash, status, expires_at, accepted_at, created_at, updated_at
                    FROM invitations
                    WHERE tenant_id = ? AND (status = 'expired' OR (status = 'pending' AND expires_at <= NOW()))
                    ORDER BY created_at DESC
                    LIMIT ? OFFSET ?
                    "#,
                )
                .bind(tenant_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            Some(InvitationStatus::Pending) => {
                // "Pending" = pending in DB and NOT yet expired
                sqlx::query_as::<_, Invitation>(
                    r#"
                    SELECT id, tenant_id, email, role_ids, invited_by, token_hash, status, expires_at, accepted_at, created_at, updated_at
                    FROM invitations
                    WHERE tenant_id = ? AND status = 'pending' AND expires_at > NOW()
                    ORDER BY created_at DESC
                    LIMIT ? OFFSET ?
                    "#,
                )
                .bind(tenant_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            Some(ref s) => {
                sqlx::query_as::<_, Invitation>(
                    r#"
                    SELECT id, tenant_id, email, role_ids, invited_by, token_hash, status, expires_at, accepted_at, created_at, updated_at
                    FROM invitations
                    WHERE tenant_id = ? AND status = ?
                    ORDER BY created_at DESC
                    LIMIT ? OFFSET ?
                    "#,
                )
                .bind(tenant_id)
                .bind(s.to_string())
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, Invitation>(
                    r#"
                    SELECT id, tenant_id, email, role_ids, invited_by, token_hash, status, expires_at, accepted_at, created_at, updated_at
                    FROM invitations
                    WHERE tenant_id = ?
                    ORDER BY created_at DESC
                    LIMIT ? OFFSET ?
                    "#,
                )
                .bind(tenant_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(invitations)
    }

    async fn count_by_tenant(
        &self,
        tenant_id: StringUuid,
        status: Option<InvitationStatus>,
    ) -> Result<i64> {
        let row: (i64,) = match status {
            Some(InvitationStatus::Expired) => {
                sqlx::query_as("SELECT COUNT(*) FROM invitations WHERE tenant_id = ? AND (status = 'expired' OR (status = 'pending' AND expires_at <= NOW()))")
                    .bind(tenant_id)
                    .fetch_one(&self.pool)
                    .await?
            }
            Some(InvitationStatus::Pending) => {
                sqlx::query_as("SELECT COUNT(*) FROM invitations WHERE tenant_id = ? AND status = 'pending' AND expires_at > NOW()")
                    .bind(tenant_id)
                    .fetch_one(&self.pool)
                    .await?
            }
            Some(ref s) => {
                sqlx::query_as("SELECT COUNT(*) FROM invitations WHERE tenant_id = ? AND status = ?")
                    .bind(tenant_id)
                    .bind(s.to_string())
                    .fetch_one(&self.pool)
                    .await?
            }
            None => {
                sqlx::query_as("SELECT COUNT(*) FROM invitations WHERE tenant_id = ?")
                    .bind(tenant_id)
                    .fetch_one(&self.pool)
                    .await?
            }
        };
        Ok(row.0)
    }

    async fn list_pending(&self) -> Result<Vec<Invitation>> {
        let invitations = sqlx::query_as::<_, Invitation>(
            r#"
            SELECT id, tenant_id, email, role_ids, invited_by, token_hash, status, expires_at, accepted_at, created_at, updated_at
            FROM invitations
            WHERE status IN ('pending', 'revoked', 'accepted', 'expired')
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(invitations)
    }

    async fn update_status(&self, id: StringUuid, status: InvitationStatus) -> Result<Invitation> {
        sqlx::query(
            r#"
            UPDATE invitations
            SET status = ?, updated_at = NOW()
            WHERE id = ?
            "#,
        )
        .bind(status.to_string())
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Invitation {} not found", id)))
    }

    async fn mark_accepted(&self, id: StringUuid) -> Result<Invitation> {
        let result = sqlx::query(
            r#"
            UPDATE invitations
            SET status = 'accepted', accepted_at = NOW(), updated_at = NOW()
            WHERE id = ? AND status = 'pending'
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::Conflict(
                "Invitation has already been accepted or is no longer pending".to_string(),
            ));
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Invitation {} not found", id)))
    }

    async fn update_token_hash(&self, id: StringUuid, token_hash: &str) -> Result<Invitation> {
        let result = sqlx::query(
            r#"
            UPDATE invitations
            SET token_hash = ?, updated_at = NOW()
            WHERE id = ?
            "#,
        )
        .bind(token_hash)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Invitation {} not found", id)));
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Invitation {} not found", id)))
    }

    async fn touch_updated_at(&self, id: StringUuid) -> Result<Invitation> {
        let result = sqlx::query(
            r#"
            UPDATE invitations
            SET updated_at = NOW()
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Invitation {} not found", id)));
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Invitation {} not found", id)))
    }

    async fn delete(&self, id: StringUuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM invitations WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Invitation {} not found", id)));
        }

        Ok(())
    }

    async fn expire_pending(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE invitations
            SET status = 'expired', updated_at = NOW()
            WHERE status = 'pending' AND expires_at <= NOW()
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn delete_by_tenant(&self, tenant_id: StringUuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM invitations WHERE tenant_id = ?")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
