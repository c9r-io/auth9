//! OAuth 2.0 token endpoint error responses per RFC 6749 Section 5.2.
//!
//! These errors are used exclusively by the `/api/v1/auth/token` endpoint
//! to return spec-compliant error codes (`invalid_grant`, `invalid_request`, etc.)
//! with the `error` + `error_description` JSON shape required by the spec.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// OAuth 2.0 error codes for the token endpoint (RFC 6749 Section 5.2).
#[derive(Debug)]
pub enum OAuthTokenError {
    /// The request is missing a required parameter, includes an unsupported
    /// parameter value, or is otherwise malformed.
    InvalidRequest(String),

    /// Client authentication failed.
    InvalidClient(String),

    /// The provided authorization grant (authorization code, refresh token, etc.)
    /// is invalid, expired, revoked, or does not match the redirection URI.
    InvalidGrant(String),

    /// The authenticated client is not authorized to use this grant type.
    UnauthorizedClient(String),

    /// The authorization grant type is not supported by the authorization server.
    UnsupportedGrantType(String),

    /// The requested scope is invalid, unknown, or malformed.
    InvalidScope(String),

    /// Internal server error (maps cache/DB failures).
    ServerError(String),
}

/// RFC 6749 Section 5.2 error response body.
#[derive(Serialize)]
struct OAuthErrorResponse {
    error: &'static str,
    error_description: String,
}

impl OAuthTokenError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidRequest(_) => "invalid_request",
            Self::InvalidClient(_) => "invalid_client",
            Self::InvalidGrant(_) => "invalid_grant",
            Self::UnauthorizedClient(_) => "unauthorized_client",
            Self::UnsupportedGrantType(_) => "unsupported_grant_type",
            Self::InvalidScope(_) => "invalid_scope",
            Self::ServerError(_) => "server_error",
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidClient(_) => StatusCode::UNAUTHORIZED,
            Self::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            // All other OAuth errors are 400 per RFC 6749
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn description(&self) -> &str {
        match self {
            Self::InvalidRequest(d)
            | Self::InvalidClient(d)
            | Self::InvalidGrant(d)
            | Self::UnauthorizedClient(d)
            | Self::UnsupportedGrantType(d)
            | Self::InvalidScope(d)
            | Self::ServerError(d) => d,
        }
    }
}

impl IntoResponse for OAuthTokenError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(OAuthErrorResponse {
            error: self.error_code(),
            error_description: self.description().to_string(),
        });
        (status, body).into_response()
    }
}

impl std::fmt::Display for OAuthTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error_code(), self.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(
            OAuthTokenError::InvalidRequest("x".into()).error_code(),
            "invalid_request"
        );
        assert_eq!(
            OAuthTokenError::InvalidClient("x".into()).error_code(),
            "invalid_client"
        );
        assert_eq!(
            OAuthTokenError::InvalidGrant("x".into()).error_code(),
            "invalid_grant"
        );
        assert_eq!(
            OAuthTokenError::UnsupportedGrantType("x".into()).error_code(),
            "unsupported_grant_type"
        );
        assert_eq!(
            OAuthTokenError::InvalidScope("x".into()).error_code(),
            "invalid_scope"
        );
        assert_eq!(
            OAuthTokenError::ServerError("x".into()).error_code(),
            "server_error"
        );
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(
            OAuthTokenError::InvalidRequest("x".into()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            OAuthTokenError::InvalidClient("x".into()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            OAuthTokenError::InvalidGrant("x".into()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            OAuthTokenError::UnsupportedGrantType("x".into()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            OAuthTokenError::ServerError("x".into()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn test_into_response_format() {
        let err = OAuthTokenError::InvalidGrant("Invalid or expired authorization code".into());
        let response = err.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["error"], "invalid_grant");
        assert_eq!(
            json["error_description"],
            "Invalid or expired authorization code"
        );
        // Must NOT have "message" or "details" fields (auth9 format)
        assert!(json.get("message").is_none());
        assert!(json.get("details").is_none());
    }

    #[tokio::test]
    async fn test_invalid_client_is_401() {
        let err = OAuthTokenError::InvalidClient("Client authentication failed".into());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_display() {
        let err = OAuthTokenError::InvalidGrant("bad token".into());
        assert_eq!(err.to_string(), "invalid_grant: bad token");
    }
}
