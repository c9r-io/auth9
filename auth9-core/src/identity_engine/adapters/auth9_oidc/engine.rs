use super::{Auth9OidcFederationBrokerAdapter, Auth9OidcSessionStoreAdapter};
use crate::error::{AppError, Result};
use crate::identity_engine::{
    FederationBroker, IdentityClientStore, IdentityCredentialStore, IdentityEngine,
    IdentityEventSource, IdentitySessionStore, IdentityUserStore,
};
use crate::keycloak::{
    CreateKeycloakUserInput, KeycloakOidcClient, KeycloakUser, KeycloakUserCredential,
    KeycloakUserUpdate, RealmUpdate,
};
use anyhow::anyhow;
use async_trait::async_trait;

#[derive(Default)]
struct Auth9OidcUserStore;

#[async_trait]
impl IdentityUserStore for Auth9OidcUserStore {
    async fn create_user(&self, _input: &CreateKeycloakUserInput) -> Result<String> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc user create not implemented"
        )))
    }

    async fn get_user(&self, user_id: &str) -> Result<KeycloakUser> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc user '{}' get not implemented",
            user_id
        )))
    }

    async fn update_user(&self, user_id: &str, _input: &KeycloakUserUpdate) -> Result<()> {
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
}

#[derive(Default)]
struct Auth9OidcCredentialStore;

#[async_trait]
impl IdentityCredentialStore for Auth9OidcCredentialStore {
    async fn list_user_credentials(&self, _user_id: &str) -> Result<Vec<KeycloakUserCredential>> {
        Ok(Vec::new())
    }

    async fn remove_totp_credentials(&self, user_id: &str) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "auth9_oidc user '{}' totp cleanup not implemented",
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
