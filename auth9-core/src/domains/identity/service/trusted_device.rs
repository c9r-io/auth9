//! Trusted device service — allows users to skip MFA on known devices

use crate::error::Result;
use crate::models::common::StringUuid;
use crate::repository::TrustedDeviceRepository;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use std::sync::Arc;
use utoipa::ToSchema;

/// Trusted device entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct TrustedDevice {
    pub id: StringUuid,
    pub user_id: StringUuid,
    pub tenant_id: Option<StringUuid>,
    pub device_fingerprint: String,
    pub device_name: Option<String>,
    pub trusted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
    pub revoked: bool,
}

/// Compute a device fingerprint from user-agent and IP address.
/// Uses SHA-256(user_agent + IP /24 subnet) to allow minor IP changes.
pub fn compute_device_fingerprint(user_agent: &str, ip: &str) -> String {
    let subnet = extract_ip_subnet(ip);
    let mut hasher = Sha256::new();
    hasher.update(user_agent.as_bytes());
    hasher.update(b"|");
    hasher.update(subnet.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Extract /24 subnet from an IPv4 address (e.g., "1.2.3.4" → "1.2.3")
/// For IPv6 or unparseable addresses, return the full address.
fn extract_ip_subnet(ip: &str) -> String {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() == 4 {
        format!("{}.{}.{}", parts[0], parts[1], parts[2])
    } else {
        // IPv6 or unknown — use /48 prefix or full address
        ip.to_string()
    }
}

/// Trusted device management service
pub struct TrustedDeviceService<R: TrustedDeviceRepository> {
    repo: Arc<R>,
}

impl<R: TrustedDeviceRepository> TrustedDeviceService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    /// Check if a device is currently trusted for a user
    pub async fn is_trusted(&self, user_id: StringUuid, fingerprint: &str) -> Result<bool> {
        match self
            .repo
            .find_by_user_and_fingerprint(user_id, fingerprint)
            .await?
        {
            Some(device) => {
                if device.revoked || device.expires_at < Utc::now() {
                    Ok(false)
                } else {
                    // Update last_used_at
                    let _ = self.repo.update_last_used(device.id).await;
                    Ok(true)
                }
            }
            None => Ok(false),
        }
    }

    /// Trust a device for the specified number of days
    pub async fn trust_device(
        &self,
        user_id: StringUuid,
        tenant_id: Option<StringUuid>,
        fingerprint: &str,
        device_name: Option<&str>,
        trust_days: u16,
    ) -> Result<TrustedDevice> {
        let now = Utc::now();
        let device = TrustedDevice {
            id: StringUuid::new_v4(),
            user_id,
            tenant_id,
            device_fingerprint: fingerprint.to_string(),
            device_name: device_name.map(String::from),
            trusted_at: now,
            expires_at: now + Duration::days(trust_days as i64),
            last_used_at: now,
            revoked: false,
        };
        self.repo.create(&device).await?;
        Ok(device)
    }

    /// List all trusted devices for a user (including expired/revoked for display)
    pub async fn list_devices(&self, user_id: StringUuid) -> Result<Vec<TrustedDevice>> {
        self.repo.list_by_user(user_id).await
    }

    /// Revoke a specific trusted device
    pub async fn revoke_device(&self, device_id: StringUuid) -> Result<()> {
        self.repo.revoke(device_id).await
    }

    /// Revoke all trusted devices for a user
    pub async fn revoke_all(&self, user_id: StringUuid) -> Result<u64> {
        self.repo.revoke_all_by_user(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::trusted_device::MockTrustedDeviceRepository;

    #[test]
    fn test_compute_fingerprint_deterministic() {
        let fp1 = compute_device_fingerprint("Mozilla/5.0", "192.168.1.100");
        let fp2 = compute_device_fingerprint("Mozilla/5.0", "192.168.1.200");
        // Same /24 subnet → same fingerprint
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_compute_fingerprint_different_subnets() {
        let fp1 = compute_device_fingerprint("Mozilla/5.0", "192.168.1.100");
        let fp2 = compute_device_fingerprint("Mozilla/5.0", "10.0.0.100");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_compute_fingerprint_different_user_agents() {
        let fp1 = compute_device_fingerprint("Mozilla/5.0", "8.8.8.8");
        let fp2 = compute_device_fingerprint("Chrome/120", "8.8.8.8");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_is_64_hex_chars() {
        let fp = compute_device_fingerprint("Test", "1.2.3.4");
        assert_eq!(fp.len(), 64);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_extract_ip_subnet_ipv4() {
        assert_eq!(extract_ip_subnet("192.168.1.100"), "192.168.1");
        assert_eq!(extract_ip_subnet("10.0.0.1"), "10.0.0");
    }

    #[test]
    fn test_extract_ip_subnet_ipv6() {
        let ipv6 = "2001:db8::1";
        assert_eq!(extract_ip_subnet(ipv6), ipv6);
    }

    #[tokio::test]
    async fn test_is_trusted_valid_device() {
        let user_id = StringUuid::new_v4();
        let mut mock = MockTrustedDeviceRepository::new();

        let device = TrustedDevice {
            id: StringUuid::new_v4(),
            user_id,
            tenant_id: None,
            device_fingerprint: "abc123".to_string(),
            device_name: None,
            trusted_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(30),
            last_used_at: Utc::now(),
            revoked: false,
        };
        let device_clone = device.clone();

        mock.expect_find_by_user_and_fingerprint()
            .returning(move |_, _| Ok(Some(device_clone.clone())));
        mock.expect_update_last_used().returning(|_| Ok(()));

        let service = TrustedDeviceService::new(Arc::new(mock));
        assert!(service.is_trusted(user_id, "abc123").await.unwrap());
    }

    #[tokio::test]
    async fn test_is_trusted_expired_device() {
        let user_id = StringUuid::new_v4();
        let mut mock = MockTrustedDeviceRepository::new();

        let device = TrustedDevice {
            id: StringUuid::new_v4(),
            user_id,
            tenant_id: None,
            device_fingerprint: "abc123".to_string(),
            device_name: None,
            trusted_at: Utc::now() - Duration::days(31),
            expires_at: Utc::now() - Duration::days(1), // expired
            last_used_at: Utc::now() - Duration::days(5),
            revoked: false,
        };

        mock.expect_find_by_user_and_fingerprint()
            .returning(move |_, _| Ok(Some(device.clone())));

        let service = TrustedDeviceService::new(Arc::new(mock));
        assert!(!service.is_trusted(user_id, "abc123").await.unwrap());
    }

    #[tokio::test]
    async fn test_is_trusted_revoked_device() {
        let user_id = StringUuid::new_v4();
        let mut mock = MockTrustedDeviceRepository::new();

        let device = TrustedDevice {
            id: StringUuid::new_v4(),
            user_id,
            tenant_id: None,
            device_fingerprint: "abc123".to_string(),
            device_name: None,
            trusted_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(30),
            last_used_at: Utc::now(),
            revoked: true, // revoked
        };

        mock.expect_find_by_user_and_fingerprint()
            .returning(move |_, _| Ok(Some(device.clone())));

        let service = TrustedDeviceService::new(Arc::new(mock));
        assert!(!service.is_trusted(user_id, "abc123").await.unwrap());
    }

    #[tokio::test]
    async fn test_is_trusted_no_device() {
        let user_id = StringUuid::new_v4();
        let mut mock = MockTrustedDeviceRepository::new();

        mock.expect_find_by_user_and_fingerprint()
            .returning(|_, _| Ok(None));

        let service = TrustedDeviceService::new(Arc::new(mock));
        assert!(!service.is_trusted(user_id, "abc123").await.unwrap());
    }

    #[tokio::test]
    async fn test_trust_device_creates_entry() {
        let user_id = StringUuid::new_v4();
        let mut mock = MockTrustedDeviceRepository::new();

        mock.expect_create()
            .withf(|d| d.device_fingerprint == "fp123" && !d.revoked)
            .returning(|_| Ok(()));

        let service = TrustedDeviceService::new(Arc::new(mock));
        let result = service
            .trust_device(user_id, None, "fp123", Some("My Laptop"), 30)
            .await;
        assert!(result.is_ok());
        let device = result.unwrap();
        assert_eq!(device.device_fingerprint, "fp123");
        assert!(!device.revoked);
    }
}
