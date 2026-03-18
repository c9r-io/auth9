//! Enterprise OIDC broker handlers.
//!
//! Executes the full OAuth2 redirect→callback→token-exchange→profile-mapping flow
//! for enterprise SSO OIDC connectors (tenant-scoped).

use crate::cache::CacheOperations;
use crate::domains::identity::api::auth::helpers::{
    AuthorizationCodeData, LoginChallengeData, AUTH_CODE_TTL_SECS,
};
use crate::error::{AppError, Result};
use crate::models::linked_identity::CreateLinkedIdentityInput;
use crate::models::user::AddUserToTenantInput;
use crate::state::{HasCache, HasDbPool, HasIdentityProviders, HasServices, HasSessionManagement};
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use url::Url;

/// Enterprise SSO login state TTL (10 minutes)
const ENTERPRISE_SSO_STATE_TTL_SECS: u64 = 600;

// ── Data Structures ──

#[derive(Debug, Serialize, Deserialize)]
struct EnterpriseSsoLoginState {
    login_challenge_id: String,
    connector_alias: String,
    tenant_id: String,
}

#[derive(Debug, Clone)]
struct ConnectorRecord {
    alias: String,
    tenant_id: String,
    config: std::collections::HashMap<String, String>,
}

struct OAuthEndpoints {
    authorization_url: String,
    token_url: String,
    userinfo_url: String,
    scopes: String,
}

#[derive(Debug, Clone)]
struct EnterpriseProfile {
    external_user_id: String,
    email: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EnterpriseSsoAuthorizeQuery {
    pub login_challenge: String,
    pub login_hint: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EnterpriseSsoCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

// ── Endpoint Resolution ──

fn resolve_endpoints(
    config: &std::collections::HashMap<String, String>,
) -> Result<OAuthEndpoints> {
    let authorization_url = config
        .get("authorizationUrl")
        .ok_or_else(|| {
            AppError::BadRequest("Missing authorizationUrl in connector config".to_string())
        })?
        .clone();
    let token_url = config
        .get("tokenUrl")
        .ok_or_else(|| AppError::BadRequest("Missing tokenUrl in connector config".to_string()))?
        .clone();
    let userinfo_url = config
        .get("userInfoUrl")
        .ok_or_else(|| {
            AppError::BadRequest("Missing userInfoUrl in connector config".to_string())
        })?
        .clone();
    let scopes = config
        .get("scopes")
        .cloned()
        .unwrap_or_else(|| "openid email profile".to_string());

    Ok(OAuthEndpoints {
        authorization_url,
        token_url,
        userinfo_url,
        scopes,
    })
}

// ── Profile Mapping ──

fn map_profile(
    config: &std::collections::HashMap<String, String>,
    json: &serde_json::Value,
) -> Result<EnterpriseProfile> {
    // Parse claim mapping from config, or use defaults
    let sub_claim = config.get("claimSub").map(|s| s.as_str()).unwrap_or("sub");
    let email_claim = config
        .get("claimEmail")
        .map(|s| s.as_str())
        .unwrap_or("email");
    let name_claim = config
        .get("claimName")
        .map(|s| s.as_str())
        .unwrap_or("name");

    let external_user_id = json[sub_claim]
        .as_str()
        .ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!(
                "Missing '{}' claim in userinfo response",
                sub_claim
            ))
        })?
        .to_string();

    Ok(EnterpriseProfile {
        external_user_id,
        email: json[email_claim].as_str().map(String::from),
        name: json[name_claim].as_str().map(String::from),
    })
}

// ── Token Exchange ──

async fn exchange_code_for_access_token(
    endpoints: &OAuthEndpoints,
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> Result<String> {
    let params = vec![
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    let response = reqwest::Client::new()
        .post(&endpoints.token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Token exchange failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Internal(anyhow::anyhow!(
            "Token exchange failed ({}): {}",
            status,
            body
        )));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse token response: {}", e)))?;

    body["access_token"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!(
                "No access_token in token response: {}",
                body
            ))
        })
}

async fn fetch_userinfo(
    endpoints: &OAuthEndpoints,
    access_token: &str,
) -> Result<serde_json::Value> {
    let response = reqwest::Client::new()
        .get(&endpoints.userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Userinfo fetch failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Internal(anyhow::anyhow!(
            "Userinfo fetch failed ({}): {}",
            status,
            body
        )));
    }

    response
        .json()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse userinfo: {}", e)))
}

// ── Build authorize URL ──

