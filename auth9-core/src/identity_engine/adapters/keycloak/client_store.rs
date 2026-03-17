use crate::error::Result;
use crate::identity_engine::IdentityClientStore;
use crate::keycloak::{KeycloakClient, KeycloakOidcClient};
use async_trait::async_trait;
use std::sync::Arc;

pub struct KeycloakClientStoreAdapter {
    #[allow(dead_code)]
    client: Arc<KeycloakClient>,
}

impl KeycloakClientStoreAdapter {
    pub fn new(client: Arc<KeycloakClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IdentityClientStore for KeycloakClientStoreAdapter {
    async fn create_oidc_client(&self, client: &KeycloakOidcClient) -> Result<String> {
        self.client.create_oidc_client(client).await
    }

    async fn get_client_secret(&self, client_uuid: &str) -> Result<String> {
        self.client.get_client_secret(client_uuid).await
    }

    async fn regenerate_client_secret(&self, client_uuid: &str) -> Result<String> {
        self.client.regenerate_client_secret(client_uuid).await
    }

    async fn get_client_uuid_by_client_id(&self, client_id: &str) -> Result<String> {
        self.client.get_client_uuid_by_client_id(client_id).await
    }

    async fn get_client_by_client_id(&self, client_id: &str) -> Result<KeycloakOidcClient> {
        self.client.get_client_by_client_id(client_id).await
    }

    async fn update_oidc_client(
        &self,
        client_uuid: &str,
        client: &KeycloakOidcClient,
    ) -> Result<()> {
        self.client.update_oidc_client(client_uuid, client).await
    }

    async fn delete_oidc_client(&self, client_uuid: &str) -> Result<()> {
        self.client.delete_oidc_client(client_uuid).await
    }
}
