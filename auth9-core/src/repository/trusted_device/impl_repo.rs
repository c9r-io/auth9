//! TrustedDeviceRepository MySQL implementation

use super::{TrustedDeviceRepository, TrustedDeviceRepositoryImpl};
use crate::domains::identity::service::trusted_device::TrustedDevice;
use crate::error::Result;
use crate::models::common::StringUuid;
use async_trait::async_trait;

#[async_trait]
impl TrustedDeviceRepository for TrustedDeviceRepositoryImpl {
    async fn find_by_user_and_fingerprint(
        &self,
        user_id: StringUuid,
        fingerprint: &str,
    ) -> Result<Option<TrustedDevice>> {
        let device = sqlx::query_as::<_, TrustedDevice>(
            r#"
            SELECT id, user_id, tenant_id, device_fingerprint, device_name,
                   trusted_at, expires_at, last_used_at, revoked
            FROM trusted_devices
            WHERE user_id = ? AND device_fingerprint = ? AND revoked = FALSE AND expires_at > NOW()
            ORDER BY trusted_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(fingerprint)
        .fetch_optional(&self.pool)
        .await?;

        Ok(device)
    }

    async fn create(&self, device: &TrustedDevice) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO trusted_devices
                (id, user_id, tenant_id, device_fingerprint, device_name,
                 trusted_at, expires_at, last_used_at, revoked)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, FALSE)
            "#,
        )
        .bind(device.id)
        .bind(device.user_id)
        .bind(device.tenant_id)
        .bind(&device.device_fingerprint)
        .bind(&device.device_name)
        .bind(device.trusted_at)
        .bind(device.expires_at)
        .bind(device.last_used_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_by_user(&self, user_id: StringUuid) -> Result<Vec<TrustedDevice>> {
        let devices = sqlx::query_as::<_, TrustedDevice>(
            r#"
            SELECT id, user_id, tenant_id, device_fingerprint, device_name,
                   trusted_at, expires_at, last_used_at, revoked
            FROM trusted_devices
            WHERE user_id = ?
            ORDER BY trusted_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(devices)
    }

    async fn revoke(&self, device_id: StringUuid) -> Result<()> {
        sqlx::query("UPDATE trusted_devices SET revoked = TRUE WHERE id = ?")
            .bind(device_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn revoke_all_by_user(&self, user_id: StringUuid) -> Result<u64> {
        let result =
            sqlx::query("UPDATE trusted_devices SET revoked = TRUE WHERE user_id = ? AND revoked = FALSE")
                .bind(user_id)
                .execute(&self.pool)
                .await?;
        Ok(result.rows_affected())
    }

    async fn update_last_used(&self, device_id: StringUuid) -> Result<()> {
        sqlx::query("UPDATE trusted_devices SET last_used_at = NOW() WHERE id = ?")
            .bind(device_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_expired(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM trusted_devices WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    async fn delete_by_user(&self, user_id: StringUuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM trusted_devices WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
