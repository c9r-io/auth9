//! Risk policy and risk analytics API handlers

use crate::domains::security_observability::service::risk_response::TenantRiskPolicy;
use crate::error::AppError;
use crate::http_support::SuccessResponse;
use crate::middleware::auth::AuthUser;
use crate::models::common::StringUuid;
use crate::policy::{enforce, PolicyAction, PolicyInput, ResourceScope};
use crate::repository::tenant_risk_policy::{TenantRiskPolicyRepository, TenantRiskPolicyRow};
use crate::state::HasServices;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Well-known nil UUID used as the platform-default risk policy tenant key
/// (for Identity tokens that don't carry a tenant_id).
const PLATFORM_DEFAULT_TENANT: Uuid = Uuid::nil();

/// Trait for state that provides risk policy repository access
pub trait HasRiskPolicy: HasServices {
    type RiskPolicyRepo: TenantRiskPolicyRepository;
    fn risk_policy_repo(&self) -> &Self::RiskPolicyRepo;
}

/// Get tenant risk policy
pub async fn get_risk_policy<S: HasRiskPolicy>(
    State(state): State<S>,
    auth: AuthUser,
) -> Result<Json<SuccessResponse<TenantRiskPolicy>>, AppError> {
    enforce(
        state.config(),
        &auth,
        &PolicyInput {
            action: PolicyAction::SecurityAlertRead,
            scope: ResourceScope::Global,
        },
    )?;

    let tenant_uuid = auth.tenant_id.unwrap_or(PLATFORM_DEFAULT_TENANT);
    let tenant_id = StringUuid::from(tenant_uuid);

    let policy = match state
        .risk_policy_repo()
        .find_by_tenant_id(tenant_id)
        .await?
    {
        Some(row) => TenantRiskPolicy {
            tenant_id: row.tenant_id.to_string(),
            mfa_threshold: row.mfa_threshold,
            block_threshold: row.block_threshold,
            notify_admin: row.notify_admin,
            auto_lock_account: row.auto_lock_account,
        },
        None => TenantRiskPolicy::default_for_tenant(&tenant_id.to_string()),
    };

    Ok(Json(SuccessResponse::new(policy)))
}

/// Update tenant risk policy request
#[derive(Debug, Deserialize)]
pub struct UpdateRiskPolicyRequest {
    pub mfa_threshold: Option<u8>,
    pub block_threshold: Option<u8>,
    pub notify_admin: Option<bool>,
    pub auto_lock_account: Option<bool>,
}

/// Update tenant risk policy
pub async fn update_risk_policy<S: HasRiskPolicy>(
    State(state): State<S>,
    auth: AuthUser,
    Json(body): Json<UpdateRiskPolicyRequest>,
) -> Result<Json<SuccessResponse<TenantRiskPolicy>>, AppError> {
    enforce(
        state.config(),
        &auth,
        &PolicyInput {
            action: PolicyAction::SecurityAlertResolve, // write permission
            scope: ResourceScope::Global,
        },
    )?;

    let tenant_id = StringUuid::from(auth.tenant_id.unwrap_or(PLATFORM_DEFAULT_TENANT));

    // Get existing or defaults
    let existing = state
        .risk_policy_repo()
        .find_by_tenant_id(tenant_id)
        .await?;

    let row = TenantRiskPolicyRow {
        id: existing
            .as_ref()
            .map(|r| r.id)
            .unwrap_or_else(StringUuid::new_v4),
        tenant_id,
        mfa_threshold: body
            .mfa_threshold
            .unwrap_or(existing.as_ref().map(|r| r.mfa_threshold).unwrap_or(51)),
        block_threshold: body
            .block_threshold
            .unwrap_or(existing.as_ref().map(|r| r.block_threshold).unwrap_or(76)),
        notify_admin: body
            .notify_admin
            .unwrap_or(existing.as_ref().map(|r| r.notify_admin).unwrap_or(true)),
        auto_lock_account: body.auto_lock_account.unwrap_or(
            existing
                .as_ref()
                .map(|r| r.auto_lock_account)
                .unwrap_or(false),
        ),
        created_at: existing
            .as_ref()
            .map(|r| r.created_at)
            .unwrap_or_else(chrono::Utc::now),
        updated_at: chrono::Utc::now(),
    };

    state.risk_policy_repo().upsert(&row).await?;

    let policy = TenantRiskPolicy {
        tenant_id: row.tenant_id.to_string(),
        mfa_threshold: row.mfa_threshold,
        block_threshold: row.block_threshold,
        notify_admin: row.notify_admin,
        auto_lock_account: row.auto_lock_account,
    };

    Ok(Json(SuccessResponse::new(policy)))
}

/// Risk trend data point
#[derive(Debug, Serialize)]
pub struct RiskTrendPoint {
    pub date: String,
    pub avg_risk_score: f64,
    pub high_risk_count: i64,
    pub total_logins: i64,
}

/// High-risk user entry
#[derive(Debug, Serialize)]
pub struct HighRiskUser {
    pub user_id: String,
    pub email: Option<String>,
    pub avg_risk_score: f64,
    pub login_count: i64,
}

/// Geo distribution entry
#[derive(Debug, Serialize)]
pub struct GeoDistributionEntry {
    pub country_code: String,
    pub login_count: i64,
}
