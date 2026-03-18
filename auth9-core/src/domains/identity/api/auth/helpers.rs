//! Shared helper functions for auth API handlers.

use crate::error::{AppError, Result};
use crate::jwt::IdentityClaims;
use crate::state::HasServices;
use axum::http::HeaderMap;
use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct CallbackState {
    pub redirect_uri: String,
    pub client_id: String,
    pub original_state: Option<String>,
}

/// Login challenge data stored during authorize → consumed by authorize_complete
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct LoginChallengeData {
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub original_state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

/// Authorization code data stored during authorize_complete → consumed by token endpoint
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct AuthorizationCodeData {
    pub user_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub session_id: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

/// Authorization code TTL (2 minutes, per OIDC spec recommendation)
pub(super) const AUTH_CODE_TTL_SECS: u64 = 120;

/// Login challenge TTL (10 minutes, generous for password + MFA flow)
pub(super) const LOGIN_CHALLENGE_TTL_SECS: u64 = 600;

/// Verify PKCE S256 code_verifier against stored code_challenge.
/// Returns true if BASE64URL(SHA256(code_verifier)) == code_challenge.
pub(super) fn verify_pkce_s256(code_verifier: &str, code_challenge: &str) -> bool {
    let hash = Sha256::digest(code_verifier.as_bytes());
    let computed = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);
    computed == code_challenge
}

/// Validate that a redirect URI is allowed for the service
pub fn validate_redirect_uri(allowed_uris: &[String], redirect_uri: &str) -> Result<()> {
    if allowed_uris.contains(&redirect_uri.to_string()) {
        Ok(())
    } else {
        Err(AppError::BadRequest("Invalid redirect_uri".to_string()))
    }
}

/// Build the callback URL from issuer
pub fn build_callback_url(issuer: &str) -> String {
    format!("{}/api/v1/auth/callback", issuer.trim_end_matches('/'))
}

/// Parameters for building Keycloak authorization URL
#[derive(Debug)]
pub struct KeycloakAuthUrlParams<'a> {
    pub keycloak_public_url: &'a str,
    pub realm: &'a str,
    pub response_type: &'a str,
    pub client_id: &'a str,
    pub callback_url: &'a str,
    pub scope: &'a str,
    pub encoded_state: &'a str,
    pub nonce: Option<&'a str>,
    pub connector_alias: Option<&'a str>,
    pub kc_action: Option<&'a str>,
    pub ui_locales: Option<&'a str>,
    pub code_challenge: Option<&'a str>,
    pub code_challenge_method: Option<&'a str>,
}

/// Build Keycloak authorization URL
pub fn build_keycloak_auth_url(params: &KeycloakAuthUrlParams) -> Result<String> {
    let mut auth_url = Url::parse(&format!(
        "{}/realms/{}/protocol/openid-connect/auth",
        params.keycloak_public_url, params.realm
    ))
    .map_err(|e| AppError::Internal(e.into()))?;

    {
        let mut pairs = auth_url.query_pairs_mut();
        pairs.append_pair("response_type", params.response_type);
        pairs.append_pair("client_id", params.client_id);
        pairs.append_pair("redirect_uri", params.callback_url);
        pairs.append_pair("scope", params.scope);
        pairs.append_pair("state", params.encoded_state);
        if let Some(n) = params.nonce {
            pairs.append_pair("nonce", n);
        }
        if let Some(alias) = params.connector_alias {
            pairs.append_pair("kc_idp_hint", alias);
        }
        if let Some(action) = params.kc_action {
            pairs.append_pair("kc_action", action);
        }
        if let Some(locales) = params.ui_locales {
            pairs.append_pair("ui_locales", locales);
        }
        if let Some(challenge) = params.code_challenge {
            pairs.append_pair("code_challenge", challenge);
        }
        if let Some(method) = params.code_challenge_method {
            pairs.append_pair("code_challenge_method", method);
        }
    }

    Ok(auth_url.to_string())
}

