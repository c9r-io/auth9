//! CAPTCHA configuration public endpoint

use crate::state::HasServices;
use axum::{extract::State, Json};
use serde::Serialize;

/// Public CAPTCHA configuration (never exposes secret_key)
#[derive(Serialize)]
pub struct PublicCaptchaConfig {
    pub enabled: bool,
    pub provider: String,
    pub site_key: String,
    pub mode: String,
}

/// GET /api/v1/public/captcha-config
///
/// Returns the CAPTCHA configuration for frontend integration.
/// Only returns public fields (site_key) — never the secret_key.
pub async fn get_captcha_config<S: HasServices>(State(state): State<S>) -> Json<PublicCaptchaConfig> {
    let config = &state.config().captcha;
    Json(PublicCaptchaConfig {
        enabled: config.enabled,
        provider: config.provider.clone(),
        site_key: config.site_key.clone(),
        mode: config.mode.clone(),
    })
}
