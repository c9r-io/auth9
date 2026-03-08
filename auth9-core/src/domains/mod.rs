//! Bounded-context modules grouping API, service, and route layers.

pub mod authorization;
pub mod identity;
pub mod integration;
pub mod platform;
pub mod provisioning;
pub mod security_observability;
pub mod tenant_access;

/// Aggregate trait for building the full HTTP router from bounded-context route modules.
///
/// This narrows server-level generics to a single bounded-context-centric bound while
/// keeping compatibility with existing `Has*` traits underneath.
pub trait BoundedContextRouterState:
    identity::context::IdentityContext
    + tenant_access::context::TenantAccessContext
    + authorization::context::AuthorizationContext
    + platform::context::PlatformContext
    + integration::context::IntegrationContext
    + security_observability::context::SecurityObservabilityContext
    + provisioning::context::ProvisioningContext
{
}

impl<T> BoundedContextRouterState for T where
    T: identity::context::IdentityContext
        + tenant_access::context::TenantAccessContext
        + authorization::context::AuthorizationContext
        + platform::context::PlatformContext
        + integration::context::IntegrationContext
        + security_observability::context::SecurityObservabilityContext
        + provisioning::context::ProvisioningContext
{
}

/// Legacy alias kept to avoid a broad rename across the codebase in one step.
pub trait DomainRouterState: BoundedContextRouterState {}

impl<T> DomainRouterState for T where T: BoundedContextRouterState {}
