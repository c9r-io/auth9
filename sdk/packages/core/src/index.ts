// Types - Claims
export type {
  IdentityClaims,
  TenantAccessClaims,
  ServiceClientClaims,
  Auth9Claims,
  TokenType,
} from "./types/claims.js";
export { getTokenType } from "./types/claims.js";

// Types - Responses
export type {
  DataResponse,
  PaginatedResponse,
  Pagination,
} from "./types/responses.js";

// Types - Domain
export type {
  Tenant,
  CreateTenantInput,
  UpdateTenantInput,
  MaliciousIpBlacklistEntry,
  UpdateMaliciousIpBlacklistInput,
  TenantUser,
} from "./types/tenant.js";
export type {
  User,
  CreateUserInput,
  UpdateUserInput,
  AddUserToTenantInput,
  UpdateUserRoleInput,
} from "./types/user.js";
export type {
  Role,
  Permission,
  CreateRoleInput,
  CreatePermissionInput,
  UpdateRoleInput,
  RoleWithPermissions,
  AssignRolesInput,
  UserRolesInTenant,
} from "./types/rbac.js";
export type {
  Service,
  CreateServiceInput,
  UpdateServiceInput,
  ServiceIntegration,
  Client,
  ClientWithSecret,
  CreateClientInput,
  ServiceWithStatus,
} from "./types/service.js";
export type { SessionInfo } from "./types/session.js";
export type {
  Invitation,
  InvitationStatus,
  CreateInvitationInput,
  InvitationValidation,
  AcceptInvitationInput,
} from "./types/invitation.js";
export type {
  Webhook,
  CreateWebhookInput,
  UpdateWebhookInput,
  WebhookTestResult,
} from "./types/webhook.js";
export type {
  IdentityProvider,
  CreateIdentityProviderInput,
  UpdateIdentityProviderInput,
  IdentityProviderTemplate,
  LinkedIdentity,
} from "./types/identity-provider.js";
export type {
  SSOConnector,
  CreateSSOConnectorInput,
  UpdateSSOConnectorInput,
  SSOTestResult,
} from "./types/sso.js";
export type {
  SamlApplication,
  CreateSamlApplicationInput,
  UpdateSamlApplicationInput,
  SamlCertificateInfo,
} from "./types/saml.js";
export type {
  AbacPolicy,
  AbacRule,
  CreateAbacPolicyInput,
  UpdateAbacPolicyInput,
  SimulateAbacInput,
  AbacSimulationResult,
} from "./types/abac.js";
export type {
  ScimToken,
  ScimTokenWithValue,
  CreateScimTokenInput,
  ScimLog,
  ScimLogQuery,
  ScimGroupMapping,
} from "./types/scim.js";
export type {
  TenantServiceInfo,
  ToggleTenantServiceInput,
} from "./types/tenant-service.js";
export type {
  LoginStats,
  LoginEvent,
  AuditLog,
  SecurityAlert,
  AuditLogQuery,
  AuditLogPage,
  LoginStatsQuery,
  LoginEventsQuery,
  DailyTrendQuery,
  DailyTrendPoint,
  SecurityAlertsQuery,
} from "./types/analytics.js";
export type {
  EmailProviderConfig,
  SmtpConfig,
  SesConfig,
  OracleConfig,
  NoneConfig,
  EmailSettings,
  UpdateEmailSettingsInput,
  TestEmailResponse,
  SendTestEmailInput,
} from "./types/system.js";
export type {
  EmailTemplateWithContent,
  EmailTemplateMetadata,
  TemplateVariable,
  EmailTemplateContent,
  UpdateEmailTemplateInput,
  PreviewEmailTemplateInput,
  SendTestEmailTemplateInput,
  RenderedEmailPreview,
  SendTestEmailResponse,
} from "./types/email-template.js";
export type { BrandingConfig, UpdateBrandingInput } from "./types/branding.js";
export type {
  Action,
  CreateActionInput,
  UpdateActionInput,
  ActionContext,
  ActionContextUser,
  ActionContextTenant,
  ActionContextRequest,
  TestActionResponse,
  ActionExecution,
  ActionStats,
  UpsertActionInput,
  BatchUpsertResponse,
  BatchError,
  LogQueryFilter,
} from "./types/action.js";
export { ActionTrigger } from "./types/action.js";
export type {
  PasswordPolicy,
  ForgotPasswordInput,
  ResetPasswordInput,
  ChangePasswordInput,
  AdminSetPasswordInput,
  UpdatePasswordPolicyInput,
} from "./types/password.js";
export type {
  Passkey,
  PasskeyRegistrationOptions,
  PasskeyRegistrationResult,
  PasskeyAuthStartInput,
  PasskeyAuthenticationOptions,
  PasskeyAuthenticationResult,
} from "./types/passkey.js";
export type {
  SendEmailOtpInput,
  VerifyEmailOtpInput,
} from "./types/email-otp.js";
export type {
  AuthorizeOptions,
  LogoutOptions,
  TenantTokenInput,
  AuthTokenResponse,
  UserInfo,
  SsoDiscoveryInput,
  SsoDiscoveryResult,
} from "./types/auth.js";
export type {
  Organization,
  CreateOrganizationInput,
} from "./types/organization.js";

// HTTP Client
export { Auth9HttpClient } from "./http-client.js";
export type { HttpClientConfig } from "./http-client.js";
export { Auth9Client } from "./auth9-client.js";
export type { Auth9ClientConfig } from "./auth9-client.js";

// Sub-Clients
export { TenantsClient } from "./clients/tenants.js";
export { UsersClient } from "./clients/users.js";
export { ServicesClient } from "./clients/services.js";
export { RolesClient } from "./clients/roles.js";
export { PermissionsClient } from "./clients/permissions.js";
export { RbacClient } from "./clients/rbac.js";
export { InvitationsClient } from "./clients/invitations.js";
export { IdentityProvidersClient } from "./clients/identity-providers.js";
export { SsoClient } from "./clients/sso.js";
export { SamlClient } from "./clients/saml.js";
export { AbacClient } from "./clients/abac.js";
export { SessionsClient } from "./clients/sessions.js";
export { WebhooksClient } from "./clients/webhooks.js";
export { ScimClient } from "./clients/scim.js";
export { TenantServicesClient } from "./clients/tenant-services.js";
export { PasswordClient } from "./clients/password.js";
export { PasskeysClient } from "./clients/passkeys.js";
export { EmailOtpClient } from "./clients/email-otp.js";
export { AuthClient } from "./clients/auth.js";
export { OrganizationsClient } from "./clients/organizations.js";
export { AuditLogsClient } from "./clients/audit-logs.js";
export { AnalyticsClient } from "./clients/analytics.js";
export { SecurityAlertsClient } from "./clients/security-alerts.js";
export { SystemClient } from "./clients/system.js";
export { EmailTemplatesClient } from "./clients/email-templates.js";
export { BrandingClient } from "./clients/branding.js";

// Errors
export {
  Auth9Error,
  NotFoundError,
  UnauthorizedError,
  ForbiddenError,
  ValidationError,
  ConflictError,
  RateLimitError,
  BadRequestError,
  createErrorFromStatus,
} from "./errors.js";

// Utils
export { toSnakeCase, toCamelCase } from "./utils.js";
