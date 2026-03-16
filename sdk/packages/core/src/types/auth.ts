export interface AuthorizeOptions {
  redirectUri: string;
  scope?: string;
  state?: string;
  responseType?: string;
  tenantId?: string;
}

export interface LogoutOptions {
  postLogoutRedirectUri?: string;
}

export interface TenantTokenInput {
  tenantId: string;
  serviceId: string;
}

export interface AuthTokenResponse {
  accessToken: string;
  tokenType: string;
  expiresIn: number;
  refreshToken?: string;
}

export interface UserInfo {
  sub: string;
  email?: string;
  emailVerified?: boolean;
  name?: string;
  picture?: string;
  [key: string]: unknown;
}

export interface SsoDiscoveryInput {
  email: string;
  clientId: string;
  redirectUri: string;
  scope: string;
  state: string;
  responseType?: string;
}

export interface SsoDiscoveryResult {
  authorizeUrl: string;
  tenantId?: string;
  connectorId?: string;
  provider?: string;
}