fn build_enterprise_authorize_url(
    endpoints: &OAuthEndpoints,
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    login_hint: Option<&str>,
) -> Result<String> {
    let mut url = Url::parse(&endpoints.authorization_url)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid authorization URL: {}", e)))?;

    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("client_id", client_id);
        pairs.append_pair("redirect_uri", redirect_uri);
        pairs.append_pair("response_type", "code");
        pairs.append_pair("scope", &endpoints.scopes);
        pairs.append_pair("state", state);
        if let Some(hint) = login_hint {
            pairs.append_pair("login_hint", hint);
        }
    }

    Ok(url.to_string())
}

fn enterprise_callback_url(config: &crate::config::Config) -> String {
    let base = config
        .keycloak
        .core_public_url
        .as_deref()
        .unwrap_or(&config.jwt.issuer);
    format!(
        "{}/api/v1/enterprise-sso/callback",
        base.trim_end_matches('/')
    )
}

fn portal_login_url(config: &crate::config::Config) -> String {
    let portal = config
        .keycloak
        .portal_url
        .as_deref()
        .unwrap_or(&config.jwt.issuer);
    format!("{}/login", portal.trim_end_matches('/'))
}

// ── DB Helpers ──

async fn load_oidc_connector(
    pool: &sqlx::MySqlPool,
    alias: &str,
) -> Result<ConnectorRecord> {
    let row = sqlx::query(
        r#"
        SELECT alias, tenant_id, config
        FROM enterprise_sso_connectors
        WHERE alias = ? AND provider_type = 'oidc' AND enabled = TRUE
        LIMIT 1
        "#,
    )
    .bind(alias)
    .fetch_optional(pool)
    .await?;

    let row = row.ok_or_else(|| {
        AppError::NotFound(format!(
            "No enabled OIDC enterprise SSO connector with alias '{}'",
            alias
        ))
    })?;

    let config_value: serde_json::Value = row.try_get("config")?;
    let config: std::collections::HashMap<String, String> =
        serde_json::from_value(config_value).unwrap_or_default();

    Ok(ConnectorRecord {
        alias: row.try_get("alias")?,
        tenant_id: row.try_get("tenant_id")?,
        config,
    })
}

// ══════════════════════════════════════════════════════════════════════
// Handlers
// ══════════════════════════════════════════════════════════════════════

/// Initiate enterprise SSO OIDC login: validate login_challenge, redirect to IdP.
#[utoipa::path(
    get,
    path = "/api/v1/enterprise-sso/authorize/{alias}",
    tag = "Identity",
    responses((status = 302, description = "Redirect to enterprise OIDC provider"))
)]
pub async fn authorize<S: HasServices + HasCache + HasDbPool>(
    State(state): State<S>,
    Path(alias): Path<String>,
    Query(params): Query<EnterpriseSsoAuthorizeQuery>,
) -> Result<Response> {
    // 1. Verify login_challenge exists (peek, do NOT consume)
    let challenge_json = state
        .cache()
        .consume_login_challenge(&params.login_challenge)
        .await?;
    let challenge_json = challenge_json.ok_or_else(|| {
        AppError::BadRequest("Invalid or expired login challenge".to_string())
    })?;
    // Re-store it immediately (peek pattern)
    state
        .cache()
        .store_login_challenge(
            &params.login_challenge,
            &challenge_json,
            super::auth::LOGIN_CHALLENGE_TTL_SECS,
        )
        .await?;

    // 2. Look up OIDC connector
    let connector = load_oidc_connector(state.db_pool(), &alias).await?;
    let endpoints = resolve_endpoints(&connector.config)?;

    // 3. Store enterprise SSO login state
    let sso_state_id = uuid::Uuid::new_v4().to_string();
    let sso_state = EnterpriseSsoLoginState {
        login_challenge_id: params.login_challenge,
        connector_alias: connector.alias.clone(),
        tenant_id: connector.tenant_id.clone(),
    };
    let sso_state_json =
        serde_json::to_string(&sso_state).map_err(|e| AppError::Internal(e.into()))?;
    state
        .cache()
        .store_enterprise_sso_state(&sso_state_id, &sso_state_json, ENTERPRISE_SSO_STATE_TTL_SECS)
        .await?;

    // 4. Build authorize URL
    let client_id = connector.config.get("clientId").ok_or_else(|| {
        AppError::BadRequest("Missing clientId in connector config".to_string())
    })?;
    let redirect_uri = enterprise_callback_url(state.config());

    let authorize_url = build_enterprise_authorize_url(
        &endpoints,
        client_id,
        &redirect_uri,
        &sso_state_id,
        params.login_hint.as_deref(),
    )?;

    metrics::counter!("auth9_enterprise_sso_total", "action" => "authorize", "connector" => connector.alias.clone())
        .increment(1);

    Ok(Redirect::temporary(&authorize_url).into_response())
}

