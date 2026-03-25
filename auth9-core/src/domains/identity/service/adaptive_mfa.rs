//! Adaptive MFA policy engine — risk-driven MFA decision making

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// MFA enforcement mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdaptiveMfaMode {
    /// Never require MFA
    Disabled,
    /// Always require MFA when enrolled (backward compatible)
    Always,
    /// Risk-driven: require MFA only when risk score exceeds threshold
    Adaptive,
    /// Encourage MFA enrollment but don't enforce
    OptionalEnroll,
}

impl std::str::FromStr for AdaptiveMfaMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "disabled" => Ok(Self::Disabled),
            "always" => Ok(Self::Always),
            "adaptive" => Ok(Self::Adaptive),
            "optional_enroll" => Ok(Self::OptionalEnroll),
            _ => Err(format!("Unknown MFA mode: {}", s)),
        }
    }
}

impl std::fmt::Display for AdaptiveMfaMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => write!(f, "disabled"),
            Self::Always => write!(f, "always"),
            Self::Adaptive => write!(f, "adaptive"),
            Self::OptionalEnroll => write!(f, "optional_enroll"),
        }
    }
}

/// Per-tenant adaptive MFA policy
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdaptiveMfaPolicy {
    pub tenant_id: String,
    pub mode: AdaptiveMfaMode,
    /// Risk score threshold above which MFA is required (default 40)
    pub risk_threshold: u8,
    /// Always require MFA for admin users (default true)
    pub always_require_for_admins: bool,
    /// Days to trust a device after MFA verification (default 30)
    pub trust_device_days: u16,
    /// Operations requiring step-up authentication
    pub step_up_operations: Vec<String>,
}

impl AdaptiveMfaPolicy {
    pub fn default_for_tenant(tenant_id: &str) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            mode: AdaptiveMfaMode::Always,
            risk_threshold: 40,
            always_require_for_admins: true,
            trust_device_days: 30,
            step_up_operations: vec![
                "change_password".to_string(),
                "modify_security_settings".to_string(),
            ],
        }
    }
}

/// MFA decision result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MfaDecision {
    /// Low risk — skip MFA
    Skip,
    /// MFA required with available methods and reason
    Required {
        methods: Vec<String>,
        reason: String,
    },
}

/// Input for MFA decision evaluation
pub struct MfaEvaluationInput {
    pub risk_score: u8,
    pub is_admin: bool,
    pub device_trusted: bool,
    pub has_mfa_enrolled: bool,
    pub mfa_methods: Vec<String>,
}

/// Stateless adaptive MFA decision engine
pub struct AdaptiveMfaEngine;

