//! impl SecurityAlertRepository for SecurityAlertRepositoryImpl

use super::{SecurityAlertRepository, SecurityAlertRepositoryImpl};
use crate::error::{AppError, Result};
use crate::models::analytics::{
    AlertSeverity, CreateSecurityAlertInput, SecurityAlert, SecurityAlertType,
};
use crate::models::common::StringUuid;
use async_trait::async_trait;

#[async_trait]
impl SecurityAlertRepository for SecurityAlertRepositoryImpl {
    async fn create(&self, input: &CreateSecurityAlertInput) -> Result<SecurityAlert> {
        let id = StringUuid::new_v4();
        let details_json = input
            .details
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| AppError::Internal(e.into()))?;

        sqlx::query(
            r#"
            INSERT INTO security_alerts (id, user_id, tenant_id, alert_type, severity,
                                         details, created_at)
            VALUES (?, ?, ?, ?, ?, ?, NOW())
            "#,
        )
        .bind(id)
        .bind(input.user_id)
        .bind(input.tenant_id)
        .bind(&input.alert_type)
        .bind(&input.severity)
        .bind(&details_json)
        .execute(&self.pool)
        .await?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Failed to create security alert")))
    }

    async fn find_by_id(&self, id: StringUuid) -> Result<Option<SecurityAlert>> {
        let alert = sqlx::query_as::<_, SecurityAlert>(
            r#"
            SELECT id, user_id, tenant_id, alert_type, severity, details,
                   resolved_at, resolved_by, created_at
            FROM security_alerts
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(alert)
    }

    async fn list(&self, offset: i64, limit: i64) -> Result<Vec<SecurityAlert>> {
        let alerts = sqlx::query_as::<_, SecurityAlert>(
            r#"
            SELECT id, user_id, tenant_id, alert_type, severity, details,
                   resolved_at, resolved_by, created_at
            FROM security_alerts
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(alerts)
    }

    async fn list_unresolved(&self, offset: i64, limit: i64) -> Result<Vec<SecurityAlert>> {
        let alerts = sqlx::query_as::<_, SecurityAlert>(
            r#"
            SELECT id, user_id, tenant_id, alert_type, severity, details,
                   resolved_at, resolved_by, created_at
            FROM security_alerts
            WHERE resolved_at IS NULL
            ORDER BY
                CASE severity
                    WHEN 'critical' THEN 1
                    WHEN 'high' THEN 2
                    WHEN 'medium' THEN 3
                    WHEN 'low' THEN 4
                END,
                created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(alerts)
    }

    async fn list_by_user(
        &self,
        user_id: StringUuid,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<SecurityAlert>> {
        let alerts = sqlx::query_as::<_, SecurityAlert>(
            r#"
            SELECT id, user_id, tenant_id, alert_type, severity, details,
                   resolved_at, resolved_by, created_at
            FROM security_alerts
            WHERE user_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(alerts)
    }

    async fn list_by_severity(
        &self,
        severity: AlertSeverity,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<SecurityAlert>> {
        let alerts = sqlx::query_as::<_, SecurityAlert>(
            r#"
            SELECT id, user_id, tenant_id, alert_type, severity, details,
                   resolved_at, resolved_by, created_at
            FROM security_alerts
            WHERE severity = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(&severity)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(alerts)
    }

    async fn list_filtered(
        &self,
        offset: i64,
        limit: i64,
        resolved_filter: Option<bool>,
        severity: Option<AlertSeverity>,
        alert_type: Option<SecurityAlertType>,
    ) -> Result<Vec<SecurityAlert>> {
        let mut sql = String::from(
            "SELECT id, user_id, tenant_id, alert_type, severity, details, \
             resolved_at, resolved_by, created_at FROM security_alerts WHERE 1=1",
        );
        match resolved_filter {
            Some(true) => sql.push_str(" AND resolved_at IS NOT NULL"),
            Some(false) => sql.push_str(" AND resolved_at IS NULL"),
            None => {} // no filter
        }
        if severity.is_some() {
            sql.push_str(" AND severity = ?");
        }
        if alert_type.is_some() {
            sql.push_str(" AND alert_type = ?");
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        let mut query = sqlx::query_as::<_, SecurityAlert>(&sql);
        if let Some(ref s) = severity {
            query = query.bind(s);
        }
        if let Some(ref t) = alert_type {
            query = query.bind(t);
        }
        query = query.bind(limit).bind(offset);

        let alerts = query.fetch_all(&self.pool).await?;
        Ok(alerts)
    }

    async fn count_filtered(
        &self,
        resolved_filter: Option<bool>,
        severity: Option<AlertSeverity>,
        alert_type: Option<SecurityAlertType>,
    ) -> Result<i64> {
        let mut sql = String::from("SELECT COUNT(*) FROM security_alerts WHERE 1=1");
        match resolved_filter {
            Some(true) => sql.push_str(" AND resolved_at IS NOT NULL"),
            Some(false) => sql.push_str(" AND resolved_at IS NULL"),
            None => {}
        }
        if severity.is_some() {
            sql.push_str(" AND severity = ?");
        }
        if alert_type.is_some() {
            sql.push_str(" AND alert_type = ?");
        }

        let mut query = sqlx::query_as::<_, (i64,)>(&sql);
        if let Some(ref s) = severity {
            query = query.bind(s);
        }
        if let Some(ref t) = alert_type {
            query = query.bind(t);
        }

        let row = query.fetch_one(&self.pool).await?;
        Ok(row.0)
    }

    async fn count(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM security_alerts")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    async fn count_unresolved(&self) -> Result<i64> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM security_alerts WHERE resolved_at IS NULL")
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0)
    }

    async fn resolve(&self, id: StringUuid, resolved_by: StringUuid) -> Result<SecurityAlert> {
        let result = sqlx::query(
            r#"
            UPDATE security_alerts
            SET resolved_at = NOW(), resolved_by = ?
            WHERE id = ? AND resolved_at IS NULL
            "#,
        )
        .bind(resolved_by)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                "Security alert not found or already resolved".to_string(),
            ));
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Failed to resolve security alert")))
    }

    async fn delete_old(&self, days: i64) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM security_alerts
            WHERE resolved_at IS NOT NULL
              AND created_at < DATE_SUB(NOW(), INTERVAL ? DAY)
            "#,
        )
        .bind(days)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn nullify_user_id(&self, user_id: StringUuid) -> Result<u64> {
        let result = sqlx::query("UPDATE security_alerts SET user_id = NULL WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    async fn delete_by_tenant(&self, tenant_id: StringUuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM security_alerts WHERE tenant_id = ?")
            .bind(tenant_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