/// Enterprise OIDC callback: exchange code, find/create user, complete login challenge.
#[utoipa::path(
    get,
    path = "/api/v1/enterprise-sso/callback",
    tag = "Identity",
    responses((status = 302, description = "Redirect with authorization code"))
)]
pub async fn callback<S: HasServices + HasIdentityProviders + HasCache + HasSessionManagement + HasDbPool>(
    State(state): State<S>,
    Query(params): Query<EnterpriseSsoCallbackQuery>,
) -> Result<Response> {
    // 1. Check for error from provider
    if params.error.is_some() {
        let login_url = portal_login_url(state.config());
        return Ok(
            Redirect::temporary(&format!("{}?error=enterprise_sso_cancelled", login_url))
                .into_response(),
        );
    }

    let code = params
        .code
        .ok_or_else(|| AppError::BadRequest("Missing code parameter".to_string()))?;
    let state_id = params
        .state
        .ok_or_else(|| AppError::BadRequest("Missing state parameter".to_string()))?;

    // 2. Consume enterprise SSO state
    let sso_state_json = state
        .cache()
        .consume_enterprise_sso_state(&state_id)
        .await?
        .ok_or_else(|| {
            AppError::BadRequest("Invalid or expired enterprise SSO state".to_string())
        })?;
    let sso_state: EnterpriseSsoLoginState =
        serde_json::from_str(&sso_state_json).map_err(|e| AppError::Internal(e.into()))?;

    // 3. Look up connector
    let connector = load_oidc_connector(state.db_pool(), &sso_state.connector_alias).await?;
    let endpoints = resolve_endpoints(&connector.config)?;
    let client_id = connector.config.get("clientId").ok_or_else(|| {
        AppError::BadRequest("Missing clientId in connector config".to_string())
    })?;
    let client_secret = connector.config.get("clientSecret").ok_or_else(|| {
        AppError::BadRequest("Missing clientSecret in connector config".to_string())
    })?;

    // 4. Exchange code for access token
    let redirect_uri = enterprise_callback_url(state.config());
    let access_token = exchange_code_for_access_token(
        &endpoints,
        client_id,
        client_secret,
        &code,
        &redirect_uri,
    )
    .await?;

    // 5. Fetch userinfo
    let userinfo_json = fetch_userinfo(&endpoints, &access_token).await?;

    // 6. Map profile using connector claim mapping
    let profile = map_profile(&connector.config, &userinfo_json)?;

    // 7. Find or create user (enterprise SSO is tenant-scoped)
    let user = find_or_create_enterprise_user(
        &state,
        &connector.alias,
        &sso_state.tenant_id,
        &profile,
    )
    .await?;

    // 8. Create session
    let session = state
        .session_service()
        .create_session(user.id, None, None, None)
        .await?;

    // 9. Consume login challenge and generate authorization code
    let challenge_json = state
        .cache()
        .consume_login_challenge(&sso_state.login_challenge_id)
        .await?
        .ok_or_else(|| {
            AppError::BadRequest("Login challenge expired during enterprise SSO login".to_string())
        })?;
    let challenge: LoginChallengeData =
        serde_json::from_str(&challenge_json).map_err(|e| AppError::Internal(e.into()))?;

    let auth_code = uuid::Uuid::new_v4().to_string();
    let code_data = AuthorizationCodeData {
        user_id: user.id.to_string(),
        email: user.email.clone(),
        display_name: user.display_name.clone(),
        session_id: session.id.to_string(),
        client_id: challenge.client_id.clone(),
        redirect_uri: challenge.redirect_uri.clone(),
        scope: challenge.scope,
        nonce: challenge.nonce,
        code_challenge: challenge.code_challenge,
        code_challenge_method: challenge.code_challenge_method,
    };
    let code_json =
        serde_json::to_string(&code_data).map_err(|e| AppError::Internal(e.into()))?;
    state
        .cache()
        .store_authorization_code(&auth_code, &code_json, AUTH_CODE_TTL_SECS)
        .await?;

    // 10. Build redirect URL
    let mut redirect_url = Url::parse(&challenge.redirect_uri)
        .map_err(|e| AppError::BadRequest(format!("Invalid redirect_uri: {}", e)))?;
    {
        let mut pairs = redirect_url.query_pairs_mut();
        pairs.append_pair("code", &auth_code);
        if let Some(original_state) = challenge.original_state {
            pairs.append_pair("state", &original_state);
        }
    }

    metrics::counter!("auth9_enterprise_sso_total", "action" => "callback_success", "connector" => connector.alias.clone())
        .increment(1);

    let mut response = Redirect::temporary(redirect_url.as_str()).into_response();
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        "no-store".parse().unwrap(),
    );
    Ok(response)
}

