//! Risk scoring engine — computes a 0-100 risk score for each login attempt

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::user_profile::UserLoginProfile;

/// Risk level derived from the numeric score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,      // 0-25
    Medium,   // 26-50
    High,     // 51-75
    Critical, // 76-100
}

impl RiskLevel {
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=25 => RiskLevel::Low,
            26..=50 => RiskLevel::Medium,
            51..=75 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }
}

/// Recommended action based on risk assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RiskAction {
    Allow,
    Alert,
    StepUpMfa,
    Block,
}

/// Individual risk factor contributing to the overall score
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RiskFactor {
    pub name: String,
    pub score: u8,
    pub weight: f64,
    pub detail: String,
}

/// Complete risk assessment for a login attempt
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RiskAssessment {
    pub score: u8,
    pub level: RiskLevel,
    pub factors: Vec<RiskFactor>,
    pub recommended_action: RiskAction,
    pub assessed_at: DateTime<Utc>,
}

/// Input data for risk assessment (gathered from the login event context)
pub struct RiskInput<'a> {
    pub ip_address: Option<&'a str>,
    pub user_agent: Option<&'a str>,
    pub country_code: Option<&'a str>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub login_hour: u32,
    pub is_blacklisted: bool,
    pub recent_failure_count: i64,
    pub has_recent_password_reset: bool,
    pub has_recent_mfa_change: bool,
    pub profile: Option<&'a UserLoginProfile>,
    /// Previous login coordinates for impossible-travel check
    pub prev_latitude: Option<f64>,
    pub prev_longitude: Option<f64>,
    pub prev_login_time: Option<DateTime<Utc>>,
    pub current_time: DateTime<Utc>,
}

/// Stateless risk scoring engine.
/// All computation is in-memory — no I/O in the scoring path.
pub struct RiskEngine;

impl RiskEngine {
    /// Assess the risk of a login attempt.
    /// Returns a `RiskAssessment` with score, level, factors, and recommended action.
    pub fn assess(input: &RiskInput<'_>) -> RiskAssessment {
        let mut factors = Vec::with_capacity(6);

        let is_learning = input
            .profile
            .map(|p| p.total_logins < 10)
            .unwrap_or(true);

        // Factor 1: IP reputation (weight 0.20)
        let ip_score = Self::score_ip_reputation(input, is_learning);
        factors.push(ip_score);

        // Factor 2: Geo anomaly (weight 0.20)
        let geo_score = Self::score_geo_anomaly(input, is_learning);
        factors.push(geo_score);

        // Factor 3: Device anomaly (weight 0.15)
        let device_score = Self::score_device_anomaly(input, is_learning);
        factors.push(device_score);

        // Factor 4: Time anomaly (weight 0.10)
        let time_score = Self::score_time_anomaly(input, is_learning);
        factors.push(time_score);

        // Factor 5: Failure history (weight 0.20)
        let failure_score = Self::score_failure_history(input);
        factors.push(failure_score);

        // Factor 6: Account status (weight 0.15)
        let account_score = Self::score_account_status(input);
        factors.push(account_score);

        // Final score = sum(factor_score * weight), clamped 0-100
        let raw_score: f64 = factors
            .iter()
            .map(|f| f.score as f64 * f.weight)
            .sum();
        let score = (raw_score.round() as u8).min(100);

        let level = RiskLevel::from_score(score);
        let recommended_action = match level {
            RiskLevel::Low => RiskAction::Allow,
            RiskLevel::Medium => RiskAction::Alert,
            RiskLevel::High => RiskAction::StepUpMfa,
            RiskLevel::Critical => RiskAction::Block,
        };

        RiskAssessment {
            score,
            level,
            factors,
            recommended_action,
            assessed_at: Utc::now(),
        }
    }

    fn score_ip_reputation(input: &RiskInput<'_>, is_learning: bool) -> RiskFactor {
        let (score, detail) = if input.is_blacklisted {
            (100u8, "IP is on blacklist".to_string())
        } else if is_learning || input.profile.is_none() {
            (0, "Learning period — IP reputation skipped".to_string())
        } else if let (Some(ip), Some(profile)) = (input.ip_address, input.profile) {
            if profile.known_ips.contains(&ip.to_string()) {
                (0, "Known IP address".to_string())
            } else {
                (40, format!("New IP address: {}", ip))
            }
        } else {
            (0, "No IP data".to_string())
        };

        RiskFactor {
            name: "ip_reputation".to_string(),
            score,
            weight: 0.20,
            detail,
        }
    }

