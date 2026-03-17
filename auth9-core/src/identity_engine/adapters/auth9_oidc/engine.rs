use super::{Auth9OidcFederationBrokerAdapter, Auth9OidcSessionStoreAdapter};
use crate::error::{AppError, Result};
use crate::identity_engine::{
    FederationBroker, IdentityClientStore, IdentityCredentialRepresentation,
    IdentityCredentialStore, IdentityEngine, IdentityEventSource, IdentitySamlClientRepresentation,
    IdentitySessionStore, IdentityUserCreateInput, IdentityUserRepresentation, IdentityUserStore,
    IdentityUserUpdateInput,
};
use crate::keycloak::{KeycloakOidcClient, RealmUpdate};
use anyhow::anyhow;
use async_trait::async_trait;

#[derive(Default)]
struct Auth9OidcUserStore;

#[async_trait]
impl IdentityUserStore for Auth9OidcUserStore {
    async fn create_user(&self, _input: &IdentityUserCreateInput) -> Result<String> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc user create not implemented"
        )))
    }

    async fn get_user(&self, user_id: &str) -> Result<IdentityUserRepresentation> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc user '{}' get not implemented",
            user_id
        )))
    }

    async fn update_user(&self, user_id: &str, _input: &IdentityUserUpdateInput) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc user '{}' update not implemented",
            user_id
        )))
    }

    async fn delete_user(&self, user_id: &str) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc user '{}' delete not implemented",
            user_id
        )))
    }

    async fn set_user_password(
        &self,
        user_id: &str,
        _password: &str,
        _temporary: bool,
    ) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc user '{}' password set not implemented",
            user_id
        )))
    }

    async fn admin_set_user_password(
        &self,
        user_id: &str,
        _password: &str,
        _temporary: bool,
    ) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc user '{}' admin password set not implemented",
            user_id
        )))
    }

    async fn validate_user_password(&self, user_id: &str, _password: &str) -> Result<bool> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc user '{}' password validation not implemented",
            user_id
        )))
    }
}

#[derive(Default)]
struct Auth9OidcClientStore;

#[async_trait]
impl IdentityClientStore for Auth9OidcClientStore {
    async fn create_oidc_client(&self, _client: &KeycloakOidcClient) -> Result<String> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc oidc client create not implemented"
        )))
    }

    async fn get_client_secret(&self, client_uuid: &str) -> Result<String> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc client '{}' secret lookup not implemented",
            client_uuid
        )))
    }

    async fn regenerate_client_secret(&self, client_uuid: &str) -> Result<String> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc client '{}' secret regeneration not implemented",
            client_uuid
        )))
    }

    async fn get_client_uuid_by_client_id(&self, client_id: &str) -> Result<String> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc client '{}' lookup not implemented",
            client_id
        )))
    }

    async fn get_client_by_client_id(&self, client_id: &str) -> Result<KeycloakOidcClient> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc client '{}' fetch not implemented",
            client_id
        )))
    }

    async fn update_oidc_client(
        &self,
        client_uuid: &str,
        _client: &KeycloakOidcClient,
    ) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc client '{}' update not implemented",
            client_uuid
        )))
    }

    async fn delete_oidc_client(&self, client_uuid: &str) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc client '{}' delete not implemented",
            client_uuid
        )))
    }

    async fn create_saml_client(
        &self,
        _client: &IdentitySamlClientRepresentation,
    ) -> Result<String> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc saml client create not implemented"
        )))
    }

    async fn update_saml_client(
        &self,
        client_uuid: &str,
        _client: &IdentitySamlClientRepresentation,
    ) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc saml client '{}' update not implemented",
            client_uuid
        )))
    }

    async fn delete_saml_client(&self, client_uuid: &str) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc saml client '{}' delete not implemented",
            client_uuid
        )))
    }

    async fn get_saml_idp_descriptor(&self) -> Result<String> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc saml descriptor not implemented"
        )))
    }

    async fn get_active_signing_certificate(&self) -> Result<String> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc signing certificate lookup not implemented"
        )))
    }

    fn saml_sso_url(&self) -> String {
        String::new()
    }
}

#[derive(Default)]
struct Auth9OidcCredentialStore;

#[async_trait]
impl IdentityCredentialStore for Auth9OidcCredentialStore {
    async fn list_user_credentials(
        &self,
        _user_id: &str,
    ) -> Result<Vec<IdentityCredentialRepresentation>> {
        Ok(Vec::new())
    }

    async fn remove_totp_credentials(&self, user_id: &str) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc user '{}' totp cleanup not implemented",
            user_id
        )))
    }

    async fn list_webauthn_credentials(
        &self,
        _user_id: &str,
    ) -> Result<Vec<IdentityCredentialRepresentation>> {
        Ok(Vec::new())
    }

    async fn delete_user_credential(&self, user_id: &str, _credential_id: &str) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc user '{}' credential deletion not implemented",
            user_id
        )))
    }
}

#[derive(Default)]
struct Auth9OidcEventSource;

#[async_trait]
impl IdentityEventSource for Auth9OidcEventSource {}

pub struct Auth9OidcIdentityEngineAdapter {
    user_store: Auth9OidcUserStore,
    client_store: Auth9OidcClientStore,
    session_store: Auth9OidcSessionStoreAdapter,
    credential_store: Auth9OidcCredentialStore,
    federation_broker: Auth9OidcFederationBrokerAdapter,
    event_source: Auth9OidcEventSource,
}

impl Auth9OidcIdentityEngineAdapter {
    pub fn new() -> Self {
        Self {
            user_store: Auth9OidcUserStore,
            client_store: Auth9OidcClientStore,
            session_store: Auth9OidcSessionStoreAdapter::new(),
            credential_store: Auth9OidcCredentialStore,
            federation_broker: Auth9OidcFederationBrokerAdapter::new(),
            event_source: Auth9OidcEventSource,
        }
    }
}

#[async_trait]
impl IdentityEngine for Auth9OidcIdentityEngineAdapter {
    fn user_store(&self) -> &dyn IdentityUserStore {
        &self.user_store
    }

    fn client_store(&self) -> &dyn IdentityClientStore {
        &self.client_store
    }

    fn session_store(&self) -> &dyn IdentitySessionStore {
        &self.session_store
    }

    fn credential_store(&self) -> &dyn IdentityCredentialStore {
        &self.credential_store
    }

    fn federation_broker(&self) -> &dyn FederationBroker {
        &self.federation_broker
    }

    fn event_source(&self) -> &dyn IdentityEventSource {
        &self.event_source
    }

    async fn update_realm(&self, _settings: &RealmUpdate) -> Result<()> {
        Ok(())
    }
}