// ── User Resolution (tenant-scoped) ──

async fn find_or_create_enterprise_user<S: HasServices + HasIdentityProviders>(
    state: &S,
    connector_alias: &str,
    tenant_id: &str,
    profile: &EnterpriseProfile,
) -> Result<crate::models::user::User> {
    let tenant_uuid = uuid::Uuid::parse_str(tenant_id)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid tenant_id in SSO state")))?;

    // Try to find existing linked identity
    let existing_link = state
        .identity_provider_service()
        .find_linked_identity(connector_alias, &profile.external_user_id)
        .await?;

    if let Some(linked) = existing_link {
        return state.user_service().get(linked.user_id).await;
    }

    // If email exists, try to find existing user by email and auto-link
    if let Some(ref email) = profile.email {
        if let Ok(existing_user) = state.user_service().get_by_email(email).await {
            // Auto-link
            let input = CreateLinkedIdentityInput {
                user_id: existing_user.id,
                provider_type: "oidc".to_string(),
                provider_alias: connector_alias.to_string(),
                external_user_id: profile.external_user_id.clone(),
                external_email: profile.email.clone(),
            };
            let _ = state
                .identity_provider_service()
                .create_linked_identity(&input)
                .await;

            // Ensure tenant membership
            ensure_tenant_membership(state, existing_user.id, tenant_uuid).await;

            return Ok(existing_user);
        }
    }

    // Create new user
    let email = profile.email.clone().ok_or_else(|| {
        AppError::BadRequest(
            "Enterprise IdP did not return an email. Cannot create account.".to_string(),
        )
    })?;

    let identity_subject = uuid::Uuid::new_v4().to_string();
    let create_input = crate::models::user::CreateUserInput {
        email: email.clone(),
        display_name: profile.name.clone(),
        avatar_url: None,
    };
    let new_user = state
        .user_service()
        .create(&identity_subject, create_input)
        .await?;

    // Create linked identity
    let input = CreateLinkedIdentityInput {
        user_id: new_user.id,
        provider_type: "oidc".to_string(),
        provider_alias: connector_alias.to_string(),
        external_user_id: profile.external_user_id.clone(),
        external_email: profile.email.clone(),
    };
    let _ = state
        .identity_provider_service()
        .create_linked_identity(&input)
        .await;

    // Add to tenant
    ensure_tenant_membership(state, new_user.id, tenant_uuid).await;

    Ok(new_user)
}

