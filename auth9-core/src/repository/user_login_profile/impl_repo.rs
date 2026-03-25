//! UserLoginProfileRepository MySQL implementation

use super::{UserLoginProfileRepository, UserLoginProfileRepositoryImpl};
use crate::domains::security_observability::service::user_profile::UserLoginProfile;
use crate::error::Result;
use crate::models::common::StringUuid;
use async_trait::async_trait;

#[async_trait]
impl UserLoginProfileRepository for UserLoginProfileRepositoryImpl {
    async fn find_by_user_id(&self, user_id: StringUuid) -> Result<Option<UserLoginProfile>> {
        let profile = sqlx::query_as::<_, UserLoginProfile>(
            r#"
            SELECT id, user_id, known_ips, known_devices, known_countries,
                   typical_login_hours, avg_login_frequency, total_logins,
                   last_updated, created_at
            FROM user_login_profiles
            WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(profile)
    }

    async fn upsert(&self, profile: &UserLoginProfile) -> Result<()> {
        let known_ips =
            serde_json::to_string(&profile.known_ips).map_err(|e| anyhow::anyhow!(e))?;
        let known_devices =
            serde_json::to_string(&profile.known_devices).map_err(|e| anyhow::anyhow!(e))?;
        let known_countries =
            serde_json::to_string(&profile.known_countries).map_err(|e| anyhow::anyhow!(e))?;
        let typical_login_hours =
            serde_json::to_string(&profile.typical_login_hours).map_err(|e| anyhow::anyhow!(e))?;

        sqlx::query(
            r#"
            INSERT INTO user_login_profiles
                (id, user_id, known_ips, known_devices, known_countries,
                 typical_login_hours, avg_login_frequency, total_logins,
                 last_updated, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, NOW(), NOW())
            ON DUPLICATE KEY UPDATE
                known_ips = VALUES(known_ips),
                known_devices = VALUES(known_devices),
                known_countries = VALUES(known_countries),
                typical_login_hours = VALUES(typical_login_hours),
                avg_login_frequency = VALUES(avg_login_frequency),
                total_logins = VALUES(total_logins),
                last_updated = NOW()
            "#,
        )
        .bind(profile.id)
        .bind(profile.user_id)
        .bind(&known_ips)
        .bind(&known_devices)
        .bind(&known_countries)
        .bind(&typical_login_hours)
        .bind(profile.avg_login_frequency)
        .bind(profile.total_logins)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_by_user_id(&self, user_id: StringUuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM user_login_profiles WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
