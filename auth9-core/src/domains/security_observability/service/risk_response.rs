//! Risk response service — evaluates automated response actions based on risk assessment

use super::risk_engine::{RiskAction, RiskAssessment, RiskLevel};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Tenant-level risk policy configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TenantRiskPolicy {
    pub tenant_id: String,
    /// Score threshold above which MFA step-up is required (default 51)
    pub mfa_threshold: u8,
    /// Score threshold above which login is blocked (default 76)
    pub block_threshold: u8,
    /// Whether to notify admin on high-risk logins
    pub notify_admin: bool,
    /// Whether to auto-lock account on critical-risk logins
    pub auto_lock_account: bool,
}

impl TenantRiskPolicy {
    pub fn default_for_tenant(tenant_id: &str) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            mfa_threshold: 51,
            block_threshold: 76,
            notify_admin: true,
            auto_lock_account: false,
        }
    }
}

/// Risk response service that maps risk assessments to concrete actions
pub struct RiskResponseService;

impl RiskResponseService {
    /// Evaluate the appropriate response action given a risk assessment and tenant policy.
    pub fn evaluate_response(assessment: &RiskAssessment, policy: &TenantRiskPolicy) -> RiskAction {
        if assessment.score >= policy.block_threshold {
            RiskAction::Block
        } else if assessment.score >= policy.mfa_threshold {
            RiskAction::StepUpMfa
        } else if assessment.level == RiskLevel::Medium {
            RiskAction::Alert
        } else {
            RiskAction::Allow
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_assessment(score: u8) -> RiskAssessment {
        RiskAssessment {
            score,
            level: RiskLevel::from_score(score),
            factors: vec![],
            recommended_action: RiskAction::Allow, // will be overridden by evaluate_response
            assessed_at: Utc::now(),
        }
    }

    fn default_policy() -> TenantRiskPolicy {
        TenantRiskPolicy::default_for_tenant("test-tenant")
    }

    #[test]
    fn test_low_risk_allows() {
        let assessment = make_assessment(10);
        let action = RiskResponseService::evaluate_response(&assessment, &default_policy());
        assert_eq!(action, RiskAction::Allow);
    }

    #[test]
    fn test_medium_risk_alerts() {
        let assessment = make_assessment(30);
        let action = RiskResponseService::evaluate_response(&assessment, &default_policy());
        assert_eq!(action, RiskAction::Alert);
    }

    #[test]
    fn test_high_risk_requires_mfa() {
        let assessment = make_assessment(51);
        let action = RiskResponseService::evaluate_response(&assessment, &default_policy());
        assert_eq!(action, RiskAction::StepUpMfa);
    }

    #[test]
    fn test_critical_risk_blocks() {
        let assessment = make_assessment(76);
        let action = RiskResponseService::evaluate_response(&assessment, &default_policy());
        assert_eq!(action, RiskAction::Block);
    }

    #[test]
    fn test_custom_thresholds() {
        let policy = TenantRiskPolicy {
            tenant_id: "custom".to_string(),
            mfa_threshold: 30,
            block_threshold: 60,
            notify_admin: true,
            auto_lock_account: false,
        };

        // Score 30 should trigger MFA with custom threshold
        let assessment = make_assessment(30);
        let action = RiskResponseService::evaluate_response(&assessment, &policy);
        assert_eq!(action, RiskAction::StepUpMfa);

        // Score 60 should block with custom threshold
        let assessment = make_assessment(60);
        let action = RiskResponseService::evaluate_response(&assessment, &policy);
        assert_eq!(action, RiskAction::Block);
    }

    #[test]
    fn test_boundary_values() {
        let policy = default_policy(); // mfa=51, block=76

        // Just below MFA threshold
        let assessment = make_assessment(50);
        let action = RiskResponseService::evaluate_response(&assessment, &policy);
        assert_eq!(action, RiskAction::Alert); // Medium range

        // Exactly at MFA threshold
        let assessment = make_assessment(51);
        let action = RiskResponseService::evaluate_response(&assessment, &policy);
        assert_eq!(action, RiskAction::StepUpMfa);

        // Just below block threshold
        let assessment = make_assessment(75);
        let action = RiskResponseService::evaluate_response(&assessment, &policy);
        assert_eq!(action, RiskAction::StepUpMfa);

        // Exactly at block threshold
        let assessment = make_assessment(76);
        let action = RiskResponseService::evaluate_response(&assessment, &policy);
        assert_eq!(action, RiskAction::Block);
    }
}