impl AdaptiveMfaEngine {
    /// Evaluate whether MFA should be required for this login attempt.
    pub fn evaluate(policy: &AdaptiveMfaPolicy, input: &MfaEvaluationInput) -> MfaDecision {
        // If user has no MFA enrolled, can't require it
        if !input.has_mfa_enrolled {
            return MfaDecision::Skip;
        }

        match policy.mode {
            AdaptiveMfaMode::Disabled => MfaDecision::Skip,

            AdaptiveMfaMode::Always => MfaDecision::Required {
                methods: input.mfa_methods.clone(),
                reason: "MFA always required".to_string(),
            },

            AdaptiveMfaMode::Adaptive => {
                // Admin override: always require MFA for admins
                if input.is_admin && policy.always_require_for_admins {
                    return MfaDecision::Required {
                        methods: input.mfa_methods.clone(),
                        reason: "Admin policy: MFA always required for administrators".to_string(),
                    };
                }

                // Trusted device with low risk: skip
                if input.device_trusted && input.risk_score < policy.risk_threshold {
                    return MfaDecision::Skip;
                }

                // High risk: require MFA
                if input.risk_score >= policy.risk_threshold {
                    return MfaDecision::Required {
                        methods: input.mfa_methods.clone(),
                        reason: format!(
                            "Risk score {} exceeds threshold {}",
                            input.risk_score, policy.risk_threshold
                        ),
                    };
                }

                // Low risk, untrusted device but below threshold: skip
                MfaDecision::Skip
            }

            AdaptiveMfaMode::OptionalEnroll => MfaDecision::Skip,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_policy() -> AdaptiveMfaPolicy {
        AdaptiveMfaPolicy::default_for_tenant("test-tenant")
    }

    fn base_input() -> MfaEvaluationInput {
        MfaEvaluationInput {
            risk_score: 20,
            is_admin: false,
            device_trusted: false,
            has_mfa_enrolled: true,
            mfa_methods: vec!["totp".to_string()],
        }
    }

    #[test]
    fn test_disabled_mode_always_skips() {
        let mut policy = default_policy();
        policy.mode = AdaptiveMfaMode::Disabled;
        let input = base_input();
        assert_eq!(AdaptiveMfaEngine::evaluate(&policy, &input), MfaDecision::Skip);
    }

    #[test]
    fn test_always_mode_always_requires() {
        let policy = default_policy(); // default is Always
        let input = base_input();
        assert!(matches!(
            AdaptiveMfaEngine::evaluate(&policy, &input),
            MfaDecision::Required { .. }
        ));
    }

    #[test]
    fn test_always_mode_skips_when_no_mfa_enrolled() {
        let policy = default_policy();
        let mut input = base_input();
        input.has_mfa_enrolled = false;
        assert_eq!(AdaptiveMfaEngine::evaluate(&policy, &input), MfaDecision::Skip);
    }

    #[test]
    fn test_adaptive_low_risk_trusted_device_skips() {
        let mut policy = default_policy();
        policy.mode = AdaptiveMfaMode::Adaptive;
        policy.risk_threshold = 40;
        let mut input = base_input();
        input.risk_score = 10;
        input.device_trusted = true;
        assert_eq!(AdaptiveMfaEngine::evaluate(&policy, &input), MfaDecision::Skip);
    }

    #[test]
    fn test_adaptive_high_risk_requires_mfa() {
        let mut policy = default_policy();
        policy.mode = AdaptiveMfaMode::Adaptive;
        policy.risk_threshold = 40;
        let mut input = base_input();
        input.risk_score = 50;
        let decision = AdaptiveMfaEngine::evaluate(&policy, &input);
        assert!(matches!(decision, MfaDecision::Required { .. }));
        if let MfaDecision::Required { reason, .. } = decision {
            assert!(reason.contains("50"));
        }
    }

    #[test]
    fn test_adaptive_admin_always_requires() {
        let mut policy = default_policy();
        policy.mode = AdaptiveMfaMode::Adaptive;
        policy.always_require_for_admins = true;
        let mut input = base_input();
        input.is_admin = true;
        input.risk_score = 5; // low risk
        input.device_trusted = true;
        let decision = AdaptiveMfaEngine::evaluate(&policy, &input);
        assert!(matches!(decision, MfaDecision::Required { .. }));
        if let MfaDecision::Required { reason, .. } = decision {
            assert!(reason.contains("Admin"));
        }
    }

    #[test]
    fn test_adaptive_admin_skips_when_flag_disabled() {
        let mut policy = default_policy();
        policy.mode = AdaptiveMfaMode::Adaptive;
        policy.always_require_for_admins = false;
        policy.risk_threshold = 40;
        let mut input = base_input();
        input.is_admin = true;
        input.risk_score = 5;
        assert_eq!(AdaptiveMfaEngine::evaluate(&policy, &input), MfaDecision::Skip);
    }

    #[test]
    fn test_adaptive_boundary_at_threshold() {
        let mut policy = default_policy();
        policy.mode = AdaptiveMfaMode::Adaptive;
        policy.risk_threshold = 40;

        // Exactly at threshold: required
        let mut input = base_input();
        input.risk_score = 40;
        assert!(matches!(
            AdaptiveMfaEngine::evaluate(&policy, &input),
            MfaDecision::Required { .. }
        ));

        // Just below threshold: skip
        input.risk_score = 39;
        assert_eq!(AdaptiveMfaEngine::evaluate(&policy, &input), MfaDecision::Skip);
    }

    #[test]
    fn test_optional_enroll_skips() {
        let mut policy = default_policy();
        policy.mode = AdaptiveMfaMode::OptionalEnroll;
        let input = base_input();
        assert_eq!(AdaptiveMfaEngine::evaluate(&policy, &input), MfaDecision::Skip);
    }

    #[test]
    fn test_mode_serialization() {
        assert_eq!(AdaptiveMfaMode::Adaptive.to_string(), "adaptive");
        assert_eq!("adaptive".parse::<AdaptiveMfaMode>().unwrap(), AdaptiveMfaMode::Adaptive);
        assert_eq!("always".parse::<AdaptiveMfaMode>().unwrap(), AdaptiveMfaMode::Always);
    }
}
