//! Cloudflare Turnstile CAPTCHA provider implementation

use super::{CaptchaProvider, CaptchaVerification};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

/// Cloudflare Turnstile verification response
#[derive(Debug, Deserialize)]
struct TurnstileResponse {
    success: bool,
    #[serde(default)]
    score: Option<f64>,
    #[serde(default)]
    challenge_ts: Option<String>,
    #[serde(default)]
    hostname: Option<String>,
    #[serde(default, rename = "error-codes")]
    error_codes: Vec<String>,
}

/// Cloudflare Turnstile CAPTCHA provider
pub struct TurnstileProvider {
    secret_key: String,
    http_client: Client,
    verify_url: String,
}

impl TurnstileProvider {
    /// Create a new Turnstile provider
    pub fn new(secret_key: String, timeout_ms: u64) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .expect("Failed to create HTTP client for Turnstile");

        Self {
            secret_key,
            http_client,
            verify_url: "https://challenges.cloudflare.com/turnstile/v0/siteverify".to_string(),
        }
    }

    /// Create with a custom verify URL (for testing)
    #[cfg(test)]
    pub fn with_verify_url(secret_key: String, timeout_ms: u64, verify_url: String) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .expect("Failed to create HTTP client for Turnstile");

        Self {
            secret_key,
            http_client,
            verify_url,
        }
    }
}

#[async_trait]
impl CaptchaProvider for TurnstileProvider {
    async fn verify(
        &self,
        token: &str,
        remote_ip: Option<&str>,
    ) -> crate::error::Result<CaptchaVerification> {
        let mut params = vec![("secret", self.secret_key.as_str()), ("response", token)];
        if let Some(ip) = remote_ip {
            params.push(("remoteip", ip));
        }

        let response = match self
            .http_client
            .post(&self.verify_url)
            .form(&params)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                // Fail-open: if we can't reach Turnstile, allow the request through
                tracing::warn!(
                    error = %e,
                    "Turnstile API unreachable, failing open"
                );
                return Ok(CaptchaVerification {
                    success: true,
                    score: None,
                    challenge_ts: None,
                    hostname: None,
                    error_codes: vec!["provider-unreachable".to_string()],
                });
            }
        };

        let turnstile_resp: TurnstileResponse = match response.json().await {
            Ok(resp) => resp,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to parse Turnstile response, failing open"
                );
                return Ok(CaptchaVerification {
                    success: true,
                    score: None,
                    challenge_ts: None,
                    hostname: None,
                    error_codes: vec!["parse-error".to_string()],
                });
            }
        };

        Ok(CaptchaVerification {
            success: turnstile_resp.success,
            score: turnstile_resp.score,
            challenge_ts: turnstile_resp.challenge_ts,
            hostname: turnstile_resp.hostname,
            error_codes: turnstile_resp.error_codes,
        })
    }

    fn provider_name(&self) -> &str {
        "turnstile"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_provider(mock_server: &MockServer) -> TurnstileProvider {
        TurnstileProvider::with_verify_url(
            "test-secret-key".to_string(),
            5000,
            format!("{}/turnstile/v0/siteverify", mock_server.uri()),
        )
    }

    #[tokio::test]
    async fn test_verify_success() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/turnstile/v0/siteverify"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "challenge_ts": "2026-03-25T00:00:00Z",
                "hostname": "auth9.example.com",
                "error-codes": [],
                "score": 0.9
            })))
            .mount(&mock_server)
            .await;

        let provider = setup_provider(&mock_server).await;
        let result = provider
            .verify("valid-token", Some("1.2.3.4"))
            .await
            .unwrap();

        assert!(result.success);
        assert_eq!(result.score, Some(0.9));
        assert_eq!(result.hostname.as_deref(), Some("auth9.example.com"));
        assert!(result.error_codes.is_empty());
    }

    #[tokio::test]
    async fn test_verify_failure() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/turnstile/v0/siteverify"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": false,
                "error-codes": ["invalid-input-response"]
            })))
            .mount(&mock_server)
            .await;

        let provider = setup_provider(&mock_server).await;
        let result = provider.verify("invalid-token", None).await.unwrap();

        assert!(!result.success);
        assert_eq!(result.error_codes, vec!["invalid-input-response"]);
    }

    #[tokio::test]
    async fn test_verify_expired_token() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/turnstile/v0/siteverify"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": false,
                "error-codes": ["timeout-or-duplicate"]
            })))
            .mount(&mock_server)
            .await;

        let provider = setup_provider(&mock_server).await;
        let result = provider.verify("expired-token", None).await.unwrap();

        assert!(!result.success);
        assert_eq!(result.error_codes, vec!["timeout-or-duplicate"]);
    }

    #[tokio::test]
    async fn test_verify_network_error_fails_open() {
        // Point to a non-existent server
        let provider = TurnstileProvider::with_verify_url(
            "test-secret".to_string(),
            100, // very short timeout
            "http://127.0.0.1:1/siteverify".to_string(),
        );

        let result = provider.verify("some-token", None).await.unwrap();

        // Should fail open
        assert!(result.success);
        assert_eq!(result.error_codes, vec!["provider-unreachable"]);
    }

    #[tokio::test]
    async fn test_verify_malformed_response_fails_open() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/turnstile/v0/siteverify"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock_server)
            .await;

        let provider = setup_provider(&mock_server).await;
        let result = provider.verify("some-token", None).await.unwrap();

        assert!(result.success);
        assert_eq!(result.error_codes, vec!["parse-error"]);
    }

    #[test]
    fn test_provider_name() {
        let provider = TurnstileProvider::new("key".to_string(), 5000);
        assert_eq!(provider.provider_name(), "turnstile");
    }
}