    fn score_geo_anomaly(input: &RiskInput<'_>, is_learning: bool) -> RiskFactor {
        if is_learning {
            return RiskFactor {
                name: "geo_anomaly".to_string(),
                score: 0,
                weight: 0.20,
                detail: "Learning period — geo anomaly skipped".to_string(),
            };
        }

        // Check impossible travel first
        if let (Some(lat), Some(lon), Some(prev_lat), Some(prev_lon), Some(prev_time)) = (
            input.latitude,
            input.longitude,
            input.prev_latitude,
            input.prev_longitude,
            input.prev_login_time,
        ) {
            let distance = super::geo::haversine_distance_km(prev_lat, prev_lon, lat, lon);
            let hours_elapsed = (input.current_time - prev_time).num_minutes() as f64 / 60.0;
            // Impossible travel: > 500km in < 1 hour
            if hours_elapsed < 1.0 && distance > 500.0 {
                return RiskFactor {
                    name: "geo_anomaly".to_string(),
                    score: 100,
                    weight: 0.20,
                    detail: format!(
                        "Impossible travel: {:.0} km in {:.0} min",
                        distance,
                        hours_elapsed * 60.0
                    ),
                };
            }
        }

        // Check new country
        if let (Some(cc), Some(profile)) = (input.country_code, input.profile) {
            if !profile.known_countries.contains(&cc.to_string()) {
                return RiskFactor {
                    name: "geo_anomaly".to_string(),
                    score: 60,
                    weight: 0.20,
                    detail: format!("New country: {}", cc),
                };
            }
        }

        RiskFactor {
            name: "geo_anomaly".to_string(),
            score: 0,
            weight: 0.20,
            detail: "Known location".to_string(),
        }
    }

    fn score_device_anomaly(input: &RiskInput<'_>, is_learning: bool) -> RiskFactor {
        if is_learning {
            return RiskFactor {
                name: "device_anomaly".to_string(),
                score: 0,
                weight: 0.15,
                detail: "Learning period — device anomaly skipped".to_string(),
            };
        }

        if let (Some(ua), Some(profile)) = (input.user_agent, input.profile) {
            if !profile.known_devices.contains(&ua.to_string()) {
                return RiskFactor {
                    name: "device_anomaly".to_string(),
                    score: 50,
                    weight: 0.15,
                    detail: "New device detected".to_string(),
                };
            }
        }

        RiskFactor {
            name: "device_anomaly".to_string(),
            score: 0,
            weight: 0.15,
            detail: "Known device".to_string(),
        }
    }

    fn score_time_anomaly(input: &RiskInput<'_>, is_learning: bool) -> RiskFactor {
        if is_learning {
            return RiskFactor {
                name: "time_anomaly".to_string(),
                score: 0,
                weight: 0.10,
                detail: "Learning period — time anomaly skipped".to_string(),
            };
        }

        if let Some(profile) = input.profile {
            let hour = input.login_hour as u8;
            if !profile.typical_login_hours.contains(&hour) && !profile.typical_login_hours.is_empty()
            {
                return RiskFactor {
                    name: "time_anomaly".to_string(),
                    score: 40,
                    weight: 0.10,
                    detail: format!("Unusual login hour: {}", hour),
                };
            }
        }

        RiskFactor {
            name: "time_anomaly".to_string(),
            score: 0,
            weight: 0.10,
            detail: "Typical login time".to_string(),
        }
    }

    fn score_failure_history(input: &RiskInput<'_>) -> RiskFactor {
        // Linear mapping: 0 failures = 0, 10+ failures = 100
        let score = ((input.recent_failure_count as f64 / 10.0) * 100.0)
            .round()
            .min(100.0) as u8;

        RiskFactor {
            name: "failure_history".to_string(),
            score,
            weight: 0.20,
            detail: format!("{} recent failed attempts", input.recent_failure_count),
        }
    }

