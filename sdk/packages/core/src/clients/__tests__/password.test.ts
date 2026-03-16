import { describe, it, expect, vi, beforeEach } from "vitest";
import { Auth9Client } from "../../auth9-client.js";
import type { PasswordPolicy } from "../../types/password.js";

const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("PasswordClient", () => {
  const client = new Auth9Client({
    baseUrl: "https://auth9.example.com",
    apiKey: "test-token", // pragma: allowlist secret
  });

  const mockPolicy: PasswordPolicy = {
    minLength: 8,
    requireUppercase: true,
    requireLowercase: true,
    requireNumbers: true,
    requireSymbols: false,
    maxAgeDays: 90,
    historyCount: 5,
    lockoutThreshold: 5,
    lockoutDurationMins: 15,
  };

  describe("forgotPassword", () => {
    it("sends POST /api/v1/auth/forgot-password", async () => {
      mockFetch.mockResolvedValue({ ok: true, status: 204 });

      await client.password.forgotPassword({ email: "user@example.com" });

      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/auth/forgot-password",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  describe("resetPassword", () => {
    it("sends POST /api/v1/auth/reset-password", async () => {
      mockFetch.mockResolvedValue({ ok: true, status: 204 });

      await client.password.resetPassword({
        token: "reset-token-123",
        newPassword: "NewPass123!", // pragma: allowlist secret
      });

      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/auth/reset-password",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  describe("changeMyPassword", () => {
    it("sends POST /api/v1/users/me/password", async () => {
      mockFetch.mockResolvedValue({ ok: true, status: 204 });

      await client.password.changeMyPassword({
        currentPassword: "OldPass123!", // pragma: allowlist secret
        newPassword: "NewPass456!", // pragma: allowlist secret
      });

      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/users/me/password",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  describe("adminSetPassword", () => {
    it("sends PUT /api/v1/users/{id}/password", async () => {
      mockFetch.mockResolvedValue({ ok: true, status: 204 });

      await client.password.adminSetPassword("user-1", {
        password: "TempPass123!", // pragma: allowlist secret
        temporary: true,
      });

      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/users/user-1/password",
        expect.objectContaining({ method: "PUT" }),
      );
    });
  });

  describe("getPolicy", () => {
    it("sends GET /api/v1/tenants/{id}/password-policy", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockPolicy }),
      });

      const result = await client.password.getPolicy("tenant-1");

      expect(result).toEqual(mockPolicy);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/tenants/tenant-1/password-policy",
        expect.objectContaining({ method: "GET" }),
      );
    });
  });

  describe("updatePolicy", () => {
    it("sends PUT /api/v1/tenants/{id}/password-policy", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockPolicy }),
      });

      const result = await client.password.updatePolicy("tenant-1", {
        minLength: 12,
      });

      expect(result).toEqual(mockPolicy);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/tenants/tenant-1/password-policy",
        expect.objectContaining({ method: "PUT" }),
      );
    });
  });
});
