import type { Auth9HttpClient } from "../http-client.js";
import type {
  AuthorizeOptions,
  LogoutOptions,
  TenantTokenInput,
  AuthTokenResponse,
  UserInfo,
  SsoDiscoveryInput,
  SsoDiscoveryResult,
} from "../types/auth.js";

export class AuthClient {
  constructor(
    private http: Auth9HttpClient,
    private baseUrl: string,
  ) {}

  getAuthorizeUrl(options: AuthorizeOptions): string {
    const params = new URLSearchParams();
    params.set("redirect_uri", options.redirectUri);
    params.set("response_type", options.responseType ?? "code");
    params.set("scope", options.scope ?? "openid profile email");
    if (options.state) params.set("state", options.state);
    if (options.tenantId) params.set("tenant_id", options.tenantId);
    return `${this.baseUrl}/api/v1/auth/authorize?${params.toString()}`;
  }

  getLogoutUrl(options?: LogoutOptions): string {
    const params = new URLSearchParams();
    if (options?.postLogoutRedirectUri) {
      params.set("post_logout_redirect_uri", options.postLogoutRedirectUri);
    }
    const query = params.toString();
    return `${this.baseUrl}/api/v1/auth/logout${query ? `?${query}` : ""}`;
  }

  async exchangeTenantToken(
    input: TenantTokenInput,
  ): Promise<AuthTokenResponse> {
    const result = await this.http.post<{ data: AuthTokenResponse }>(
      "/api/v1/auth/tenant-token",
      input,
    );
    return result.data;
  }

  async getUserInfo(): Promise<UserInfo> {
    const result = await this.http.get<{ data: UserInfo }>(
      "/api/v1/auth/userinfo",
    );
    return result.data;
  }

  async discoverEnterpriseSso(
    input: SsoDiscoveryInput,
  ): Promise<SsoDiscoveryResult> {
    const params = new URLSearchParams();
    params.set("client_id", input.clientId);
    params.set("redirect_uri", input.redirectUri);
    params.set("scope", input.scope);
    params.set("state", input.state);
    params.set("response_type", input.responseType ?? "code");
    const result = await this.http.post<{ data: SsoDiscoveryResult }>(
      `/api/v1/enterprise-sso/discovery?${params.toString()}`,
      { email: input.email },
    );
    return result.data;
  }
}