    fn score_account_status(input: &RiskInput<'_>) -> RiskFactor {
        let mut score = 0u8;
        let mut details = Vec::new();

        if input.has_recent_password_reset {
            score += 30;
            details.push("recent password reset");
        }
        if input.has_recent_mfa_change {
            score += 20;
            details.push("recent MFA change");
        }

        let detail = if details.is_empty() {
            "No recent account changes".to_string()
        } else {
            details.join(", ")
        };

        RiskFactor {
            name: "account_status".to_string(),
            score: score.min(100),
            weight: 0.15,
            detail,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input<'a>() -> RiskInput<'a> {
        RiskInput {
            ip_address: Some("8.8.8.8"),
            user_agent: Some("Mozilla/5.0"),
            country_code: Some("US"),
            latitude: Some(40.7128),
            longitude: Some(-74.006),
            login_hour: 14,
            is_blacklisted: false,
            recent_failure_count: 0,
            has_recent_password_reset: false,
            has_recent_mfa_change: false,
            profile: None,
            prev_latitude: None,
            prev_longitude: None,
            prev_login_time: None,
            current_time: Utc::now(),
        }
    }

    fn make_profile() -> UserLoginProfile {
        use crate::models::common::StringUuid;
        UserLoginProfile {
            id: StringUuid::new_v4(),
            user_id: StringUuid::new_v4(),
            known_ips: vec!["8.8.8.8".to_string()],
            known_devices: vec!["Mozilla/5.0".to_string()],
            known_countries: vec!["US".to_string()],
            typical_login_hours: vec![9, 10, 11, 12, 13, 14, 15, 16, 17],
            avg_login_frequency: 2.0,
            total_logins: 50,
            last_updated: Utc::now(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_low_risk_known_user() {
        let profile = make_profile();
        let mut input = base_input();
        input.profile = Some(&profile);
        let assessment = RiskEngine::assess(&input);
        assert_eq!(assessment.level, RiskLevel::Low);
        assert!(assessment.score <= 25, "score: {}", assessment.score);
        assert_eq!(assessment.recommended_action, RiskAction::Allow);
    }

    #[test]
    fn test_critical_risk_blacklisted_ip() {
        let profile = make_profile();
        let mut input = base_input();
        input.profile = Some(&profile);
        input.is_blacklisted = true;
        input.recent_failure_count = 10;
        let assessment = RiskEngine::assess(&input);
        assert!(assessment.score >= 40, "score: {}", assessment.score);
    }

    #[test]
    fn test_learning_period_skips_profile_factors() {
        let mut profile = make_profile();
        profile.total_logins = 5; // learning period
        let mut input = base_input();
        input.profile = Some(&profile);
        input.ip_address = Some("1.2.3.4"); // unknown IP
        input.user_agent = Some("EvilBot"); // unknown device
        input.country_code = Some("RU"); // unknown country
        input.login_hour = 3; // unusual hour

        let assessment = RiskEngine::assess(&input);
        // During learning period, profile-dependent factors should be 0
        // Only failure_history and account_status are active
        assert_eq!(assessment.level, RiskLevel::Low);
    }

    #[test]
    fn test_impossible_travel_score() {
        let profile = make_profile();
        let mut input = base_input();
        input.profile = Some(&profile);
        // New York coords
        input.latitude = Some(40.7128);
        input.longitude = Some(-74.006);
        // Previous login from Tokyo, 30 min ago
        input.prev_latitude = Some(35.6762);
        input.prev_longitude = Some(139.6503);
        input.prev_login_time = Some(Utc::now() - chrono::Duration::minutes(30));

        let assessment = RiskEngine::assess(&input);
        // Geo anomaly factor should be 100 (impossible travel)
        let geo_factor = assessment
            .factors
            .iter()
            .find(|f| f.name == "geo_anomaly")
            .unwrap();
        assert_eq!(geo_factor.score, 100);
    }

    #[test]
    fn test_new_device_new_country() {
        let profile = make_profile();
        let mut input = base_input();
        input.profile = Some(&profile);
        input.user_agent = Some("UnknownBrowser/1.0");
        input.country_code = Some("CN");

        let assessment = RiskEngine::assess(&input);
        assert!(assessment.score > 0);
        let device_factor = assessment
            .factors
            .iter()
            .find(|f| f.name == "device_anomaly")
            .unwrap();
        assert_eq!(device_factor.score, 50);
        let geo_factor = assessment
            .factors
            .iter()
            .find(|f| f.name == "geo_anomaly")
            .unwrap();
        assert_eq!(geo_factor.score, 60);
    }

    #[test]
    fn test_failure_history_linear_scaling() {
        let mut input = base_input();
        input.recent_failure_count = 5;
        let assessment = RiskEngine::assess(&input);
        let factor = assessment
            .factors
            .iter()
            .find(|f| f.name == "failure_history")
            .unwrap();
        assert_eq!(factor.score, 50);

        input.recent_failure_count = 10;
        let assessment = RiskEngine::assess(&input);
        let factor = assessment
            .factors
            .iter()
            .find(|f| f.name == "failure_history")
            .unwrap();
        assert_eq!(factor.score, 100);

        input.recent_failure_count = 15;
        let assessment = RiskEngine::assess(&input);
        let factor = assessment
            .factors
            .iter()
            .find(|f| f.name == "failure_history")
            .unwrap();
        assert_eq!(factor.score, 100); // capped
    }

    #[test]
    fn test_account_status_factors() {
        let mut input = base_input();
        input.has_recent_password_reset = true;
        input.has_recent_mfa_change = true;
        let assessment = RiskEngine::assess(&input);
        let factor = assessment
            .factors
            .iter()
            .find(|f| f.name == "account_status")
            .unwrap();
        assert_eq!(factor.score, 50);
    }

    #[test]
    fn test_risk_level_boundaries() {
        assert_eq!(RiskLevel::from_score(0), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(25), RiskLevel::Low);
        assert_eq!(RiskLevel::from_score(26), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(50), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_score(51), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(75), RiskLevel::High);
        assert_eq!(RiskLevel::from_score(76), RiskLevel::Critical);
        assert_eq!(RiskLevel::from_score(100), RiskLevel::Critical);
    }

    #[test]
    fn test_unusual_login_hour() {
        let profile = make_profile();
        let mut input = base_input();
        input.profile = Some(&profile);
        input.login_hour = 3; // 3 AM, not in typical hours

        let assessment = RiskEngine::assess(&input);
        let factor = assessment
            .factors
            .iter()
            .find(|f| f.name == "time_anomaly")
            .unwrap();
        assert_eq!(factor.score, 40);
    }
}
