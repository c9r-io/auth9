//! OpenAPI 3.0 documentation assembly
//!
//! Aggregates all handler path annotations and domain schemas into a single
//! OpenAPI specification. Swagger UI and ReDoc are served in non-production
//! environments.

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Auth9 Core API",
        version = "0.1.0",
        description = "Auth9 Identity & Access Management Service API",
        license(name = "Proprietary"),
        contact(name = "Auth9 Team")
    ),
    tags(
        (name = "System", description = "Health checks and system status"),
        (name = "Identity", description = "Authentication, sessions, passwords, WebAuthn, and identity providers"),
        (name = "Tenant Access", description = "Tenants, users, invitations, organizations, and SSO connectors"),
        (name = "Authorization", description = "Services, RBAC roles, permissions, and tenant-service associations"),
        (name = "Platform", description = "System settings, email configuration, branding, and email templates"),
        (name = "Integration", description = "Webhooks, actions, and Keycloak event ingestion"),
        (name = "Security & Observability", description = "Audit logs, analytics, and security alerts"),
    ),
    security(
        ("bearer_jwt" = [])
    ),
    components(
        // Security schemes
        schemas(
            // ── Shared response types ──────────────────────────────────
            crate::http_support::PaginationQuery,
            crate::http_support::PaginationMeta,
            crate::http_support::MessageResponse,

            // ── Common ─────────────────────────────────────────────────
            crate::domain::common::StringUuid,

            // ── Tenant domain ──────────────────────────────────────────
            crate::domain::tenant::Tenant,
            crate::domain::tenant::TenantStatus,
            crate::domain::tenant::TenantSettings,
            crate::domain::tenant::TenantBranding,
            crate::domain::tenant::CreateTenantInput,
            crate::domain::tenant::CreateOrganizationInput,
            crate::domain::tenant::UpdateTenantInput,
            crate::domain::tenant::TenantServiceAssoc,
            crate::domain::tenant::ServiceWithStatus,
            crate::domain::tenant::ToggleServiceInput,

            // ── User domain ────────────────────────────────────────────
            crate::domain::user::User,
            crate::domain::user::TenantUser,
            crate::domain::user::CreateUserInput,
            crate::domain::user::UpdateUserInput,
            crate::domain::user::AddUserToTenantInput,
            crate::domain::user::AdminSetPasswordInput,
            crate::domain::user::UserTenantInfo,
            crate::domain::user::TenantUserWithTenant,
            crate::domain::user::TenantInfo,

            // ── Service / Client domain ────────────────────────────────
            crate::domain::service::Service,
            crate::domain::service::ServiceStatus,
            crate::domain::service::Client,
            crate::domain::service::CreateServiceInput,
            crate::domain::service::CreateClientInput,
            crate::domain::service::UpdateServiceInput,

            // ── RBAC domain ────────────────────────────────────────────
            crate::domain::rbac::Permission,
            crate::domain::rbac::Role,
            crate::domain::rbac::RolePermission,
            crate::domain::rbac::UserTenantRole,
            crate::domain::rbac::CreatePermissionInput,
            crate::domain::rbac::CreateRoleInput,
            crate::domain::rbac::UpdateRoleInput,
            crate::domain::rbac::AssignRolesInput,
            crate::domain::rbac::UserRolesInTenant,
            crate::domain::abac::AbacMode,
            crate::domain::abac::AbacEffect,
            crate::domain::abac::AbacRule,
            crate::domain::abac::AbacPolicyDocument,
            crate::domain::abac::AbacPolicySetSummary,
            crate::domain::abac::AbacPolicyVersionSummary,
            crate::domain::abac::AbacSimulationInput,
            crate::domain::abac::AbacSimulationResult,
            crate::domains::authorization::api::abac::CreateAbacPolicyInput,
            crate::domains::authorization::api::abac::UpdateAbacPolicyInput,
            crate::domains::authorization::api::abac::PublishAbacPolicyInput,
            crate::domains::authorization::api::abac::RollbackAbacPolicyInput,
            crate::domains::authorization::api::abac::SimulateAbacPolicyInput,

            // ── Invitation domain ──────────────────────────────────────
            crate::domain::invitation::InvitationStatus,
            crate::domain::invitation::Invitation,
            crate::domain::invitation::CreateInvitationInput,
            crate::domain::invitation::InvitationResponse,
            crate::domain::invitation::AcceptInvitationInput,

            // ── Password domain ────────────────────────────────────────
            crate::domain::password::PasswordPolicy,
            crate::domain::password::ForgotPasswordInput,
            crate::domain::password::ResetPasswordInput,
            crate::domain::password::ChangePasswordInput,
            crate::domain::password::UpdatePasswordPolicyInput,

            // ── Session domain ─────────────────────────────────────────
            crate::domain::session::SessionInfo,

            // ── Analytics domain ───────────────────────────────────────
            crate::domain::analytics::LoginEvent,
            crate::domain::analytics::LoginEventType,
            crate::domain::analytics::LoginStats,
            crate::domain::analytics::DailyTrendPoint,

            // ── Security domain ────────────────────────────────────────
            crate::domain::analytics::SecurityAlert,
            crate::domain::analytics::SecurityAlertType,
            crate::domain::analytics::AlertSeverity,

            // ── Webhook domain ─────────────────────────────────────────
            crate::domain::analytics::Webhook,
            crate::domain::analytics::CreateWebhookInput,
            crate::domain::analytics::UpdateWebhookInput,

            // ── Action domain ──────────────────────────────────────────
            crate::domain::action::Action,
            crate::domain::action::ActionTrigger,
            crate::domain::action::CreateActionInput,
            crate::domain::action::UpdateActionInput,
            crate::domain::action::ActionExecution,
            crate::domain::action::ActionStats,
            crate::domain::action::TestActionResponse,
            crate::domain::action::BatchUpsertResponse,
            crate::domain::action::BatchError,
            crate::domain::action::UpsertActionInput,

            // ── Branding domain ────────────────────────────────────────
            crate::domain::branding::BrandingConfig,

            // ── Email domain ───────────────────────────────────────────
            crate::domain::email::EmailProviderConfig,
            crate::domain::email::SmtpConfig,
            crate::domain::email::SesConfig,
            crate::domain::email::TenantEmailSettings,

            // ── Email template domain ──────────────────────────────────
            crate::domain::email_template::EmailTemplateType,
            crate::domain::email_template::EmailTemplateContent,
            crate::domain::email_template::EmailTemplateMetadata,
            crate::domain::email_template::EmailTemplateWithContent,
            crate::domain::email_template::RenderedEmailPreview,

            // ── Enterprise SSO domain ──────────────────────────────────
            crate::domain::enterprise_sso::EnterpriseSsoConnector,
            crate::domain::enterprise_sso::EnterpriseSsoDiscoveryResult,

            // ── Identity provider domain ───────────────────────────────
            crate::domain::identity_provider::IdentityProviderType,
            crate::domain::identity_provider::IdentityProvider,
            crate::domain::identity_provider::IdentityProviderTemplate,

            // ── Linked identity domain ─────────────────────────────────
            crate::domain::linked_identity::LinkedIdentityInfo,

            // ── System settings domain ─────────────────────────────────
            crate::domain::system_settings::SystemSettingResponse,
            crate::domain::system_settings::SettingCategory,

            // ── WebAuthn domain ────────────────────────────────────────
            crate::domain::webauthn::WebAuthnCredential,

            // ── Health ─────────────────────────────────────────────────
            crate::domains::security_observability::api::health::HealthResponse,
        ),
    ),
    paths(
        // ── System ─────────────────────────────────────────────────
        crate::domains::security_observability::api::health::health,
        crate::domains::security_observability::api::health::ready,

        // ── Identity: Auth ─────────────────────────────────────────
        crate::domains::identity::api::auth::openid_configuration,
        crate::domains::identity::api::auth::jwks,
        crate::domains::identity::api::auth::authorize,
        crate::domains::identity::api::auth::callback,
        crate::domains::identity::api::auth::enterprise_sso_discovery,
        crate::domains::identity::api::auth::token,
        crate::domains::identity::api::auth::tenant_token,
        crate::domains::identity::api::auth::logout_redirect,
        crate::domains::identity::api::auth::logout,
        crate::domains::identity::api::auth::userinfo,

        // ── Identity: Password ─────────────────────────────────────
        crate::domains::identity::api::password::forgot_password,
        crate::domains::identity::api::password::reset_password,
        crate::domains::identity::api::password::change_password,
        crate::domains::identity::api::password::admin_set_password,
        crate::domains::identity::api::password::get_password_policy,
        crate::domains::identity::api::password::update_password_policy,

        // ── Identity: Session ──────────────────────────────────────
        crate::domains::identity::api::session::list_my_sessions,
        crate::domains::identity::api::session::revoke_session,
        crate::domains::identity::api::session::revoke_other_sessions,
        crate::domains::identity::api::session::force_logout_user,

        // ── Identity: WebAuthn ─────────────────────────────────────
        crate::domains::identity::api::webauthn::start_registration,
        crate::domains::identity::api::webauthn::complete_registration,
        crate::domains::identity::api::webauthn::start_authentication,
        crate::domains::identity::api::webauthn::complete_authentication,
        crate::domains::identity::api::webauthn::list_passkeys,
        crate::domains::identity::api::webauthn::delete_passkey,

        // ── Identity: Identity Provider ────────────────────────────
        crate::domains::identity::api::identity_provider::list_providers,
        crate::domains::identity::api::identity_provider::create_provider,
        crate::domains::identity::api::identity_provider::get_templates,
        crate::domains::identity::api::identity_provider::get_provider,
        crate::domains::identity::api::identity_provider::update_provider,
        crate::domains::identity::api::identity_provider::delete_provider,
        crate::domains::identity::api::identity_provider::list_my_linked_identities,
        crate::domains::identity::api::identity_provider::unlink_identity,

        // ── Tenant Access: Tenant ──────────────────────────────────
        crate::domains::tenant_access::api::tenant::list,
        crate::domains::tenant_access::api::tenant::get,
        crate::domains::tenant_access::api::tenant::create,
        crate::domains::tenant_access::api::tenant::update,
        crate::domains::tenant_access::api::tenant::delete,

        // ── Tenant Access: User ────────────────────────────────────
        crate::domains::tenant_access::api::user::list,
        crate::domains::tenant_access::api::user::get,
        crate::domains::tenant_access::api::user::create,
        crate::domains::tenant_access::api::user::get_me,
        crate::domains::tenant_access::api::user::update_me,
        crate::domains::tenant_access::api::user::update,
        crate::domains::tenant_access::api::user::delete,
        crate::domains::tenant_access::api::user::enable_mfa,
        crate::domains::tenant_access::api::user::disable_mfa,
        crate::domains::tenant_access::api::user::get_tenants,
        crate::domains::tenant_access::api::user::add_to_tenant,
        crate::domains::tenant_access::api::user::remove_from_tenant,
        crate::domains::tenant_access::api::user::update_role_in_tenant,
        crate::domains::tenant_access::api::user::list_by_tenant,

        // ── Tenant Access: Invitation ──────────────────────────────
        crate::domains::tenant_access::api::invitation::list,
        crate::domains::tenant_access::api::invitation::create,
        crate::domains::tenant_access::api::invitation::get,
        crate::domains::tenant_access::api::invitation::delete,
        crate::domains::tenant_access::api::invitation::accept,
        crate::domains::tenant_access::api::invitation::revoke,
        crate::domains::tenant_access::api::invitation::resend,

        // ── Tenant Access: Organization ────────────────────────────
        crate::domains::tenant_access::api::organization::create_organization,
        crate::domains::tenant_access::api::organization::get_my_tenants,

        // ── Tenant Access: SSO ─────────────────────────────────────
        crate::domains::tenant_access::api::tenant_sso::list_connectors,
        crate::domains::tenant_access::api::tenant_sso::create_connector,
        crate::domains::tenant_access::api::tenant_sso::update_connector,
        crate::domains::tenant_access::api::tenant_sso::delete_connector,
        crate::domains::tenant_access::api::tenant_sso::test_connector,

        // ── Authorization: Service ─────────────────────────────────
        crate::domains::authorization::api::service::list,
        crate::domains::authorization::api::service::get,
        crate::domains::authorization::api::service::create,
        crate::domains::authorization::api::service::update,
        crate::domains::authorization::api::service::delete,
        crate::domains::authorization::api::service::integration_info,
        crate::domains::authorization::api::service::list_clients,
        crate::domains::authorization::api::service::create_client,
        crate::domains::authorization::api::service::delete_client,
        crate::domains::authorization::api::service::regenerate_client_secret,

        // ── Authorization: Role & Permission ───────────────────────
        crate::domains::authorization::api::role::create_permission,
        crate::domains::authorization::api::role::delete_permission,
        crate::domains::authorization::api::role::list_permissions,
        crate::domains::authorization::api::role::create_role,
        crate::domains::authorization::api::role::get_role,
        crate::domains::authorization::api::role::update_role,
        crate::domains::authorization::api::role::delete_role,
        crate::domains::authorization::api::role::list_roles,
        crate::domains::authorization::api::role::assign_permission,
        crate::domains::authorization::api::role::remove_permission,
        crate::domains::authorization::api::role::assign_roles,
        crate::domains::authorization::api::role::get_user_roles,
        crate::domains::authorization::api::role::get_user_assigned_roles,
        crate::domains::authorization::api::role::unassign_role,

        // ── Authorization: Tenant-Service ──────────────────────────
        crate::domains::authorization::api::tenant_service::list_services,
        crate::domains::authorization::api::tenant_service::toggle_service,
        crate::domains::authorization::api::tenant_service::get_enabled_services,
        crate::domains::authorization::api::abac::list_policies,
        crate::domains::authorization::api::abac::create_policy,
        crate::domains::authorization::api::abac::update_policy,
        crate::domains::authorization::api::abac::publish_policy,
        crate::domains::authorization::api::abac::rollback_policy,
        crate::domains::authorization::api::abac::simulate_policy,

        // ── Platform: System Settings ──────────────────────────────
        crate::domains::platform::api::system_settings::get_email_settings,
        crate::domains::platform::api::system_settings::update_email_settings,
        crate::domains::platform::api::system_settings::test_email_connection,
        crate::domains::platform::api::system_settings::send_test_email,

        // ── Platform: Branding ─────────────────────────────────────
        crate::domains::platform::api::branding::get_public_branding,
        crate::domains::platform::api::branding::get_branding,
        crate::domains::platform::api::branding::update_branding,

        // ── Platform: Email Templates ──────────────────────────────
        crate::domains::platform::api::email_template::list_templates,
        crate::domains::platform::api::email_template::get_template,
        crate::domains::platform::api::email_template::update_template,
        crate::domains::platform::api::email_template::reset_template,
        crate::domains::platform::api::email_template::preview_template,
        crate::domains::platform::api::email_template::send_test_email,

        // ── Integration: Webhook ───────────────────────────────────
        crate::domains::integration::api::webhook::list_webhooks,
        crate::domains::integration::api::webhook::create_webhook,
        crate::domains::integration::api::webhook::get_webhook,
        crate::domains::integration::api::webhook::update_webhook,
        crate::domains::integration::api::webhook::delete_webhook,
        crate::domains::integration::api::webhook::test_webhook,
        crate::domains::integration::api::webhook::regenerate_webhook_secret,

        // ── Integration: Action ────────────────────────────────────
        crate::domains::integration::api::action::list_actions,
        crate::domains::integration::api::action::create_action,
        crate::domains::integration::api::action::get_action,
        crate::domains::integration::api::action::update_action,
        crate::domains::integration::api::action::delete_action,
        crate::domains::integration::api::action::batch_upsert_actions,
        crate::domains::integration::api::action::test_action,
        crate::domains::integration::api::action::get_action_stats,
        crate::domains::integration::api::action::query_action_logs,
        crate::domains::integration::api::action::get_action_log,
        crate::domains::integration::api::action::get_triggers,

        // ── Integration: Keycloak Event ────────────────────────────
        crate::domains::integration::api::keycloak_event::receive,

        // ── Security & Observability: Audit ────────────────────────
        crate::domains::security_observability::api::audit::list,

        // ── Security & Observability: Analytics ────────────────────
        crate::domains::security_observability::api::analytics::get_stats,
        crate::domains::security_observability::api::analytics::list_events,
        crate::domains::security_observability::api::analytics::get_daily_trend,

        // ── Security & Observability: Security Alerts ──────────────
        crate::domains::security_observability::api::security_alert::list_alerts,
        crate::domains::security_observability::api::security_alert::resolve_alert,
    ),
)]
pub struct ApiDoc;

