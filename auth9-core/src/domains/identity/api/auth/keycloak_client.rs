//! Keycloak HTTP client helpers for token exchange and userinfo.

use super::helpers::{build_callback_url, CallbackState};
use crate::error::{AppError, Result};
use crate::state::HasServices;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct KeycloakTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct KeycloakUserInfo {
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
}

pub(super) async fn exchange_code_for_tokens<S: HasServices>(
    state: &S,
    callback_state: &CallbackState,
    code: &str,
) -> Result<KeycloakTokenResponse> {
    let kc_client = state
        .keycloak_client()
        .get_client_by_client_id(&callback_state.client_id)
        .await?;
    let client_uuid = kc_client
        .id
        .ok_or_else(|| AppError::Keycloak("Client UUID missing".to_string()))?;

    let token_url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        state.config().keycloak.url,
        state.config().keycloak.realm
    );
    let callback_url = build_callback_url(
        state
            .config()
            .keycloak
            .core_public_url
            .as_deref()
            .unwrap_or(&state.config().jwt.issuer),
    );

    let mut params = vec![
        ("grant_type", "authorization_code".to_string()),
        ("client_id", callback_state.client_id.clone()),
        ("code", code.to_string()),
        ("redirect_uri", callback_url),
    ];

    // Public clients don't have a secret; only fetch and send secret for confidential clients
    if !kc_client.public_client {
        let client_secret = state
            .keycloak_client()
            .get_client_secret(&client_uuid)
            .await?;
        params.push(("client_secret", client_secret));
    }

    let response = reqwest::Client::new()
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::Keycloak(format!("Failed to exchange code: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Keycloak(format!(
            "Failed to exchange code: {} - {}",
            status, body
        )));
    }

    // Debug: log raw response for troubleshooting
    let body = response
        .text()
        .await
        .map_err(|e| AppError::Keycloak(format!("Failed to read token response: {}", e)))?;
    tracing::debug!("Token exchange response length: {} bytes", body.len());

    serde_json::from_str(&body)
        .map_err(|e| AppError::Keycloak(format!("Failed to parse token response: {}", e)))
}

pub(super) async fn exchange_refresh_token<S: HasServices>(
    state: &S,
    callback_state: &CallbackState,
    refresh_token: &str,
) -> Result<KeycloakTokenResponse> {
    let kc_client = state
        .keycloak_client()
        .get_client_by_client_id(&callback_state.client_id)
        .await?;
    let client_uuid = kc_client
        .id
        .ok_or_else(|| AppError::Keycloak("Client UUID missing".to_string()))?;

    let token_url = format!(
        "{}/realms/{}/protocol/openid-connect/token",
        state.config().keycloak.url,
        state.config().keycloak.realm
    );

    let mut params = vec![
        ("grant_type", "refresh_token".to_string()),
        ("client_id", callback_state.client_id.clone()),
        ("refresh_token", refresh_token.to_string()),
    ];

    if !kc_client.public_client {
        let client_secret = state
            .keycloak_client()
            .get_client_secret(&client_uuid)
            .await?;
        params.push(("client_secret", client_secret));
    }

    let response = reqwest::Client::new()
        .post(&token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::Keycloak(format!("Failed to refresh token: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Keycloak(format!(
            "Token refresh failed ({}). This endpoint requires a Keycloak refresh_token \
            (obtained from OIDC login), not an Auth9 gRPC refresh_token. \
            Details: {}",
            status, body
        )));
    }

    response
        .json()
        .await
        .map_err(|e| AppError::Keycloak(format!("Failed to parse token response: {}", e)))
}

pub(super) async fn fetch_userinfo<S: HasServices>(
    state: &S,
    access_token: &str,
) -> Result<KeycloakUserInfo> {
    let userinfo_url = format!(
        "{}/realms/{}/protocol/openid-connect/userinfo",
        state.config().keycloak.url,
        state.config().keycloak.realm
    );

    tracing::debug!(
        "Fetching userinfo from {} with token length {}",
        userinfo_url,
        access_token.len()
    );

    let response = reqwest::Client::new()
        .get(&userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::Keycloak(format!("Failed to fetch userinfo: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Keycloak(format!(
            "Failed to fetch userinfo: {} - {}",
            status, body
        )));
    }

    response
        .json()
        .await
        .map_err(|e| AppError::Keycloak(format!("Failed to parse userinfo: {}", e)))
}
