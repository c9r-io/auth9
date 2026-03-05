//! OpenID Connect discovery and JWKS endpoints.

use crate::error::Result;
use crate::state::HasServices;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use base64::Engine;
use rsa::pkcs8::DecodePublicKey;
use rsa::traits::PublicKeyParts;
use rsa::RsaPublicKey;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// OpenID Connect Discovery endpoint
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OpenIdConfiguration {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: Option<String>,
    pub end_session_endpoint: String,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub claims_supported: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct Jwks {
    pub keys: Vec<JwkKey>,
}

#[derive(Debug, Serialize)]
pub(super) struct JwkKey {
    pub kty: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub alg: String,
    pub kid: String,
    pub n: String,
    pub e: String,
}

#[utoipa::path(
    get,
    path = "/.well-known/openid-configuration",
    tag = "Identity",
    responses(
        (status = 200, description = "OpenID configuration")
    )
)]
pub async fn openid_configuration<S: HasServices>(State(state): State<S>) -> impl IntoResponse {
    let base_url = &state.config().jwt.issuer;
    // Always include jwks_uri - it returns empty keys array for HS256 mode
    let jwks_uri = Some(format!("{}/.well-known/jwks.json", base_url));
    let algs = if state.jwt_manager().uses_rsa() {
        vec!["RS256".to_string()]
    } else {
        vec!["HS256".to_string()]
    };

    Json(OpenIdConfiguration {
        issuer: base_url.clone(),
        authorization_endpoint: format!("{}/api/v1/auth/authorize", base_url),
        token_endpoint: format!("{}/api/v1/auth/token", base_url),
        userinfo_endpoint: format!("{}/api/v1/auth/userinfo", base_url),
        jwks_uri,
        end_session_endpoint: format!("{}/api/v1/auth/logout", base_url),
        response_types_supported: vec![
            "code".to_string(),
            "token".to_string(),
            "id_token".to_string(),
        ],
        grant_types_supported: vec![
            "authorization_code".to_string(),
            "client_credentials".to_string(),
            "refresh_token".to_string(),
        ],
        subject_types_supported: vec!["public".to_string()],
        id_token_signing_alg_values_supported: algs,
        scopes_supported: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        token_endpoint_auth_methods_supported: vec![
            "client_secret_basic".to_string(),
            "client_secret_post".to_string(),
        ],
        claims_supported: vec![
            "sub".to_string(),
            "email".to_string(),
            "name".to_string(),
            "iss".to_string(),
            "aud".to_string(),
            "exp".to_string(),
            "iat".to_string(),
        ],
    })
}

#[utoipa::path(
    get,
    path = "/.well-known/jwks.json",
    tag = "Identity",
    responses(
        (status = 200, description = "JSON Web Key Set")
    )
)]
pub async fn jwks<S: HasServices>(State(state): State<S>) -> impl IntoResponse {
    let public_key_pem = match state.jwt_manager().public_key_pem() {
        Some(key) => key,
        None => {
            // Return empty JWKS for HS256 mode (symmetric keys are not exposed via JWKS)
            return Json(Jwks { keys: vec![] }).into_response();
        }
    };

    let public_key = match RsaPublicKey::from_public_key_pem(public_key_pem) {
        Ok(key) => key,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let n = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
    let e = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());

    let mut keys = vec![JwkKey {
        kty: "RSA".to_string(),
        use_: "sig".to_string(),
        alg: "RS256".to_string(),
        kid: "auth9-current".to_string(),
        n,
        e,
    }];

    // Include previous key for rotation support (allows verifying tokens signed with old key)
    if let Some(prev_pem) = state.jwt_manager().previous_public_key_pem() {
        if let Ok(prev_key) = RsaPublicKey::from_public_key_pem(prev_pem) {
            let prev_n =
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(prev_key.n().to_bytes_be());
            let prev_e =
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(prev_key.e().to_bytes_be());
            keys.push(JwkKey {
                kty: "RSA".to_string(),
                use_: "sig".to_string(),
                alg: "RS256".to_string(),
                kid: "auth9-previous".to_string(),
                n: prev_n,
                e: prev_e,
            });
        }
    }

    Json(Jwks { keys }).into_response()
}