/// Security scheme definition added via modify
impl ApiDoc {
    pub fn build() -> utoipa::openapi::OpenApi {
        let mut doc = Self::openapi();
        // Add Bearer JWT security scheme
        if let Some(c) = doc.components.as_mut() {
            c.security_schemes.insert(
                "bearer_jwt".to_string(),
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
        doc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_spec_is_valid() {
        let doc = ApiDoc::build();
        let json = serde_json::to_string_pretty(&doc).expect("should serialize to JSON");
        // Verify it's valid JSON
        let _parsed: serde_json::Value = serde_json::from_str(&json).expect("should be valid JSON");
        // Verify basic OpenAPI structure
        assert!(json.contains("\"openapi\""));
        assert!(json.contains("\"paths\""));
        assert!(json.contains("\"components\""));
    }

    #[test]
    fn test_openapi_spec_has_paths() {
        let doc = ApiDoc::build();
        // We registered ~135 handlers; there should be a good number of paths
        assert!(
            doc.paths.paths.len() > 50,
            "Expected >50 paths, got {}",
            doc.paths.paths.len()
        );
    }

    #[test]
    fn test_openapi_spec_has_schemas() {
        let doc = ApiDoc::build();
        let schemas = doc
            .components
            .as_ref()
            .map(|c| c.schemas.len())
            .unwrap_or(0);
        assert!(schemas > 30, "Expected >30 schemas, got {}", schemas);
    }

    #[test]
    fn test_openapi_spec_has_security_scheme() {
        let doc = ApiDoc::build();
        let has_bearer = doc
            .components
            .as_ref()
            .map(|c| c.security_schemes.contains_key("bearer_jwt"))
            .unwrap_or(false);
        assert!(has_bearer, "Missing bearer_jwt security scheme");
    }
}
