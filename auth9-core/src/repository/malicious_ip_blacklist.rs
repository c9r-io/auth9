use crate::error::Result;
use crate::models::common::StringUuid;
use crate::models::system_settings::{MaliciousIpBlacklistEntry, TenantMaliciousIpBlacklistEntry};
use async_trait::async_trait;
use sqlx::MySqlPool;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait MaliciousIpBlacklistRepository: Send + Sync {
    async fn list(&self) -> Result<Vec<MaliciousIpBlacklistEntry>>;
    async fn list_by_tenant(
        &self,
        tenant_id: StringUuid,
    ) -> Result<Vec<TenantMaliciousIpBlacklistEntry>>;
    async fn replace_all(
        &self,
        entries: &[MaliciousIpBlacklistEntry],
        created_by: Option<StringUuid>,
    ) -> Result<Vec<MaliciousIpBlacklistEntry>>;
    async fn replace_all_for_tenant(
        &self,
        tenant_id: StringUuid,
        entries: &[TenantMaliciousIpBlacklistEntry],
        created_by: Option<StringUuid>,
    ) -> Result<Vec<TenantMaliciousIpBlacklistEntry>>;
    async fn find_by_ip(&self, ip_address: &str) -> Result<Option<MaliciousIpBlacklistEntry>>;
    async fn find_by_ip_in_tenant(
        &self,
        tenant_id: StringUuid,
        ip_address: &str,
    ) -> Result<Option<TenantMaliciousIpBlacklistEntry>>;
}

pub struct MaliciousIpBlacklistRepositoryImpl {
    pool: MySqlPool,
}

impl MaliciousIpBlacklistRepositoryImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MaliciousIpBlacklistRepository for MaliciousIpBlacklistRepositoryImpl {
    async fn list(&self) -> Result<Vec<MaliciousIpBlacklistEntry>> {
        let rows = sqlx::query_as::<_, MaliciousIpBlacklistEntry>(
            r#"
            SELECT id, ip_address, reason, created_by, created_at, updated_at
            FROM malicious_ip_blacklist
            ORDER BY ip_address ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn list_by_tenant(
        &self,
        tenant_id: StringUuid,
    ) -> Result<Vec<TenantMaliciousIpBlacklistEntry>> {
        let rows = sqlx::query_as::<_, TenantMaliciousIpBlacklistEntry>(
            r#"
            SELECT id, tenant_id, ip_address, reason, created_by, created_at, updated_at
            FROM tenant_malicious_ip_blacklist
            WHERE tenant_id = ?
            ORDER BY ip_address ASC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn replace_all(
        &self,
        entries: &[MaliciousIpBlacklistEntry],
        created_by: Option<StringUuid>,
    ) -> Result<Vec<MaliciousIpBlacklistEntry>> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM malicious_ip_blacklist")
            .execute(&mut *tx)
            .await?;

        for entry in entries {
            sqlx::query(
                r#"
                INSERT INTO malicious_ip_blacklist (id, ip_address, reason, created_by, created_at, updated_at)
                VALUES (?, ?, ?, ?, NOW(), NOW())
                "#,
            )
            .bind(entry.id)
            .bind(&entry.ip_address)
            .bind(&entry.reason)
            .bind(created_by)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        self.list().await
    }

    async fn replace_all_for_tenant(
        &self,
        tenant_id: StringUuid,
        entries: &[TenantMaliciousIpBlacklistEntry],
        created_by: Option<StringUuid>,
    ) -> Result<Vec<TenantMaliciousIpBlacklistEntry>> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM tenant_malicious_ip_blacklist WHERE tenant_id = ?")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        for entry in entries {
            sqlx::query(
                r#"
                INSERT INTO tenant_malicious_ip_blacklist
                    (id, tenant_id, ip_address, reason, created_by, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, NOW(), NOW())
                "#,
            )
            .bind(entry.id)
            .bind(entry.tenant_id)
            .bind(&entry.ip_address)
            .bind(&entry.reason)
            .bind(created_by)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        self.list_by_tenant(tenant_id).await
    }

    async fn find_by_ip(&self, ip_address: &str) -> Result<Option<MaliciousIpBlacklistEntry>> {
        let row = sqlx::query_as::<_, MaliciousIpBlacklistEntry>(
            r#"
            SELECT id, ip_address, reason, created_by, created_at, updated_at
            FROM malicious_ip_blacklist
            WHERE ip_address = ?
            LIMIT 1
            "#,
        )
        .bind(ip_address)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    async fn find_by_ip_in_tenant(
        &self,
        tenant_id: StringUuid,
        ip_address: &str,
    ) -> Result<Option<TenantMaliciousIpBlacklistEntry>> {
        let row = sqlx::query_as::<_, TenantMaliciousIpBlacklistEntry>(
            r#"
            SELECT id, tenant_id, ip_address, reason, created_by, created_at, updated_at
            FROM tenant_malicious_ip_blacklist
            WHERE tenant_id = ? AND ip_address = ?
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .bind(ip_address)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }
}
