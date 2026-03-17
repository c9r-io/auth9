use crate::error::Result;
use crate::identity_engine::{
    IdentityClientStore, IdentityProtocolMapperRepresentation, IdentitySamlClientRepresentation,
};
use crate::keycloak::{
    KeycloakClient, KeycloakOidcClient, KeycloakProtocolMapper, KeycloakSamlClient,
};
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

impl From<IdentityProtocolMapperRepresentation> for KeycloakProtocolMapper {
    fn from(value: IdentityProtocolMapperRepresentation) -> Self {
        Self {
            name: value.name,
            protocol: value.protocol,
            protocol_mapper: value.protocol_mapper,
            config: value.config,
        }
    }
}

impl From<IdentitySamlClientRepresentation> for KeycloakSamlClient {
    fn from(value: IdentitySamlClientRepresentation) -> Self {
        Self {
            id: value.id,
            client_id: value.client_id,
            name: value.name,
            enabled: value.enabled,
            protocol: value.protocol,
            base_url: value.base_url,
            redirect_uris: value.redirect_uris,
            attributes: value.attributes,
            protocol_mappers: value.protocol_mappers.into_iter().map(Into::into).collect(),
        }
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

    async fn create_saml_client(
        &self,
        client: &IdentitySamlClientRepresentation,
    ) -> Result<String> {
        self.client.create_saml_client(&client.clone().into()).await
    }

    async fn update_saml_client(
        &self,
        client_uuid: &str,
        client: &IdentitySamlClientRepresentation,
    ) -> Result<()> {
        self.client
            .update_saml_client(client_uuid, &client.clone().into())
            .await
    }

    async fn delete_saml_client(&self, client_uuid: &str) -> Result<()> {
        self.client.delete_saml_client(client_uuid).await
    }

    async fn get_saml_idp_descriptor(&self) -> Result<String> {
        self.client.get_saml_idp_descriptor().await
    }

    async fn get_active_signing_certificate(&self) -> Result<String> {
        let keys = self.client.get_realm_keys().await?;
        keys.keys
            .iter()
            .find(|k| {
                k.status.as_deref() == Some("ACTIVE")
                    && k.key_use.as_deref() == Some("SIG")
                    && k.key_type.as_deref() == Some("RSA")
            })
            .and_then(|k| k.certificate.clone())
            .ok_or_else(|| {
                crate::error::AppError::Internal(anyhow::anyhow!(
                    "No active RSA signing certificate found"
                ))
            })
    }

    fn saml_sso_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/saml",
            self.client.public_url(),
            self.client.realm()
        )
    }
}
