use crate::error::Result;
use crate::identity_engine::{IdentityCredentialRepresentation, IdentityCredentialStore};
use crate::keycloak::{KeycloakClient, KeycloakCredentialRepresentation, KeycloakUserCredential};
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

impl From<KeycloakUserCredential> for IdentityCredentialRepresentation {
    fn from(value: KeycloakUserCredential) -> Self {
        Self {
            id: value.id,
            credential_type: value.credential_type,
            user_label: None,
            created_date: None,
        }
    }
}

impl From<KeycloakCredentialRepresentation> for IdentityCredentialRepresentation {
    fn from(value: KeycloakCredentialRepresentation) -> Self {
        Self {
            id: value.id,
            credential_type: value.credential_type,
            user_label: value.user_label,
            created_date: value.created_date,
        }
    }
}

#[async_trait]
impl IdentityCredentialStore for KeycloakCredentialStoreAdapter {
    async fn list_user_credentials(
        &self,
        user_id: &str,
    ) -> Result<Vec<IdentityCredentialRepresentation>> {
        Ok(self
            .client
            .list_user_credentials(user_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn remove_totp_credentials(&self, user_id: &str) -> Result<()> {
        self.client.remove_totp_credentials(user_id).await
    }

    async fn list_webauthn_credentials(
        &self,
        user_id: &str,
    ) -> Result<Vec<IdentityCredentialRepresentation>> {
        Ok(self
            .client
            .list_webauthn_credentials(user_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    async fn delete_user_credential(&self, user_id: &str, credential_id: &str) -> Result<()> {
        self.client
            .delete_user_credential(user_id, credential_id)
            .await
    }
}