/// Build Keycloak logout URL
pub fn build_keycloak_logout_url(
    keycloak_public_url: &str,
    realm: &str,
    id_token_hint: Option<&str>,
    post_logout_redirect_uri: Option<&str>,
    state: Option<&str>,
) -> Result<String> {
    let mut logout_url = Url::parse(&format!(
        "{}/realms/{}/protocol/openid-connect/logout",
        keycloak_public_url, realm
    ))
    .map_err(|e| AppError::Internal(e.into()))?;

    {
        let mut pairs = logout_url.query_pairs_mut();
        if let Some(hint) = id_token_hint {
            pairs.append_pair("id_token_hint", hint);
        }
        if let Some(uri) = post_logout_redirect_uri {
            pairs.append_pair("post_logout_redirect_uri", uri);
        }
        if let Some(s) = state {
            pairs.append_pair("state", s);
        }
    }

    Ok(logout_url.to_string())
}

/// Extract client IP address from request headers
/// Checks X-Forwarded-For, X-Real-IP, then falls back to None
pub(super) fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Check X-Forwarded-For first (may contain multiple IPs)
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            // Take the first IP (original client)
            if let Some(ip) = xff_str.split(',').next() {
                return Some(ip.trim().to_string());
            }
        }
    }

    // Check X-Real-IP
    if let Some(xri) = headers.get("x-real-ip") {
        if let Ok(ip) = xri.to_str() {
            return Some(ip.to_string());
        }
    }

    None
}

pub(super) fn extract_identity_claims_from_headers<S: HasServices>(
    state: &S,
    headers: &HeaderMap,
) -> Result<IdentityClaims> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .ok_or_else(|| AppError::Unauthorized("Missing authorization token".to_string()))?;
    let auth_str = auth_header
        .to_str()
        .map_err(|_| AppError::Unauthorized("Invalid authorization header".to_string()))?;
    let token = auth_str.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::Unauthorized("Authorization must use Bearer scheme".to_string())
    })?;

    state
        .jwt_manager()
        .verify_identity_token(token)
        .map_err(|e| AppError::Unauthorized(format!("Invalid identity token: {}", e)))
}

// Legacy helpers kept for unit tests and backward-compatibility checks.
#[cfg(test)]
pub(super) fn encode_state(state_payload: &CallbackState) -> Result<String> {
    use base64::Engine;
    let bytes = serde_json::to_vec(state_payload).map_err(|e| AppError::Internal(e.into()))?;
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}

