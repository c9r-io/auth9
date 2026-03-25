//! TenantRiskPolicyRepository MySQL implementation

use super::{TenantRiskPolicyRepository, TenantRiskPolicyRepositoryImpl, TenantRiskPolicyRow};
use crate::error::Result;
use crate::models::common::StringUuid;
use async_trait::async_trait;

#[async_trait]
impl TenantRiskPolicyRepository for TenantRiskPolicyRepositoryImpl {
    async fn find_by_tenant_id(
        &self,
        tenant_id: StringUuid,
    ) -> Result<Option<TenantRiskPolicyRow>> {
        let row = sqlx::query_as::<_, TenantRiskPolicyRow>(
            r#"
            SELECT id, tenant_id, mfa_threshold, block_threshold,
                   notify_admin, auto_lock_account, created_at, updated_at
            FROM tenant_risk_policies
            WHERE tenant_id = ?
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn upsert(&self, row: &TenantRiskPolicyRow) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO tenant_risk_policies
                (id, tenant_id, mfa_threshold, block_threshold,
                 notify_admin, auto_lock_account, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, NOW(), NOW())
            ON DUPLICATE KEY UPDATE
                mfa_threshold = VALUES(mfa_threshold),
                block_threshold = VALUES(block_threshold),
                notify_admin = VALUES(notify_admin),
                auto_lock_account = VALUES(auto_lock_account),
                updated_at = NOW()
            "#,
        )
        .bind(row.id)
        .bind(row.tenant_id)
        .bind(row.mfa_threshold)
        .bind(row.block_threshold)
        .bind(row.notify_admin)
        .bind(row.auto_lock_account)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_by_tenant_id(&self, tenant_id: StringUuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM tenant_risk_policies WHERE tenant_id = ?")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