// Suppress unused import warning — `Result` is needed by the utoipa macro expansion
#[allow(unused_imports)]
use Result as _Result;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openid_configuration_structure() {
        let config = OpenIdConfiguration {
            issuer: "https://auth9.example.com".to_string(),
            authorization_endpoint: "https://auth9.example.com/api/v1/auth/authorize".to_string(),
            token_endpoint: "https://auth9.example.com/api/v1/auth/token".to_string(),
            userinfo_endpoint: "https://auth9.example.com/api/v1/auth/userinfo".to_string(),
            jwks_uri: Some("https://auth9.example.com/.well-known/jwks.json".to_string()),
            end_session_endpoint: "https://auth9.example.com/api/v1/auth/logout".to_string(),
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec!["authorization_code".to_string()],
            subject_types_supported: vec!["public".to_string()],
            id_token_signing_alg_values_supported: vec!["RS256".to_string()],
            scopes_supported: vec!["openid".to_string()],
            token_endpoint_auth_methods_supported: vec!["client_secret_post".to_string()],
            claims_supported: vec!["sub".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("issuer"));
        assert!(json.contains("authorization_endpoint"));
        assert!(json.contains("jwks_uri"));
    }

    #[test]
    fn test_openid_configuration_without_jwks() {
        let config = OpenIdConfiguration {
            issuer: "https://auth9.example.com".to_string(),
            authorization_endpoint: "https://auth9.example.com/api/v1/auth/authorize".to_string(),
            token_endpoint: "https://auth9.example.com/api/v1/auth/token".to_string(),
            userinfo_endpoint: "https://auth9.example.com/api/v1/auth/userinfo".to_string(),
            jwks_uri: None,
            end_session_endpoint: "https://auth9.example.com/api/v1/auth/logout".to_string(),
            response_types_supported: vec![],
            grant_types_supported: vec![],
            subject_types_supported: vec![],
            id_token_signing_alg_values_supported: vec![],
            scopes_supported: vec![],
            token_endpoint_auth_methods_supported: vec![],
            claims_supported: vec![],
        };

        assert!(config.jwks_uri.is_none());
    }

    #[test]
    fn test_openid_configuration_serialization() {
        let config = OpenIdConfiguration {
            issuer: "https://test.example.com".to_string(),
            authorization_endpoint: "https://test.example.com/auth".to_string(),
            token_endpoint: "https://test.example.com/token".to_string(),
            userinfo_endpoint: "https://test.example.com/userinfo".to_string(),
            jwks_uri: Some("https://test.example.com/jwks".to_string()),
            end_session_endpoint: "https://test.example.com/logout".to_string(),
            response_types_supported: vec![
                "code".to_string(),
                "token".to_string(),
                "id_token".to_string(),
            ],
            grant_types_supported: vec![
                "authorization_code".to_string(),
                "client_credentials".to_string(),
            ],
            subject_types_supported: vec!["public".to_string()],
            id_token_signing_alg_values_supported: vec!["RS256".to_string(), "HS256".to_string()],
            scopes_supported: vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
            ],
            token_endpoint_auth_methods_supported: vec![
                "client_secret_basic".to_string(),
                "client_secret_post".to_string(),
            ],
            claims_supported: vec!["sub".to_string(), "email".to_string(), "name".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: OpenIdConfiguration = serde_json::from_str(&json).unwrap();

        assert_eq!(config.issuer, parsed.issuer);
        assert_eq!(
            config.response_types_supported.len(),
            parsed.response_types_supported.len()
        );
    }

    #[test]
    fn test_openid_configuration_deserialization() {
        let json = r#"{
            "issuer": "https://test.com",
            "authorization_endpoint": "https://test.com/auth",
            "token_endpoint": "https://test.com/token",
            "userinfo_endpoint": "https://test.com/userinfo",
            "jwks_uri": "https://test.com/jwks",
            "end_session_endpoint": "https://test.com/logout",
            "response_types_supported": ["code"],
            "grant_types_supported": ["authorization_code"],
            "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["RS256"],
            "scopes_supported": ["openid"],
            "token_endpoint_auth_methods_supported": ["client_secret_post"],
            "claims_supported": ["sub"]
        }"#;

        let config: OpenIdConfiguration = serde_json::from_str(json).unwrap();
        assert_eq!(config.issuer, "https://test.com");
        assert_eq!(config.jwks_uri, Some("https://test.com/jwks".to_string()));
    }

    #[test]
    fn test_jwks_key_structure() {
        let key = JwkKey {
            kty: "RSA".to_string(),
            use_: "sig".to_string(),
            alg: "RS256".to_string(),
            kid: "key-1".to_string(),
            n: "modulus".to_string(),
            e: "AQAB".to_string(),
        };

        let json = serde_json::to_string(&key).unwrap();
        assert!(json.contains("\"kty\":\"RSA\""));
        assert!(json.contains("\"use\":\"sig\""));
        assert!(json.contains("\"alg\":\"RS256\""));
        assert!(json.contains("\"kid\":\"key-1\""));
        assert!(json.contains("\"n\":\"modulus\""));
        assert!(json.contains("\"e\":\"AQAB\""));
    }

    #[test]
    fn test_jwks_structure() {
        let jwks = Jwks {
            keys: vec![JwkKey {
                kty: "RSA".to_string(),
                use_: "sig".to_string(),
                alg: "RS256".to_string(),
                kid: "default".to_string(),
                n: "n".to_string(),
                e: "e".to_string(),
            }],
        };

        let json = serde_json::to_string(&jwks).unwrap();
        assert!(json.contains("\"keys\""));
        assert!(json.contains("RSA"));
    }

    #[test]
    fn test_jwks_empty_keys() {
        let jwks = Jwks { keys: vec![] };
        let json = serde_json::to_string(&jwks).unwrap();
        assert!(json.contains("\"keys\":[]"));
    }
}
