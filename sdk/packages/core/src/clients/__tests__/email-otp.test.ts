import { describe, it, expect, vi, beforeEach } from "vitest";
import { Auth9Client } from "../../auth9-client.js";
import type { AuthTokenResponse } from "../../types/auth.js";

const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("EmailOtpClient", () => {
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

  describe("send", () => {
    it("sends POST /api/v1/auth/email-otp/send", async () => {
      mockFetch.mockResolvedValue({ ok: true, status: 204 });

      await client.emailOtp.send({ email: "user@example.com" });

      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/auth/email-otp/send",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  describe("verify", () => {
    it("sends POST /api/v1/auth/email-otp/verify", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockTokenResponse }),
      });

      const result = await client.emailOtp.verify({
        email: "user@example.com",
        code: "123456",
      });

      expect(result).toEqual(mockTokenResponse);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/auth/email-otp/verify",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });
});