#[cfg(test)]
pub(super) fn decode_state(state: Option<&str>) -> Result<CallbackState> {
    use base64::Engine;
    let encoded = state.ok_or_else(|| AppError::BadRequest("Missing state".to_string()))?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|e| AppError::BadRequest(format!("Invalid state: {}", e)))?;
    serde_json::from_slice(&bytes).map_err(|e| AppError::Internal(e.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;

    #[test]
    fn test_decode_state_success() {
        let state_payload = CallbackState {
            redirect_uri: "https://example.com/callback".to_string(),
            client_id: "test-client".to_string(),
            original_state: Some("original".to_string()),
        };

        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&state_payload).unwrap());

        let result = decode_state(Some(&encoded));
        assert!(result.is_ok());

        let decoded = result.unwrap();
        assert_eq!(decoded.redirect_uri, "https://example.com/callback");
        assert_eq!(decoded.client_id, "test-client");
        assert_eq!(decoded.original_state, Some("original".to_string()));
    }

    #[test]
    fn test_decode_state_missing() {
        let result = decode_state(None);
        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    #[test]
    fn test_decode_state_invalid_base64() {
        let result = decode_state(Some("not-valid-base64!!!"));
        assert!(matches!(result, Err(AppError::BadRequest(_))));
    }

    #[test]
    fn test_decode_state_invalid_json() {
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"not valid json");

        let result = decode_state(Some(&encoded));
        assert!(matches!(result, Err(AppError::Internal(_))));
    }

    #[test]
    fn test_decode_state_without_original_state() {
        let state_payload = CallbackState {
            redirect_uri: "https://example.com/callback".to_string(),
            client_id: "test-client".to_string(),
            original_state: None,
        };

        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&state_payload).unwrap());

        let result = decode_state(Some(&encoded));
        assert!(result.is_ok());
        assert!(result.unwrap().original_state.is_none());
    }

    #[test]
    fn test_callback_state_roundtrip() {
        let original = CallbackState {
            redirect_uri: "https://example.com/cb".to_string(),
            client_id: "my-client".to_string(),
            original_state: Some("state123".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let decoded: CallbackState = serde_json::from_str(&json).unwrap();

        assert_eq!(original.redirect_uri, decoded.redirect_uri);
        assert_eq!(original.client_id, decoded.client_id);
        assert_eq!(original.original_state, decoded.original_state);
    }

    #[test]
    fn test_validate_redirect_uri_valid() {
        let allowed = vec![
            "https://app.example.com/callback".to_string(),
            "https://app.example.com/oauth".to_string(),
        ];
        let result = validate_redirect_uri(&allowed, "https://app.example.com/callback");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_redirect_uri_invalid() {
        let allowed = vec!["https://app.example.com/callback".to_string()];
        let result = validate_redirect_uri(&allowed, "https://evil.com/callback");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_redirect_uri_empty_list() {
        let allowed: Vec<String> = vec![];
        let result = validate_redirect_uri(&allowed, "https://any.com/callback");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_callback_url() {
        let url = build_callback_url("https://auth9.example.com");
        assert_eq!(url, "https://auth9.example.com/api/v1/auth/callback");
    }

    #[test]
    fn test_build_callback_url_strips_trailing_slash() {
        let url = build_callback_url("https://auth9.example.com/");
        assert_eq!(url, "https://auth9.example.com/api/v1/auth/callback");
    }

    #[test]
    fn test_encode_state_roundtrip() {
        let state = CallbackState {
            redirect_uri: "https://app.com/cb".to_string(),
            client_id: "test-client".to_string(),
            original_state: Some("user-state".to_string()),
        };

        let encoded = encode_state(&state).unwrap();
        let decoded = decode_state(Some(&encoded)).unwrap();

        assert_eq!(state.redirect_uri, decoded.redirect_uri);
        assert_eq!(state.client_id, decoded.client_id);
        assert_eq!(state.original_state, decoded.original_state);
    }

    #[test]
    fn test_build_keycloak_auth_url() {
        let url = build_keycloak_auth_url(&KeycloakAuthUrlParams {
            keycloak_public_url: "https://keycloak.example.com",
            realm: "my-realm",
            response_type: "code",
            client_id: "my-client",
            callback_url: "https://app.com/callback",
            scope: "openid profile",
            encoded_state: "encoded-state",
            nonce: None,
            connector_alias: None,
            kc_action: None,
            ui_locales: None,
            code_challenge: None,
            code_challenge_method: None,
        })
        .unwrap();

        assert!(url.contains("keycloak.example.com"));
        assert!(url.contains("my-realm"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=my-client"));
        assert!(url.contains("scope=openid"));
    }

    #[test]
    fn test_build_keycloak_auth_url_with_nonce() {
        let url = build_keycloak_auth_url(&KeycloakAuthUrlParams {
            keycloak_public_url: "https://keycloak.example.com",
            realm: "test",
            response_type: "code",
            client_id: "client",
            callback_url: "https://app.com/cb",
            scope: "openid",
            encoded_state: "state",
            nonce: Some("my-nonce"),
            connector_alias: None,
            kc_action: None,
            ui_locales: None,
            code_challenge: None,
            code_challenge_method: None,
        })
        .unwrap();

        assert!(url.contains("nonce=my-nonce"));
    }

    #[test]
    fn test_build_keycloak_auth_url_with_kc_action() {
        let url = build_keycloak_auth_url(&KeycloakAuthUrlParams {
            keycloak_public_url: "https://keycloak.example.com",
            realm: "test",
            response_type: "code",
            client_id: "client",
            callback_url: "https://app.com/cb",
            scope: "openid",
            encoded_state: "state",
            nonce: None,
            connector_alias: Some("github"),
            kc_action: Some("idp_link:github"),
            ui_locales: None,
            code_challenge: None,
            code_challenge_method: None,
        })
        .unwrap();

        assert!(url.contains("kc_idp_hint=github"));
        assert!(url.contains("kc_action=idp_link%3Agithub"));
    }

    #[test]
    fn test_build_keycloak_auth_url_with_ui_locales() {
        let url = build_keycloak_auth_url(&KeycloakAuthUrlParams {
            keycloak_public_url: "https://keycloak.example.com",
            realm: "test",
            response_type: "code",
            client_id: "client",
            callback_url: "https://app.com/cb",
            scope: "openid",
            encoded_state: "state",
            nonce: None,
            connector_alias: None,
            kc_action: None,
            ui_locales: Some("zh-CN"),
            code_challenge: None,
            code_challenge_method: None,
        })
        .unwrap();

        assert!(url.contains("ui_locales=zh-CN"));
    }

    #[test]
    fn test_build_keycloak_auth_url_without_ui_locales() {
        let url = build_keycloak_auth_url(&KeycloakAuthUrlParams {
            keycloak_public_url: "https://keycloak.example.com",
            realm: "test",
            response_type: "code",
            client_id: "client",
            callback_url: "https://app.com/cb",
            scope: "openid",
            encoded_state: "state",
            nonce: None,
            connector_alias: None,
            kc_action: None,
            ui_locales: None,
            code_challenge: None,
            code_challenge_method: None,
        })
        .unwrap();

        assert!(!url.contains("ui_locales"));
    }

    #[test]
    fn test_build_keycloak_logout_url_minimal() {
        let url =
            build_keycloak_logout_url("https://keycloak.example.com", "my-realm", None, None, None)
                .unwrap();

        assert!(url.contains("keycloak.example.com"));
        assert!(url.contains("my-realm"));
        assert!(url.contains("logout"));
        // No query params when all options are None
        assert!(!url.contains("id_token_hint"));
    }

    #[test]
    fn test_build_keycloak_logout_url_full() {
        let url = build_keycloak_logout_url(
            "https://keycloak.example.com",
            "my-realm",
            Some("token-hint"),
            Some("https://app.com/logged-out"),
            Some("logout-state"),
        )
        .unwrap();

        assert!(url.contains("id_token_hint=token-hint"));
        assert!(url.contains("post_logout_redirect_uri="));
        assert!(url.contains("state=logout-state"));
    }

    #[test]
    fn test_build_keycloak_logout_url_partial() {
        // Only id_token_hint
        let url = build_keycloak_logout_url(
            "https://keycloak.example.com",
            "test",
            Some("hint"),
            None,
            None,
        )
        .unwrap();
        assert!(url.contains("id_token_hint=hint"));
        assert!(!url.contains("post_logout_redirect_uri"));

        // Only redirect_uri
        let url = build_keycloak_logout_url(
            "https://keycloak.example.com",
            "test",
            None,
            Some("https://app.com/logout"),
            None,
        )
        .unwrap();
        assert!(!url.contains("id_token_hint"));
        assert!(url.contains("post_logout_redirect_uri="));
    }

    #[test]
    fn test_encode_state_with_empty_original_state() {
        let state = CallbackState {
            redirect_uri: "https://app.com/cb".to_string(),
            client_id: "client".to_string(),
            original_state: None,
        };

        let encoded = encode_state(&state).unwrap();
        let decoded = decode_state(Some(&encoded)).unwrap();

        assert!(decoded.original_state.is_none());
    }

    #[test]
    fn test_validate_redirect_uri_with_multiple_uris() {
        let allowed = vec![
            "https://app1.com/cb".to_string(),
            "https://app2.com/cb".to_string(),
            "https://app3.com/cb".to_string(),
        ];

        assert!(validate_redirect_uri(&allowed, "https://app1.com/cb").is_ok());
        assert!(validate_redirect_uri(&allowed, "https://app2.com/cb").is_ok());
        assert!(validate_redirect_uri(&allowed, "https://app3.com/cb").is_ok());
        assert!(validate_redirect_uri(&allowed, "https://app4.com/cb").is_err());
    }

    #[test]
    fn test_validate_redirect_uri_exact_match() {
        let allowed = vec!["https://app.com/callback".to_string()];

        // Should not match partial or similar URIs
        assert!(validate_redirect_uri(&allowed, "https://app.com/callback").is_ok());
        assert!(validate_redirect_uri(&allowed, "https://app.com/callback/").is_err());
        assert!(validate_redirect_uri(&allowed, "https://app.com/callback?foo=bar").is_err());
    }

    #[test]
    fn test_build_callback_url_with_path() {
        let url = build_callback_url("https://auth9.example.com/api");
        assert_eq!(url, "https://auth9.example.com/api/api/v1/auth/callback");
    }

    #[test]
    fn test_build_keycloak_auth_url_encodes_special_chars() {
        let url = build_keycloak_auth_url(&KeycloakAuthUrlParams {
            keycloak_public_url: "https://keycloak.example.com",
            realm: "test",
            response_type: "code",
            client_id: "my-app",
            callback_url: "https://app.com/cb?foo=bar",
            scope: "openid profile email",
            encoded_state: "state123",
            nonce: Some("nonce with spaces"),
            connector_alias: None,
            kc_action: None,
            ui_locales: None,
            code_challenge: None,
            code_challenge_method: None,
        })
        .unwrap();

        // Verify URL encoding
        assert!(
            url.contains("scope=openid+profile+email")
                || url.contains("scope=openid%20profile%20email")
        );
    }

    #[test]
    fn test_decode_state_with_special_characters() {
        let state_payload = CallbackState {
            redirect_uri: "https://example.com/callback?foo=bar&baz=qux".to_string(),
            client_id: "client-with-dashes_and_underscores".to_string(),
            original_state: Some("state with spaces and \u{00e9}mojis \u{1f389}".to_string()),
        };

        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(&state_payload).unwrap());

        let result = decode_state(Some(&encoded));
        assert!(result.is_ok());

        let decoded = result.unwrap();
        assert!(decoded.redirect_uri.contains("foo=bar"));
        assert!(decoded
            .original_state
            .as_ref()
            .unwrap()
            .contains('\u{1f389}'));
    }

    #[test]
    fn test_decode_state_with_empty_string() {
        let result = decode_state(Some(""));
        // Empty string decodes to empty bytes, which is invalid JSON
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_state_with_unicode() {
        let state = CallbackState {
            redirect_uri: "https://\u{4f8b}\u{3048}.jp/callback".to_string(),
            client_id: "\u{65e5}\u{672c}\u{8a9e}\u{30af}\u{30e9}\u{30a4}\u{30a2}\u{30f3}\u{30c8}"
                .to_string(),
            original_state: Some(
                "\u{0441}\u{043e}\u{0441}\u{0442}\u{043e}\u{044f}\u{043d}\u{0438}\u{0435}"
                    .to_string(),
            ),
        };

        let encoded = encode_state(&state).unwrap();
        let decoded = decode_state(Some(&encoded)).unwrap();

        assert_eq!(decoded.redirect_uri, state.redirect_uri);
        assert_eq!(decoded.client_id, state.client_id);
        assert_eq!(decoded.original_state, state.original_state);
    }

    #[test]
    fn test_extract_client_ip_from_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "192.168.1.100".parse().unwrap());
        assert_eq!(
            extract_client_ip(&headers),
            Some("192.168.1.100".to_string())
        );
    }

    #[test]
    fn test_extract_client_ip_from_x_forwarded_for_multiple() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "10.0.0.1, 192.168.1.1, 172.16.0.1".parse().unwrap(),
        );
        // Should take the first IP (original client)
        assert_eq!(extract_client_ip(&headers), Some("10.0.0.1".to_string()));
    }

    #[test]
    fn test_extract_client_ip_from_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "203.0.113.50".parse().unwrap());
        assert_eq!(
            extract_client_ip(&headers),
            Some("203.0.113.50".to_string())
        );
    }

    #[test]
    fn test_extract_client_ip_x_forwarded_for_takes_priority() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "10.0.0.1".parse().unwrap());
        headers.insert("x-real-ip", "203.0.113.50".parse().unwrap());
        // X-Forwarded-For takes priority over X-Real-IP
        assert_eq!(extract_client_ip(&headers), Some("10.0.0.1".to_string()));
    }

    #[test]
    fn test_extract_client_ip_no_headers() {
        let headers = HeaderMap::new();
        assert_eq!(extract_client_ip(&headers), None);
    }

    #[test]
    fn test_extract_client_ip_ipv6() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "::1".parse().unwrap());
        assert_eq!(extract_client_ip(&headers), Some("::1".to_string()));
    }

    #[test]
    fn test_build_keycloak_auth_url_with_pkce() {
        let url = build_keycloak_auth_url(&KeycloakAuthUrlParams {
            keycloak_public_url: "https://keycloak.example.com",
            realm: "test",
            response_type: "code",
            client_id: "client",
            callback_url: "https://app.com/cb",
            scope: "openid",
            encoded_state: "state",
            nonce: None,
            connector_alias: None,
            kc_action: None,
            ui_locales: None,
            code_challenge: Some("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"),
            code_challenge_method: Some("S256"),
        })
        .unwrap();

        assert!(url.contains("code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM")); // pragma: allowlist secret
        assert!(url.contains("code_challenge_method=S256"));
    }

    #[test]
    fn test_build_keycloak_auth_url_without_pkce() {
        let url = build_keycloak_auth_url(&KeycloakAuthUrlParams {
            keycloak_public_url: "https://keycloak.example.com",
            realm: "test",
            response_type: "code",
            client_id: "client",
            callback_url: "https://app.com/cb",
            scope: "openid",
            encoded_state: "state",
            nonce: None,
            connector_alias: None,
            kc_action: None,
            ui_locales: None,
            code_challenge: None,
            code_challenge_method: None,
        })
        .unwrap();

        assert!(!url.contains("code_challenge"));
        assert!(!url.contains("code_challenge_method"));
    }

    // ==================== PKCE Tests ====================

    #[test]
    fn test_verify_pkce_s256_correct_verifier() {
        // RFC 7636 Appendix B test vector
        let code_verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"; // pragma: allowlist secret
        // SHA256 of verifier, base64url-encoded
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(code_verifier.as_bytes());
        let code_challenge =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);

        assert!(verify_pkce_s256(code_verifier, &code_challenge));
    }

    #[test]
    fn test_verify_pkce_s256_wrong_verifier() {
        let code_verifier = "correct-verifier"; // pragma: allowlist secret
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(code_verifier.as_bytes());
        let code_challenge =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);

        assert!(!verify_pkce_s256("wrong-verifier", &code_challenge));
    }

    #[test]
    fn test_verify_pkce_s256_empty_verifier() {
        assert!(!verify_pkce_s256("", "some-challenge"));
    }

    // ==================== LoginChallengeData / AuthorizationCodeData Tests ====================

    #[test]
    fn test_login_challenge_data_roundtrip() {
        let data = LoginChallengeData {
            client_id: "my-app".to_string(),
            redirect_uri: "https://app.example.com/callback".to_string(),
            scope: "openid profile".to_string(),
            original_state: Some("csrf-state".to_string()),
            nonce: Some("nonce-123".to_string()),
            code_challenge: Some("challenge".to_string()),
            code_challenge_method: Some("S256".to_string()),
        };

        let json = serde_json::to_string(&data).unwrap();
        let decoded: LoginChallengeData = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.client_id, "my-app");
        assert_eq!(decoded.nonce, Some("nonce-123".to_string()));
        assert_eq!(decoded.code_challenge_method, Some("S256".to_string()));
    }

    #[test]
    fn test_login_challenge_data_without_optionals() {
        let data = LoginChallengeData {
            client_id: "app".to_string(),
            redirect_uri: "https://app.example.com/cb".to_string(),
            scope: "openid".to_string(),
            original_state: None,
            nonce: None,
            code_challenge: None,
            code_challenge_method: None,
        };

        let json = serde_json::to_string(&data).unwrap();
        let decoded: LoginChallengeData = serde_json::from_str(&json).unwrap();
        assert!(decoded.nonce.is_none());
        assert!(decoded.code_challenge.is_none());
    }

    #[test]
    fn test_authorization_code_data_roundtrip() {
        let data = AuthorizationCodeData {
            user_id: "user-123".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            session_id: "session-456".to_string(),
            client_id: "my-app".to_string(),
            redirect_uri: "https://app.example.com/callback".to_string(),
            scope: "openid profile".to_string(),
            nonce: Some("nonce-789".to_string()),
            code_challenge: Some("challenge-abc".to_string()),
            code_challenge_method: Some("S256".to_string()),
        };

        let json = serde_json::to_string(&data).unwrap();
        let decoded: AuthorizationCodeData = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.user_id, "user-123");
        assert_eq!(decoded.email, "test@example.com");
        assert_eq!(decoded.session_id, "session-456");
        assert_eq!(decoded.nonce, Some("nonce-789".to_string()));
    }
}
