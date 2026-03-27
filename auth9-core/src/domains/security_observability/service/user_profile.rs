//! User login profile service — maintains behavioral baselines for anomaly detection

use crate::error::Result;
use crate::models::analytics::LoginEvent;
use crate::models::common::StringUuid;
use crate::repository::UserLoginProfileRepository;
use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use utoipa::ToSchema;

/// Maximum number of IPs/devices/countries to retain in the profile
const MAX_KNOWN_IPS: usize = 100;
const MAX_KNOWN_DEVICES: usize = 50;
const MAX_KNOWN_COUNTRIES: usize = 50;

/// Sliding window in days for known IPs/devices/countries (used in future Redis cache eviction)
#[allow(dead_code)]
const SLIDING_WINDOW_DAYS: i64 = 90;

/// Number of logins before the profile exits the learning period
pub const LEARNING_PERIOD_LOGINS: i32 = 10;

/// User login behavior profile (stored in DB, cached in Redis in future)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct UserLoginProfile {
    pub id: StringUuid,
    pub user_id: StringUuid,
    /// Recent IP addresses (last 90 days, max 100)
    #[sqlx(json)]
    pub known_ips: Vec<String>,
    /// Known device fingerprints (user-agent strings)
    #[sqlx(json)]
    pub known_devices: Vec<String>,
    /// Known country codes (ISO 3166-1 alpha-2)
    #[sqlx(json)]
    pub known_countries: Vec<String>,
    /// Typical login hours (0-23)
    #[sqlx(json)]
    pub typical_login_hours: Vec<u8>,
    /// Rolling average daily login frequency
    pub avg_login_frequency: f64,
    /// Total login count (used for learning period detection)
    pub total_logins: i32,
    pub last_updated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl UserLoginProfile {
    pub fn is_learning_period(&self) -> bool {
        self.total_logins < LEARNING_PERIOD_LOGINS
    }
}

/// Service for managing user login profiles
pub struct UserLoginProfileService<R: UserLoginProfileRepository> {
    repo: Arc<R>,
}

