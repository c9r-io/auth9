use crate::error::Result;
use crate::identity_engine::IdentityUserStore;
use crate::keycloak::{CreateKeycloakUserInput, KeycloakClient, KeycloakUser, KeycloakUserUpdate};
use async_trait::async_trait;
use std::sync::Arc;

pub struct KeycloakUserStoreAdapter {
    #[allow(dead_code)]
    client: Arc<KeycloakClient>,
}

impl KeycloakUserStoreAdapter {
    pub fn new(client: Arc<KeycloakClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IdentityUserStore for KeycloakUserStoreAdapter {
    async fn create_user(&self, input: &CreateKeycloakUserInput) -> Result<String> {
        self.client.create_user(input).await
    }

    async fn get_user(&self, user_id: &str) -> Result<KeycloakUser> {
        self.client.get_user(user_id).await
    }

    async fn update_user(&self, user_id: &str, input: &KeycloakUserUpdate) -> Result<()> {
        self.client.update_user(user_id, input).await
    }

    async fn delete_user(&self, user_id: &str) -> Result<()> {
        self.client.delete_user(user_id).await
    }

    async fn validate_user_password(&self, user_id: &str, password: &str) -> Result<bool> {
        self.client.validate_user_password(user_id, password).await
    }
}
