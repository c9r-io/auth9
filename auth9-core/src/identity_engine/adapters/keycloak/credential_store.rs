use crate::error::Result;
use crate::identity_engine::IdentityCredentialStore;
use crate::keycloak::{KeycloakClient, KeycloakUserCredential};
use async_trait::async_trait;
use std::sync::Arc;

pub struct KeycloakCredentialStoreAdapter {
    #[allow(dead_code)]
    client: Arc<KeycloakClient>,
}

impl KeycloakCredentialStoreAdapter {
    pub fn new(client: Arc<KeycloakClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IdentityCredentialStore for KeycloakCredentialStoreAdapter {
    async fn list_user_credentials(&self, user_id: &str) -> Result<Vec<KeycloakUserCredential>> {
        self.client.list_user_credentials(user_id).await
    }

    async fn remove_totp_credentials(&self, user_id: &str) -> Result<()> {
        self.client.remove_totp_credentials(user_id).await
    }
}
