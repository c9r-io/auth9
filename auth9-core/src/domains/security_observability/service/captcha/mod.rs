//! CAPTCHA provider abstraction layer for bot protection
//!
//! Supports multiple CAPTCHA providers (Cloudflare Turnstile, reCAPTCHA v3, hCaptcha)
//! with a unified trait interface.

pub mod turnstile;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Result of a CAPTCHA token verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaVerification {
    /// Whether the token was valid
    pub success: bool,
    /// Risk score (0.0–1.0), supported by some providers (Turnstile, reCAPTCHA v3)
    pub score: Option<f64>,
    /// Timestamp of the challenge
    pub challenge_ts: Option<String>,
    /// Hostname the challenge was solved on
    pub hostname: Option<String>,
    /// Error codes returned by the provider
    pub error_codes: Vec<String>,
}

/// CAPTCHA provider trait — implement for each provider
#[async_trait]
pub trait CaptchaProvider: Send + Sync {
    /// Verify a CAPTCHA response token
    async fn verify(
        &self,
        token: &str,
        remote_ip: Option<&str>,
    ) -> crate::error::Result<CaptchaVerification>;

    /// Provider name for logging and configuration
    fn provider_name(&self) -> &str;
}

/// Supported CAPTCHA provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptchaProviderType {
    Turnstile,
    RecaptchaV3,
    HCaptcha,
}

impl fmt::Display for CaptchaProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Turnstile => write!(f, "turnstile"),
            Self::RecaptchaV3 => write!(f, "recaptcha_v3"),
            Self::HCaptcha => write!(f, "hcaptcha"),
        }
    }
}

impl std::str::FromStr for CaptchaProviderType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "turnstile" => Ok(Self::Turnstile),
            "recaptcha_v3" | "recaptchav3" | "recaptcha" => Ok(Self::RecaptchaV3),
            "hcaptcha" => Ok(Self::HCaptcha),
            _ => Err(format!("Unknown CAPTCHA provider: {}", s)),
        }
    }
}

/// CAPTCHA enforcement mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptchaMode {
    /// All requests to protected endpoints require CAPTCHA
    Always,
    /// CAPTCHA required only when suspicious activity is detected
    Adaptive,
    /// CAPTCHA disabled
    Disabled,
}

impl fmt::Display for CaptchaMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Always => write!(f, "always"),
            Self::Adaptive => write!(f, "adaptive"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}

impl std::str::FromStr for CaptchaMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "always" => Ok(Self::Always),
            "adaptive" => Ok(Self::Adaptive),
            "disabled" => Ok(Self::Disabled),
            _ => Err(format!("Unknown CAPTCHA mode: {}", s)),
        }
    }
}

/// A no-op CAPTCHA provider that always succeeds (used when CAPTCHA is disabled)
pub struct NoOpCaptchaProvider;

#[async_trait]
impl CaptchaProvider for NoOpCaptchaProvider {
    async fn verify(
        &self,
        _token: &str,
        _remote_ip: Option<&str>,
    ) -> crate::error::Result<CaptchaVerification> {
        Ok(CaptchaVerification {
            success: true,
            score: None,
            challenge_ts: None,
            hostname: None,
            error_codes: vec![],
        })
    }

    fn provider_name(&self) -> &str {
        "noop"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_from_str() {
        assert_eq!(
            "turnstile".parse::<CaptchaProviderType>().unwrap(),
            CaptchaProviderType::Turnstile
        );
        assert_eq!(
            "recaptcha_v3".parse::<CaptchaProviderType>().unwrap(),
            CaptchaProviderType::RecaptchaV3
        );
        assert_eq!(
            "hcaptcha".parse::<CaptchaProviderType>().unwrap(),
            CaptchaProviderType::HCaptcha
        );
        assert!("invalid".parse::<CaptchaProviderType>().is_err());
    }

    #[test]
    fn test_provider_type_display() {
        assert_eq!(CaptchaProviderType::Turnstile.to_string(), "turnstile");
        assert_eq!(CaptchaProviderType::RecaptchaV3.to_string(), "recaptcha_v3");
        assert_eq!(CaptchaProviderType::HCaptcha.to_string(), "hcaptcha");
    }

    #[test]
    fn test_captcha_mode_from_str() {
        assert_eq!("always".parse::<CaptchaMode>().unwrap(), CaptchaMode::Always);
        assert_eq!(
            "adaptive".parse::<CaptchaMode>().unwrap(),
            CaptchaMode::Adaptive
        );
        assert_eq!(
            "disabled".parse::<CaptchaMode>().unwrap(),
            CaptchaMode::Disabled
        );
        assert!("invalid".parse::<CaptchaMode>().is_err());
    }

    #[test]
    fn test_captcha_mode_display() {
        assert_eq!(CaptchaMode::Always.to_string(), "always");
        assert_eq!(CaptchaMode::Adaptive.to_string(), "adaptive");
        assert_eq!(CaptchaMode::Disabled.to_string(), "disabled");
    }

    #[tokio::test]
    async fn test_noop_provider_always_succeeds() {
        let provider = NoOpCaptchaProvider;
        let result = provider.verify("any-token", None).await.unwrap();
        assert!(result.success);
        assert_eq!(provider.provider_name(), "noop");
    }
}
