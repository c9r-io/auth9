use crate::error::Result;
use crate::identity_engine::{
    IdentityCredentialInput, IdentityUserCreateInput, IdentityUserRepresentation,
    IdentityUserStore, IdentityUserUpdateInput,
};
use crate::keycloak::{
    CreateKeycloakUserInput, KeycloakClient, KeycloakCredential, KeycloakUser, KeycloakUserUpdate,
};
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

impl From<IdentityCredentialInput> for KeycloakCredential {
    fn from(value: IdentityCredentialInput) -> Self {
        Self {
            credential_type: value.credential_type,
            value: value.value,
            temporary: value.temporary,
        }
    }
}

impl From<IdentityUserCreateInput> for CreateKeycloakUserInput {
    fn from(value: IdentityUserCreateInput) -> Self {
        Self {
            username: value.username,
            email: value.email,
            first_name: value.first_name,
            last_name: value.last_name,
            enabled: value.enabled,
            email_verified: value.email_verified,
            credentials: value
                .credentials
                .map(|credentials| credentials.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<KeycloakUser> for IdentityUserRepresentation {
    fn from(value: KeycloakUser) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            first_name: value.first_name,
            last_name: value.last_name,
            enabled: value.enabled,
            email_verified: value.email_verified,
            attributes: value.attributes,
        }
    }
}

impl From<IdentityUserUpdateInput> for KeycloakUserUpdate {
    fn from(value: IdentityUserUpdateInput) -> Self {
        Self {
            username: value.username,
            email: value.email,
            first_name: value.first_name,
            last_name: value.last_name,
            enabled: value.enabled,
            email_verified: value.email_verified,
            required_actions: value.required_actions,
        }
    }
}

#[async_trait]
impl IdentityUserStore for KeycloakUserStoreAdapter {
    async fn create_user(&self, input: &IdentityUserCreateInput) -> Result<String> {
        self.client.create_user(&input.clone().into()).await
    }

    async fn get_user(&self, user_id: &str) -> Result<IdentityUserRepresentation> {
        Ok(self.client.get_user(user_id).await?.into())
    }

    async fn update_user(&self, user_id: &str, input: &IdentityUserUpdateInput) -> Result<()> {
        self.client
            .update_user(user_id, &input.clone().into())
            .await
    }

    async fn delete_user(&self, user_id: &str) -> Result<()> {
        self.client.delete_user(user_id).await
    }

    async fn set_user_password(
        &self,
        user_id: &str,
        password: &str,
        temporary: bool,
    ) -> Result<()> {
        self.client
            .reset_user_password(user_id, password, temporary)
            .await
    }

    async fn admin_set_user_password(
        &self,
        user_id: &str,
        password: &str,
        temporary: bool,
    ) -> Result<()> {
        self.client
            .admin_reset_user_password(user_id, password, temporary)
            .await
    }

    async fn validate_user_password(&self, user_id: &str, password: &str) -> Result<bool> {
        self.client.validate_user_password(user_id, password).await
    }
}
