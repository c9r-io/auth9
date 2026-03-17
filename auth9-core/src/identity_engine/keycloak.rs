use super::{
    FederationBroker, IdentityClientStore, IdentityCredentialStore, IdentityEngine,
    IdentityEventSource, IdentitySessionStore, IdentityUserStore,
};
use crate::error::Result;
use crate::keycloak::{
    KeycloakClient, KeycloakFederatedIdentity, KeycloakIdentityProvider, RealmUpdate,
};
use async_trait::async_trait;

#[async_trait]
impl IdentityUserStore for KeycloakClient {}

#[async_trait]
impl IdentityClientStore for KeycloakClient {}

#[async_trait]
impl IdentityCredentialStore for KeycloakClient {}

#[async_trait]
impl IdentityEventSource for KeycloakClient {}

#[async_trait]
impl IdentitySessionStore for KeycloakClient {
    async fn delete_user_session(&self, session_id: &str) -> Result<()> {
        KeycloakClient::delete_user_session(self, session_id).await
    }

    async fn logout_user(&self, user_id: &str) -> Result<()> {
        KeycloakClient::logout_user(self, user_id).await
    }
}

#[async_trait]
impl FederationBroker for KeycloakClient {
    async fn list_identity_providers(&self) -> Result<Vec<KeycloakIdentityProvider>> {
        KeycloakClient::list_identity_providers(self).await
    }

    async fn get_identity_provider(&self, alias: &str) -> Result<KeycloakIdentityProvider> {
        KeycloakClient::get_identity_provider(self, alias).await
    }

    async fn create_identity_provider(&self, provider: &KeycloakIdentityProvider) -> Result<()> {
        KeycloakClient::create_identity_provider(self, provider).await
    }

    async fn update_identity_provider(
        &self,
        alias: &str,
        provider: &KeycloakIdentityProvider,
    ) -> Result<()> {
        KeycloakClient::update_identity_provider(self, alias, provider).await
    }

    async fn delete_identity_provider(&self, alias: &str) -> Result<()> {
        KeycloakClient::delete_identity_provider(self, alias).await
    }

    async fn get_user_federated_identities(
        &self,
        user_id: &str,
    ) -> Result<Vec<KeycloakFederatedIdentity>> {
        KeycloakClient::get_user_federated_identities(self, user_id).await
    }

    async fn remove_user_federated_identity(
        &self,
        user_id: &str,
        provider_alias: &str,
    ) -> Result<()> {
        KeycloakClient::remove_user_federated_identity(self, user_id, provider_alias).await
    }
}

#[async_trait]
impl IdentityEngine for KeycloakClient {
    fn user_store(&self) -> &dyn IdentityUserStore {
        self
    }

    fn client_store(&self) -> &dyn IdentityClientStore {
        self
    }

    fn session_store(&self) -> &dyn IdentitySessionStore {
        self
    }

    fn credential_store(&self) -> &dyn IdentityCredentialStore {
        self
    }

    fn federation_broker(&self) -> &dyn FederationBroker {
        self
    }

    fn event_source(&self) -> &dyn IdentityEventSource {
        self
    }

    async fn update_realm(&self, settings: &RealmUpdate) -> Result<()> {
        KeycloakClient::update_realm(self, settings).await
    }
}
