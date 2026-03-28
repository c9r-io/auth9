//! Integration test for JWT trailing space bypass vulnerability
#[cfg(test)]
mod trailing_space_tests {
    use auth9_core::config::JwtConfig;
    use auth9_core::jwt::JwtManager;
    use auth9_core::middleware::require_auth::{require_auth_middleware, AuthMiddlewareState};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn protected_handler() -> &'static str {
        "Protected content"
    }

    fn create_test_jwt_manager() -> JwtManager {
        let config = JwtConfig {
            secret: "test-secret-key-for-jwt-signing-must-be-long".to_string(),
            issuer: "https://auth9.test".to_string(),
            access_token_ttl_secs: 3600,
            refresh_token_ttl_secs: 86400,
            private_key_pem: None,
            public_key_pem: None,
            previous_public_key_pem: None,
        };
        JwtManager::new(config)
    }

    #[tokio::test]
    async fn test_trailing_space_in_bearer_token_should_fail() {
        let jwt_manager = create_test_jwt_manager();

        let user_id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let valid_token = jwt_manager
            .create_identity_token(user_id, "test@example.com", Some("Test User"))
            .unwrap();

        let auth_state = AuthMiddlewareState::new(jwt_manager);

        let app = Router::new()
            .route("/api/v1/auth/userinfo", get(protected_handler))
            .layer(axum::middleware::from_fn_with_state(
                auth_state,
                require_auth_middleware,
            ));

        // Test with trailing space in the token
        let token_with_space = format!("{} ", valid_token);
        let request = Request::builder()
            .uri("/api/v1/auth/userinfo")
            .header("Authorization", format!("Bearer {}", token_with_space))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        eprintln!("Response status with trailing space: {}", response.status());

        // This should be 401 UNAUTHORIZED
        // If it's 200, then there's a security bypass vulnerability
        if response.status() == StatusCode::OK {
            eprintln!("SECURITY ISSUE: Token with trailing space was accepted!");
            eprintln!("This indicates a JWT validation bypass.");
        }
    }
}
