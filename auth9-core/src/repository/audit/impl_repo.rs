//! impl AuditRepository for AuditRepositoryImpl

use super::{
    AuditLog, AuditLogQuery, AuditLogWithActor, AuditRepository, AuditRepositoryImpl,
    CreateAuditLogInput,
};
use crate::domain::StringUuid;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
impl AuditRepository for AuditRepositoryImpl {
    async fn create(&self, input: &CreateAuditLogInput) -> Result<()> {
        let old_value = input
            .old_value
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());
        let new_value = input
            .new_value
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());

        let actor_id = input.actor_id.map(|id| id.to_string());
        let resource_id = input.resource_id.map(|id| id.to_string());

        sqlx::query(
            r#"
            INSERT INTO audit_logs (actor_id, action, resource_type, resource_id, old_value, new_value, ip_address, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, NOW())
            "#,
        )
        .bind(actor_id)
        .bind(&input.action)
        .bind(&input.resource_type)
        .bind(resource_id)
        .bind(old_value)
        .bind(new_value)
        .bind(&input.ip_address)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find(&self, query: &AuditLogQuery) -> Result<Vec<AuditLog>> {
        let mut sql = String::from(
            "SELECT id, actor_id, action, resource_type, resource_id, old_value, new_value, ip_address, created_at FROM audit_logs WHERE 1=1",
        );

        if query.actor_id.is_some() {
            sql.push_str(" AND actor_id = ?");
        }
        if query.resource_type.is_some() {
            sql.push_str(" AND resource_type = ?");
        }
        if query.resource_id.is_some() {
            sql.push_str(" AND resource_id = ?");
        }
        if query.action.is_some() {
            sql.push_str(" AND action = ?");
        }
        if query.from_date.is_some() {
            sql.push_str(" AND created_at >= ?");
        }
        if query.to_date.is_some() {
            sql.push_str(" AND created_at <= ?");
        }

        sql.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        let mut query_builder = sqlx::query_as::<_, AuditLog>(&sql);

        if let Some(actor_id) = query.actor_id {
            query_builder = query_builder.bind(actor_id.to_string());
        }
        if let Some(ref resource_type) = query.resource_type {
            query_builder = query_builder.bind(resource_type);
        }
        if let Some(resource_id) = query.resource_id {
            query_builder = query_builder.bind(resource_id.to_string());
        }
        if let Some(ref action) = query.action {
            query_builder = query_builder.bind(action);
        }
        if let Some(from_date) = query.from_date {
            query_builder = query_builder.bind(from_date);
        }
        if let Some(to_date) = query.to_date {
            query_builder = query_builder.bind(to_date);
        }

        let limit = query.limit.unwrap_or(50).min(100);
        let offset = query.offset.unwrap_or(0);
        query_builder = query_builder.bind(limit).bind(offset);

        let logs = query_builder.fetch_all(&self.pool).await?;
        Ok(logs)
    }

    async fn find_with_actor(&self, query: &AuditLogQuery) -> Result<Vec<AuditLogWithActor>> {
        let mut sql = String::from(
            "SELECT al.id, al.actor_id, u.email as actor_email, u.display_name as actor_display_name, \
             al.action, al.resource_type, al.resource_id, al.old_value, al.new_value, al.ip_address, al.created_at \
             FROM audit_logs al \
             LEFT JOIN users u ON al.actor_id = u.id \
             WHERE 1=1",
        );

        if query.actor_id.is_some() {
            sql.push_str(" AND al.actor_id = ?");
        }
        if query.resource_type.is_some() {
            sql.push_str(" AND al.resource_type = ?");
        }
        if query.resource_id.is_some() {
            sql.push_str(" AND al.resource_id = ?");
        }
        if query.action.is_some() {
            sql.push_str(" AND al.action = ?");
        }
        if query.from_date.is_some() {
            sql.push_str(" AND al.created_at >= ?");
        }
        if query.to_date.is_some() {
            sql.push_str(" AND al.created_at <= ?");
        }

        sql.push_str(" ORDER BY al.created_at DESC LIMIT ? OFFSET ?");

        let mut query_builder = sqlx::query_as::<_, AuditLogWithActor>(&sql);

        if let Some(actor_id) = query.actor_id {
            query_builder = query_builder.bind(actor_id.to_string());
        }
        if let Some(ref resource_type) = query.resource_type {
            query_builder = query_builder.bind(resource_type);
        }
        if let Some(resource_id) = query.resource_id {
            query_builder = query_builder.bind(resource_id.to_string());
        }
        if let Some(ref action) = query.action {
            query_builder = query_builder.bind(action);
        }
        if let Some(from_date) = query.from_date {
            query_builder = query_builder.bind(from_date);
        }
        if let Some(to_date) = query.to_date {
            query_builder = query_builder.bind(to_date);
        }

        let limit = query.limit.unwrap_or(50).min(100);
        let offset = query.offset.unwrap_or(0);
        query_builder = query_builder.bind(limit).bind(offset);

        let logs = query_builder.fetch_all(&self.pool).await?;
        Ok(logs)
    }

    async fn count(&self, query: &AuditLogQuery) -> Result<i64> {
        let mut sql = String::from("SELECT COUNT(*) FROM audit_logs WHERE 1=1");

        if query.actor_id.is_some() {
            sql.push_str(" AND actor_id = ?");
        }
        if query.resource_type.is_some() {
            sql.push_str(" AND resource_type = ?");
        }
        if query.resource_id.is_some() {
            sql.push_str(" AND resource_id = ?");
        }
        if query.action.is_some() {
            sql.push_str(" AND action = ?");
        }
        if query.from_date.is_some() {
            sql.push_str(" AND created_at >= ?");
        }
        if query.to_date.is_some() {
            sql.push_str(" AND created_at <= ?");
        }

        let mut query_builder = sqlx::query_as::<_, (i64,)>(&sql);

        if let Some(actor_id) = query.actor_id {
            query_builder = query_builder.bind(actor_id.to_string());
        }
        if let Some(ref resource_type) = query.resource_type {
            query_builder = query_builder.bind(resource_type);
        }
        if let Some(resource_id) = query.resource_id {
            query_builder = query_builder.bind(resource_id.to_string());
        }
        if let Some(ref action) = query.action {
            query_builder = query_builder.bind(action);
        }
        if let Some(from_date) = query.from_date {
            query_builder = query_builder.bind(from_date);
        }
        if let Some(to_date) = query.to_date {
            query_builder = query_builder.bind(to_date);
        }

        let (count,) = query_builder.fetch_one(&self.pool).await?;
        Ok(count)
    }

    async fn nullify_actor_id(&self, actor_id: StringUuid) -> Result<u64> {
        let result = sqlx::query("UPDATE audit_logs SET actor_id = NULL WHERE actor_id = ?")
            .bind(actor_id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
