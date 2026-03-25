use crate::state::{
    HasAdaptiveMfa, HasAnalytics, HasBranding, HasCache, HasDbPool, HasEmailVerification,
    HasIdentityProviders, HasLdapAuth, HasMfa, HasPasswordManagement, HasRequiredActions,
    HasServices, HasSessionManagement, HasSystemSettings, HasTrustedDevices, HasWebAuthn,
};

pub trait IdentityContext:
    HasServices
    + HasCache
    + HasPasswordManagement
    + HasSessionManagement
    + HasWebAuthn
    + HasIdentityProviders
    + HasAnalytics
    + HasDbPool
    + HasSystemSettings
    + HasBranding
    + HasEmailVerification
    + HasRequiredActions
    + HasMfa
    + HasLdapAuth
    + HasTrustedDevices
    + HasAdaptiveMfa
{
}

impl<T> IdentityContext for T where
    T: HasServices
        + HasCache
        + HasPasswordManagement
        + HasSessionManagement
        + HasWebAuthn
        + HasIdentityProviders
        + HasAnalytics
        + HasDbPool
        + HasSystemSettings
        + HasBranding
        + HasEmailVerification
        + HasRequiredActions
        + HasMfa
        + HasLdapAuth
        + HasTrustedDevices
        + HasAdaptiveMfa
{
}
