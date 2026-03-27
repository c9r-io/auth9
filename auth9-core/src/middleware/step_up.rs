//! Step-up authentication middleware
//!
//! Protects sensitive operations by requiring recent MFA verification.
//! After MFA step-up, a Redis key is set with a 15-minute TTL.

use crate::cache::CacheOperations;
use crate::error::AppError;

/// TTL for step-up authentication tokens (15 minutes)
pub const STEP_UP_TTL_SECS: u64 = 900;

/// Redis key prefix for step-up tokens
const STEP_UP_KEY_PREFIX: &str = "auth9:step_up:";

/// Check if a user has completed step-up authentication recently.
pub async fn has_step_up<C: CacheOperations>(cache: &C, user_id: &str) -> bool {
    let key = format!("{}{}", STEP_UP_KEY_PREFIX, user_id);
    // Reuse the OTP cache primitives for generic key-value storage
    cache.get_otp(&key).await.ok().flatten().is_some()
}

/// Record a successful step-up authentication.
pub async fn record_step_up<C: CacheOperations>(cache: &C, user_id: &str) -> Result<(), AppError> {
    let key = format!("{}{}", STEP_UP_KEY_PREFIX, user_id);
    cache
        .store_otp(&key, "1", STEP_UP_TTL_SECS)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to record step-up: {}", e)))
}

/// Require step-up authentication. Returns an error if not recently verified.
pub fn require_step_up(has_step_up: bool, operation: &str) -> Result<(), AppError> {
    if has_step_up {
        Ok(())
    } else {
        Err(AppError::Forbidden(format!(
            "Step-up authentication required for operation: {}",
            operation
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_require_step_up_passes() {
        assert!(require_step_up(true, "change_password").is_ok());
    }

    #[test]
    fn test_require_step_up_fails() {
        let err = require_step_up(false, "change_password").unwrap_err();
        assert!(err.to_string().contains("Step-up authentication required"));
        assert!(err.to_string().contains("change_password"));
    }
}
