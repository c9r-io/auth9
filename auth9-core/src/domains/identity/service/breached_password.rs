//! Breached password detection via HIBP k-Anonymity API
//!
//! Uses the Have I Been Pwned Passwords API to check if a password has appeared
//! in known data breaches. Only a 5-character SHA-1 prefix is sent to the API,
//! so the full password hash never leaves the server.

use crate::config::HibpConfig;
use sha1::{Digest, Sha1};
use std::time::Duration;

/// Result of a breached password check.
pub struct BreachCheckResult {
    pub is_breached: bool,
    pub breach_count: u64,
}

/// Service for checking passwords against the HIBP Pwned Passwords database.
#[derive(Clone)]
pub struct BreachedPasswordService {
    http_client: reqwest::Client,
    enabled: bool,
    api_base_url: String,
}

impl BreachedPasswordService {
    /// Create a new service from configuration.
    pub fn new(config: &HibpConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .user_agent("Auth9-Core")
            .build()
            .unwrap_or_default();

        Self {
            http_client,
            enabled: config.enabled,
            api_base_url: config.api_base_url.clone(),
        }
    }

    /// Check if a password has appeared in known data breaches.
    ///
    /// Uses the k-Anonymity model: only a 5-character SHA-1 prefix is sent to HIBP.
    /// On any error (timeout, network, parse), returns not-breached (fail-open).
    pub async fn check_password(&self, password: &str) -> BreachCheckResult {
        if !self.enabled {
            return BreachCheckResult {
                is_breached: false,
                breach_count: 0,
            };
        }

        let mut hasher = Sha1::new();
        hasher.update(password.as_bytes());
        let hash = format!("{:X}", hasher.finalize());

        let prefix = &hash[..5];
        let suffix = &hash[5..];

        let url = format!("{}/range/{}", self.api_base_url, prefix);

        let response = match self.http_client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::warn!("HIBP API request failed (fail-open): {}", e);
                return BreachCheckResult {
                    is_breached: false,
                    breach_count: 0,
                };
            }
        };

        let body = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                tracing::warn!("HIBP API response read failed (fail-open): {}", e);
                return BreachCheckResult {
                    is_breached: false,
                    breach_count: 0,
                };
            }
        };

        for line in body.lines() {
            let parts: Vec<&str> = line.trim().splitn(2, ':').collect();
            if parts.len() == 2 && parts[0].eq_ignore_ascii_case(suffix) {
                let count = parts[1].trim().parse::<u64>().unwrap_or(1);
                return BreachCheckResult {
                    is_breached: true,
                    breach_count: count,
                };
            }
        }

        BreachCheckResult {
            is_breached: false,
            breach_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn sha1_hex(input: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(input.as_bytes());
        format!("{:X}", hasher.finalize())
    }

    fn service_with_url(url: &str) -> BreachedPasswordService {
        BreachedPasswordService {
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_millis(2000))
                .build()
                .unwrap(),
            enabled: true,
            api_base_url: url.to_string(),
        }
    }

    #[test]
    fn test_sha1_hash_correctness() {
        // Known test vector: SHA-1("password") = 5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8
        let hash = sha1_hex("password"); // pragma: allowlist secret
        assert_eq!(hash, "5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8"); // pragma: allowlist secret

        // Verify prefix/suffix split
        assert_eq!(&hash[..5], "5BAA6");
        assert_eq!(&hash[5..], "1E4C9B93F3F0682250B6CF8331B7EE68FD8"); // pragma: allowlist secret
    }

    #[tokio::test]
    async fn test_breached_password_found() {
        let mock_server = MockServer::start().await;

        // SHA-1("password") prefix = "5BAA6", suffix below // pragma: allowlist secret
        let response_body =
            "0018A45C4D1DEF81644B54AB7F969B88D65:1\r\n\
             1E4C9B93F3F0682250B6CF8331B7EE68FD8:3861493\r\n\
             1E4E7FCAA4D6F8D5209B3A8C5D0A3B12345:42\r\n";

        Mock::given(method("GET"))
            .and(path_regex("/range/5BAA6"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let service = service_with_url(&mock_server.uri());
        let result = service.check_password("password").await; // pragma: allowlist secret
        assert!(result.is_breached);
        assert_eq!(result.breach_count, 3861493);
    }

    #[tokio::test]
    async fn test_password_not_breached() {
        let mock_server = MockServer::start().await;

        let response_body =
            "0018A45C4D1DEF81644B54AB7F969B88D65:1\r\n\
             AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA1:10\r\n";

        Mock::given(method("GET"))
            .and(path_regex("/range/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let service = service_with_url(&mock_server.uri());
        let result = service
            .check_password("my-super-unique-password-xyzzy-42!")
            .await;
        assert!(!result.is_breached);
        assert_eq!(result.breach_count, 0);
    }

    #[tokio::test]
    async fn test_api_timeout_fail_open() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex("/range/.*"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
            .mount(&mock_server)
            .await;

        // Service with very short timeout
        let service = BreachedPasswordService {
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_millis(100))
                .build()
                .unwrap(),
            enabled: true,
            api_base_url: mock_server.uri(),
        };

        let result = service.check_password("password").await; // pragma: allowlist secret
        assert!(!result.is_breached, "Should fail-open on timeout");
    }

    #[tokio::test]
    async fn test_api_error_fail_open() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex("/range/.*"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let service = service_with_url(&mock_server.uri());
        let result = service.check_password("password").await; // pragma: allowlist secret
        // 500 response still has a text body (empty), so it returns not-breached
        assert!(!result.is_breached, "Should fail-open on server error");
    }

    #[tokio::test]
    async fn test_disabled_skips_check() {
        let service = BreachedPasswordService {
            http_client: reqwest::Client::new(),
            enabled: false,
            api_base_url: "http://should-not-be-called".to_string(),
        };

        let result = service.check_password("password").await; // pragma: allowlist secret
        assert!(!result.is_breached);
    }

    #[tokio::test]
    async fn test_empty_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex("/range/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&mock_server)
            .await;

        let service = service_with_url(&mock_server.uri());
        let result = service.check_password("password").await; // pragma: allowlist secret
        assert!(!result.is_breached);
    }

    #[tokio::test]
    async fn test_case_insensitive_suffix_matching() {
        let mock_server = MockServer::start().await;

        // Return suffix in lowercase (HIBP API returns uppercase, but be defensive)
        let hash = sha1_hex("password"); // pragma: allowlist secret
        let suffix_lower = hash[5..].to_lowercase();
        let response_body = format!("{}:999\r\n", suffix_lower);

        Mock::given(method("GET"))
            .and(path_regex("/range/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let service = service_with_url(&mock_server.uri());
        let result = service.check_password("password").await; // pragma: allowlist secret
        assert!(result.is_breached);
        assert_eq!(result.breach_count, 999);
    }
}
