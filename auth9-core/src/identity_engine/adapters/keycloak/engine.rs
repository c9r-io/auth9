use super::{
    KeycloakClientStoreAdapter, KeycloakCredentialStoreAdapter, KeycloakEventSourceAdapter,
    KeycloakFederationBrokerAdapter, KeycloakSessionStoreAdapter, KeycloakUserStoreAdapter,
};
use crate::error::{AppError, Result};
use crate::identity_engine::{
    FederationBroker, IdentityActionStore, IdentityClientStore, IdentityCredentialStore,
    IdentityEngine, IdentityEventSource, IdentitySessionStore, IdentityUserStore,
    IdentityVerificationStore, PendingActionInfo, VerificationTokenInfo,
};
use crate::keycloak::{KeycloakClient, RealmUpdate};
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;

/// No-op action store for Keycloak backend.
/// Required actions in Keycloak mode are handled by Keycloak itself.
#[derive(Default)]
struct KeycloakNoOpActionStore;

#[async_trait]
impl IdentityActionStore for KeycloakNoOpActionStore {
    async fn get_pending_actions(&self, _user_id: &str) -> Result<Vec<PendingActionInfo>> {
        Ok(Vec::new())
    }

    async fn create_action(
        &self,
        _user_id: &str,
        _action_type: &str,
        _metadata: Option<serde_json::Value>,
    ) -> Result<String> {
        Err(AppError::Internal(anyhow!(
            "pending actions not supported in keycloak backend"
        )))
    }

    async fn complete_action(&self, _action_id: &str) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "pending actions not supported in keycloak backend"
        )))
    }

    async fn cancel_action(&self, _action_id: &str) -> Result<()> {
        Err(AppError::Internal(anyhow!(
            "pending actions not supported in keycloak backend"
        )))
    }
}

/// No-op verification store for Keycloak backend.
/// Email verification in Keycloak mode is handled by Keycloak itself.
#[derive(Default)]
struct KeycloakNoOpVerificationStore;

#[async_trait]
impl IdentityVerificationStore for KeycloakNoOpVerificationStore {
    async fn get_verification_status(&self, _user_id: &str) -> Result<bool> {
        Ok(true) // Keycloak handles verification
    }

    async fn set_email_verified(&self, _user_id: &str, _verified: bool) -> Result<()> {
        Ok(())
    }

    async fn create_verification_token(
        &self,
        _user_id: &str,
        _token_hash: &str,
        _expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<VerificationTokenInfo> {
        Err(AppError::Internal(anyhow!(
            "verification tokens not supported in keycloak backend"
        )))
    }

    async fn find_valid_token(&self, _token_hash: &str) -> Result<Option<VerificationTokenInfo>> {
        Ok(None)
    }

    async fn mark_token_used(&self, _token_id: &str) -> Result<()> {
        Ok(())
    }

    async fn invalidate_user_tokens(&self, _user_id: &str) -> Result<u64> {
        Ok(0)
    }
}

pub struct KeycloakIdentityEngineAdapter {
    user_store: KeycloakUserStoreAdapter,
    client_store: KeycloakClientStoreAdapter,
    session_store: KeycloakSessionStoreAdapter,
    credential_store: KeycloakCredentialStoreAdapter,
    federation_broker: KeycloakFederationBrokerAdapter,
    event_source: KeycloakEventSourceAdapter,
    action_store: KeycloakNoOpActionStore,
    verification_store: KeycloakNoOpVerificationStore,
    client: Arc<KeycloakClient>,
}

impl KeycloakIdentityEngineAdapter {
    pub fn new(client: Arc<KeycloakClient>) -> Self {
        Self {
            user_store: KeycloakUserStoreAdapter::new(client.clone()),
            client_store: KeycloakClientStoreAdapter::new(client.clone()),
            session_store: KeycloakSessionStoreAdapter::new(client.clone()),
            credential_store: KeycloakCredentialStoreAdapter::new(client.clone()),
            federation_broker: KeycloakFederationBrokerAdapter::new(client.clone()),
            event_source: KeycloakEventSourceAdapter::new(client.clone()),
            action_store: KeycloakNoOpActionStore,
            verification_store: KeycloakNoOpVerificationStore,
            client,
        }
    }
}

#[async_trait]
impl IdentityEngine for KeycloakIdentityEngineAdapter {
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

    fn action_store(&self) -> &dyn IdentityActionStore {
        &self.action_store
    }

    fn verification_store(&self) -> &dyn IdentityVerificationStore {
        &self.verification_store
    }

    async fn update_realm(&self, settings: &RealmUpdate) -> Result<()> {
        self.client.update_realm(settings).await
    }
}
