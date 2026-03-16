import { describe, it, expect, vi, beforeEach } from "vitest";
import { Auth9Client } from "../../auth9-client.js";
import type { Passkey } from "../../types/passkey.js";
import type { AuthTokenResponse } from "../../types/auth.js";

const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("PasskeysClient", () => {
  const client = new Auth9Client({
    baseUrl: "https://auth9.example.com",
    apiKey: "test-token", // pragma: allowlist secret
  });

  const mockPasskey: Passkey = {
    id: "pk-1",
    name: "My Yubikey",
    credentialId: "cred-abc123",
    createdAt: "2026-01-01T00:00:00Z",
    lastUsedAt: "2026-01-15T10:00:00Z",
  };

  const mockTokenResponse: AuthTokenResponse = {
    accessToken: "eyJ...", // pragma: allowlist secret
    tokenType: "Bearer",
    expiresIn: 3600,
    refreshToken: "refresh-xyz", // pragma: allowlist secret
  };

  describe("list", () => {
    it("sends GET /api/v1/users/me/passkeys", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: [mockPasskey] }),
      });

      const result = await client.passkeys.list();

      expect(result).toEqual([mockPasskey]);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/users/me/passkeys",
        expect.objectContaining({ method: "GET" }),
      );
    });
  });

  describe("delete", () => {
    it("sends DELETE /api/v1/users/me/passkeys/{id}", async () => {
      mockFetch.mockResolvedValue({ ok: true, status: 204 });

      await client.passkeys.delete("pk-1");

      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/users/me/passkeys/pk-1",
        expect.objectContaining({ method: "DELETE" }),
      );
    });
  });

  describe("startRegistration", () => {
    it("sends POST /api/v1/users/me/passkeys/register/start", async () => {
      const mockOptions = { publicKey: { challenge: "abc123" } };
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockOptions }),
      });

      const result = await client.passkeys.startRegistration();

      expect(result).toEqual(mockOptions);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/users/me/passkeys/register/start",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  describe("completeRegistration", () => {
    it("sends POST /api/v1/users/me/passkeys/register/complete", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockPasskey }),
      });

      const result = await client.passkeys.completeRegistration({
        id: "cred-1",
        rawId: "raw-1",
        type: "public-key",
        response: { attestationObject: "abc", clientDataJSON: "def" },
      });

      expect(result).toEqual(mockPasskey);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/users/me/passkeys/register/complete",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  describe("startAuthentication", () => {
    it("sends POST /api/v1/auth/webauthn/authenticate/start", async () => {
      const mockAuthOptions = {
        challengeId: "ch-1",
        publicKey: { challenge: "xyz" },
      };
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockAuthOptions }),
      });

      const result = await client.passkeys.startAuthentication({
        email: "user@example.com",
      });

      expect(result).toEqual(mockAuthOptions);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/auth/webauthn/authenticate/start",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  describe("completeAuthentication", () => {
    it("sends POST /api/v1/auth/webauthn/authenticate/complete", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockTokenResponse }),
      });

      const result = await client.passkeys.completeAuthentication({
        challengeId: "ch-1",
        id: "cred-1",
        rawId: "raw-1",
        type: "public-key",
        response: { authenticatorData: "abc", signature: "def" },
      });

      expect(result).toEqual(mockTokenResponse);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/auth/webauthn/authenticate/complete",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });
});
