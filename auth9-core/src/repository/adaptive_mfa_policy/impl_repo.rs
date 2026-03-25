//! AdaptiveMfaPolicyRepository MySQL implementation

use super::{AdaptiveMfaPolicyRepository, AdaptiveMfaPolicyRepositoryImpl, AdaptiveMfaPolicyRow};
use crate::error::Result;
use crate::models::common::StringUuid;
use async_trait::async_trait;

#[async_trait]
impl AdaptiveMfaPolicyRepository for AdaptiveMfaPolicyRepositoryImpl {
    async fn find_by_tenant_id(
        &self,
        tenant_id: StringUuid,
    ) -> Result<Option<AdaptiveMfaPolicyRow>> {
        let row = sqlx::query_as::<_, AdaptiveMfaPolicyRow>(
            r#"
            SELECT id, tenant_id, mode, risk_threshold, always_require_for_admins,
                   trust_device_days, step_up_operations, created_at, updated_at
            FROM adaptive_mfa_policies
            WHERE tenant_id = ?
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn upsert(&self, row: &AdaptiveMfaPolicyRow) -> Result<()> {
        let step_up_json =
            serde_json::to_string(&row.step_up_operations).map_err(|e| anyhow::anyhow!(e))?;

        sqlx::query(
            r#"
            INSERT INTO adaptive_mfa_policies
                (id, tenant_id, mode, risk_threshold, always_require_for_admins,
                 trust_device_days, step_up_operations, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, NOW(), NOW())
            ON DUPLICATE KEY UPDATE
                mode = VALUES(mode),
                risk_threshold = VALUES(risk_threshold),
                always_require_for_admins = VALUES(always_require_for_admins),
                trust_device_days = VALUES(trust_device_days),
                step_up_operations = VALUES(step_up_operations),
                updated_at = NOW()
            "#,
        )
        .bind(row.id)
        .bind(row.tenant_id)
        .bind(&row.mode)
        .bind(row.risk_threshold)
        .bind(row.always_require_for_admins)
        .bind(row.trust_device_days)
        .bind(&step_up_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_by_tenant_id(&self, tenant_id: StringUuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM adaptive_mfa_policies WHERE tenant_id = ?")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