impl<R: UserLoginProfileRepository> UserLoginProfileService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    /// Get the profile for a user, or None if no profile exists yet.
    pub async fn get_profile(&self, user_id: StringUuid) -> Result<Option<UserLoginProfile>> {
        self.repo.find_by_user_id(user_id).await
    }

    /// Update the user's login profile after a successful login event.
    /// Creates the profile if it doesn't exist. This should be called
    /// asynchronously (non-blocking) from the login flow.
    pub async fn update_after_login(&self, event: &LoginEvent) -> Result<()> {
        let user_id = match event.user_id {
            Some(uid) => uid,
            None => return Ok(()),
        };

        let mut profile = match self.repo.find_by_user_id(user_id).await? {
            Some(p) => p,
            None => UserLoginProfile {
                id: StringUuid::new_v4(),
                user_id,
                known_ips: vec![],
                known_devices: vec![],
                known_countries: vec![],
                typical_login_hours: vec![],
                avg_login_frequency: 0.0,
                total_logins: 0,
                last_updated: Utc::now(),
                created_at: Utc::now(),
            },
        };

        // Update known IPs
        if let Some(ref ip) = event.ip_address {
            if !profile.known_ips.contains(ip) {
                profile.known_ips.push(ip.clone());
                if profile.known_ips.len() > MAX_KNOWN_IPS {
                    profile.known_ips.remove(0);
                }
            }
        }

        // Update known devices (user_agent as fingerprint)
        if let Some(ref ua) = event.user_agent {
            if !profile.known_devices.contains(ua) {
                profile.known_devices.push(ua.clone());
                if profile.known_devices.len() > MAX_KNOWN_DEVICES {
                    profile.known_devices.remove(0);
                }
            }
        }

        // Update known countries
        if let Some(ref cc) = event.country_code {
            if !profile.known_countries.contains(cc) {
                profile.known_countries.push(cc.clone());
                if profile.known_countries.len() > MAX_KNOWN_COUNTRIES {
                    profile.known_countries.remove(0);
                }
            }
        }

        // Update typical login hours
        let hour = event.created_at.hour() as u8;
        if !profile.typical_login_hours.contains(&hour) {
            profile.typical_login_hours.push(hour);
            profile.typical_login_hours.sort();
        }

        // Update login frequency (simple rolling average)
        profile.total_logins += 1;
        if profile.total_logins > 1 {
            let days_since_creation = (Utc::now() - profile.created_at).num_days().max(1) as f64;
            profile.avg_login_frequency = profile.total_logins as f64 / days_since_creation;
        } else {
            profile.avg_login_frequency = 1.0;
        }

        profile.last_updated = Utc::now();

        self.repo.upsert(&profile).await
    }

    /// Delete profile when user is deleted
    pub async fn delete_profile(&self, user_id: StringUuid) -> Result<u64> {
        self.repo.delete_by_user_id(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::analytics::LoginEventType;
    use crate::repository::user_login_profile::MockUserLoginProfileRepository;

    fn make_event(user_id: StringUuid) -> LoginEvent {
        LoginEvent {
            id: 1,
            user_id: Some(user_id),
            email: Some("test@auth9.example".to_string()),
            tenant_id: None,
            event_type: LoginEventType::Success,
            ip_address: Some("203.0.113.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            device_type: Some("desktop".to_string()),
            location: Some("New York, US".to_string()),
            session_id: None,
            failure_reason: None,
            provider_alias: None,
            provider_type: None,
            latitude: Some(40.7128),
            longitude: Some(-74.006),
            country_code: Some("US".to_string()),
            risk_score: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_creates_new_profile_on_first_login() {
        let user_id = StringUuid::new_v4();
        let mut mock = MockUserLoginProfileRepository::new();

        mock.expect_find_by_user_id().returning(|_| Ok(None));
        mock.expect_upsert().returning(|_| Ok(()));

        let service = UserLoginProfileService::new(Arc::new(mock));
        let event = make_event(user_id);
        let result = service.update_after_login(&event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_updates_existing_profile() {
        let user_id = StringUuid::new_v4();
        let mut mock = MockUserLoginProfileRepository::new();

        let existing = UserLoginProfile {
            id: StringUuid::new_v4(),
            user_id,
            known_ips: vec!["10.0.0.1".to_string()],
            known_devices: vec!["OldBrowser/1.0".to_string()],
            known_countries: vec!["UK".to_string()],
            typical_login_hours: vec![10],
            avg_login_frequency: 1.0,
            total_logins: 5,
            last_updated: Utc::now(),
            created_at: Utc::now() - chrono::Duration::days(5),
        };

        mock.expect_find_by_user_id()
            .returning(move |_| Ok(Some(existing.clone())));
        mock.expect_upsert()
            .withf(|profile| {
                profile.total_logins == 6
                    && profile.known_ips.contains(&"203.0.113.1".to_string())
                    && profile.known_devices.contains(&"Mozilla/5.0".to_string())
                    && profile.known_countries.contains(&"US".to_string())
            })
            .returning(|_| Ok(()));

        let service = UserLoginProfileService::new(Arc::new(mock));
        let event = make_event(user_id);
        let result = service.update_after_login(&event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_no_duplicate_ips() {
        let user_id = StringUuid::new_v4();
        let mut mock = MockUserLoginProfileRepository::new();

        let existing = UserLoginProfile {
            id: StringUuid::new_v4(),
            user_id,
            known_ips: vec!["203.0.113.1".to_string()], // same IP as event
            known_devices: vec!["Mozilla/5.0".to_string()],
            known_countries: vec!["US".to_string()],
            typical_login_hours: vec![],
            avg_login_frequency: 1.0,
            total_logins: 3,
            last_updated: Utc::now(),
            created_at: Utc::now() - chrono::Duration::days(3),
        };

        mock.expect_find_by_user_id()
            .returning(move |_| Ok(Some(existing.clone())));
        mock.expect_upsert()
            .withf(|profile| {
                // Should still have exactly 1 IP (no duplicate added)
                profile.known_ips.len() == 1
            })
            .returning(|_| Ok(()));

        let service = UserLoginProfileService::new(Arc::new(mock));
        let event = make_event(user_id);
        let result = service.update_after_login(&event).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_learning_period() {
        let profile = UserLoginProfile {
            id: StringUuid::new_v4(),
            user_id: StringUuid::new_v4(),
            known_ips: vec![],
            known_devices: vec![],
            known_countries: vec![],
            typical_login_hours: vec![],
            avg_login_frequency: 0.0,
            total_logins: 9,
            last_updated: Utc::now(),
            created_at: Utc::now(),
        };
        assert!(profile.is_learning_period());

        let profile2 = UserLoginProfile {
            total_logins: 10,
            ..profile
        };
        assert!(!profile2.is_learning_period());
    }
}
