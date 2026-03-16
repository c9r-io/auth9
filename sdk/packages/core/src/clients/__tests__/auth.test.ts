import { describe, it, expect, vi, beforeEach } from "vitest";
import { Auth9Client } from "../../auth9-client.js";
import type { AuthTokenResponse, UserInfo } from "../../types/auth.js";

const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("AuthClient", () => {
  const client = new Auth9Client({
    baseUrl: "https://auth9.example.com",
    apiKey: "test-token", // pragma: allowlist secret
  });

  const mockTokenResponse: AuthTokenResponse = {
    accessToken: "eyJ...", // pragma: allowlist secret
    tokenType: "Bearer",
    expiresIn: 3600,
    refreshToken: "refresh-xyz", // pragma: allowlist secret
  };

  const mockUserInfo: UserInfo = {
    sub: "user-1",
    email: "user@example.com",
    emailVerified: true,
    name: "Test User",
  };

  describe("getAuthorizeUrl", () => {
    it("builds authorize URL with required params", () => {
      const url = client.auth.getAuthorizeUrl({
        redirectUri: "https://app.example.com/callback",
      });

      expect(url).toContain(
        "https://auth9.example.com/api/v1/auth/authorize?",
      );
      expect(url).toContain(
        "redirect_uri=https%3A%2F%2Fapp.example.com%2Fcallback",
      );
      expect(url).toContain("response_type=code");
      expect(url).toContain("scope=openid+profile+email");
    });

    it("includes optional params", () => {
      const url = client.auth.getAuthorizeUrl({
        redirectUri: "https://app.example.com/callback",
        state: "random-state",
        tenantId: "tenant-1",
        scope: "openid",
        responseType: "token",
      });

      expect(url).toContain("state=random-state");
      expect(url).toContain("tenant_id=tenant-1");
      expect(url).toContain("scope=openid");
      expect(url).toContain("response_type=token");
    });
  });

  describe("getLogoutUrl", () => {
    it("builds logout URL without params", () => {
      const url = client.auth.getLogoutUrl();

      expect(url).toBe("https://auth9.example.com/api/v1/auth/logout");
    });

    it("builds logout URL with redirect", () => {
      const url = client.auth.getLogoutUrl({
        postLogoutRedirectUri: "https://app.example.com",
      });

      expect(url).toContain(
        "post_logout_redirect_uri=https%3A%2F%2Fapp.example.com",
      );
    });
  });

  describe("exchangeTenantToken", () => {
    it("sends POST /api/v1/auth/tenant-token", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockTokenResponse }),
      });

      const result = await client.auth.exchangeTenantToken({
        tenantId: "tenant-1",
        serviceId: "svc-1",
      });

      expect(result).toEqual(mockTokenResponse);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/auth/tenant-token",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  describe("getUserInfo", () => {
    it("sends GET /api/v1/auth/userinfo", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockUserInfo }),
      });

      const result = await client.auth.getUserInfo();

      expect(result).toEqual(mockUserInfo);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/auth/userinfo",
        expect.objectContaining({ method: "GET" }),
      );
    });
  });

  describe("discoverEnterpriseSso", () => {
    it("sends POST /api/v1/enterprise-sso/discovery", async () => {
      const mockResult = {
        authorizeUrl: "https://idp.example.com/sso",
        tenantId: "tenant-1",
        connectorId: "conn-1",
        provider: "okta",
      };
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockResult }),
      });

      const result = await client.auth.discoverEnterpriseSso({
        email: "user@corp.example.com",
        clientId: "my-app",
        redirectUri: "https://app.example.com/callback",
        scope: "openid profile email",
        state: "random-state",
      });

      expect(result).toEqual(mockResult);
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining(
          "https://auth9.example.com/api/v1/enterprise-sso/discovery?",
        ),
        expect.objectContaining({ method: "POST" }),
      );
    });
  });
});