async fn ensure_tenant_membership<S: HasServices>(
    state: &S,
    user_id: crate::models::common::StringUuid,
    tenant_id: uuid::Uuid,
) {
    // Check if already a member
    if let Ok(tenants) = state.user_service().get_user_tenants(user_id).await {
        if tenants
            .iter()
            .any(|t| *t.tenant_id == tenant_id)
        {
            return;
        }
    }

    // Add as member
    let input = AddUserToTenantInput {
        user_id: *user_id,
        tenant_id,
        role_in_tenant: "member".to_string(),
    };
    let _ = state.user_service().add_to_tenant(input).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_resolve_endpoints_success() {
        let mut config = HashMap::new();
        config.insert(
            "authorizationUrl".to_string(),
            "https://idp.example.com/auth".to_string(),
        );
        config.insert(
            "tokenUrl".to_string(),
            "https://idp.example.com/token".to_string(),
        );
        config.insert(
            "userInfoUrl".to_string(),
            "https://idp.example.com/userinfo".to_string(),
        );
        let endpoints = resolve_endpoints(&config).unwrap();
        assert_eq!(endpoints.authorization_url, "https://idp.example.com/auth");
        assert_eq!(endpoints.token_url, "https://idp.example.com/token");
        assert_eq!(endpoints.userinfo_url, "https://idp.example.com/userinfo");
        assert_eq!(endpoints.scopes, "openid email profile");
    }

    #[test]
    fn test_resolve_endpoints_custom_scopes() {
        let mut config = HashMap::new();
        config.insert(
            "authorizationUrl".to_string(),
            "https://idp.example.com/auth".to_string(),
        );
        config.insert(
            "tokenUrl".to_string(),
            "https://idp.example.com/token".to_string(),
        );
        config.insert(
            "userInfoUrl".to_string(),
            "https://idp.example.com/userinfo".to_string(),
        );
        config.insert("scopes".to_string(), "openid email".to_string());
        let endpoints = resolve_endpoints(&config).unwrap();
        assert_eq!(endpoints.scopes, "openid email");
    }

    #[test]
    fn test_resolve_endpoints_missing_url() {
        let config = HashMap::new();
        let result = resolve_endpoints(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_endpoints_missing_userinfo() {
        let mut config = HashMap::new();
        config.insert(
            "authorizationUrl".to_string(),
            "https://idp.example.com/auth".to_string(),
        );
        config.insert(
            "tokenUrl".to_string(),
            "https://idp.example.com/token".to_string(),
        );
        let result = resolve_endpoints(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_profile_default_claims() {
        let config = HashMap::new();
        let json = serde_json::json!({
            "sub": "enterprise-user-123",
            "email": "user@corp.example.com",
            "name": "Enterprise User"
        });
        let profile = map_profile(&config, &json).unwrap();
        assert_eq!(profile.external_user_id, "enterprise-user-123");
        assert_eq!(profile.email, Some("user@corp.example.com".to_string()));
        assert_eq!(profile.name, Some("Enterprise User".to_string()));
    }

    #[test]
    fn test_map_profile_custom_claims() {
        let mut config = HashMap::new();
        config.insert("claimSub".to_string(), "oid".to_string());
        config.insert("claimEmail".to_string(), "upn".to_string());
        config.insert("claimName".to_string(), "display_name".to_string());
        let json = serde_json::json!({
            "oid": "azure-oid-456",
            "upn": "user@corp.example.com",
            "display_name": "Corp User"
        });
        let profile = map_profile(&config, &json).unwrap();
        assert_eq!(profile.external_user_id, "azure-oid-456");
        assert_eq!(profile.email, Some("user@corp.example.com".to_string()));
        assert_eq!(profile.name, Some("Corp User".to_string()));
    }

    #[test]
    fn test_map_profile_missing_sub_claim() {
        let config = HashMap::new();
        let json = serde_json::json!({
            "email": "user@example.com"
        });
        let result = map_profile(&config, &json);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_profile_minimal() {
        let config = HashMap::new();
        let json = serde_json::json!({
            "sub": "user-789"
        });
        let profile = map_profile(&config, &json).unwrap();
        assert_eq!(profile.external_user_id, "user-789");
        assert!(profile.email.is_none());
        assert!(profile.name.is_none());
    }

    #[test]
    fn test_build_enterprise_authorize_url() {
        let endpoints = OAuthEndpoints {
            authorization_url: "https://idp.corp.example.com/authorize".to_string(),
            token_url: String::new(),
            userinfo_url: String::new(),
            scopes: "openid email profile".to_string(),
        };
        let url = build_enterprise_authorize_url(
            &endpoints,
            "my-client-id",
            "https://auth9.example.com/api/v1/enterprise-sso/callback",
            "state-123",
            None,
        )
        .unwrap();
        assert!(url.contains("client_id=my-client-id"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("state=state-123"));
        assert!(url.contains("scope=openid"));
        assert!(!url.contains("login_hint"));
    }

    #[test]
    fn test_build_enterprise_authorize_url_with_login_hint() {
        let endpoints = OAuthEndpoints {
            authorization_url: "https://idp.corp.example.com/authorize".to_string(),
            token_url: String::new(),
            userinfo_url: String::new(),
            scopes: "openid email profile".to_string(),
        };
        let url = build_enterprise_authorize_url(
            &endpoints,
            "client-id",
            "https://auth9.example.com/api/v1/enterprise-sso/callback",
            "state-456",
            Some("user@corp.example.com"),
        )
        .unwrap();
        assert!(url.contains("login_hint=user%40corp.example.com"));
    }

    #[test]
    fn test_enterprise_sso_login_state_roundtrip() {
        let state = EnterpriseSsoLoginState {
            login_challenge_id: "challenge-123".to_string(),
            connector_alias: "okta-oidc".to_string(),
            tenant_id: "tenant-456".to_string(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let decoded: EnterpriseSsoLoginState = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.login_challenge_id, "challenge-123");
        assert_eq!(decoded.connector_alias, "okta-oidc");
        assert_eq!(decoded.tenant_id, "tenant-456");
    }
}
