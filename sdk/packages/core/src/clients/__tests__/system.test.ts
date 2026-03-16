import { describe, it, expect, vi, beforeEach } from "vitest";
import { Auth9Client } from "../../auth9-client.js";
import type { EmailSettings, TestEmailResponse } from "../../types/system.js";
import type { MaliciousIpBlacklistEntry } from "../../types/tenant.js";

const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("SystemClient", () => {
  const client = new Auth9Client({
    baseUrl: "https://auth9.example.com",
    apiKey: "test-token", // pragma: allowlist secret
  });

  const mockEmailSettings: EmailSettings = {
    config: {
      type: "smtp",
      host: "smtp.example.com",
      port: 587,
      username: "user",
      password: "****",
      useTls: true,
      fromEmail: "noreply@example.com",
    },
  };

  describe("getEmailSettings", () => {
    it("sends GET /api/v1/system/email", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockEmailSettings }),
      });

      const result = await client.system.getEmailSettings();

      expect(result).toEqual(mockEmailSettings);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/email",
        expect.objectContaining({ method: "GET" }),
      );
    });
  });

  describe("updateEmailSettings", () => {
    it("sends PUT /api/v1/system/email", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockEmailSettings }),
      });

      const result = await client.system.updateEmailSettings({
        config: mockEmailSettings.config,
      });

      expect(result).toEqual(mockEmailSettings);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/email",
        expect.objectContaining({ method: "PUT" }),
      );
    });
  });

  describe("testEmailConnection", () => {
    it("sends POST /api/v1/system/email/test", async () => {
      const mockResponse: TestEmailResponse = {
        success: true,
        message: "Connection successful",
      };

      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockResponse),
      });

      const result = await client.system.testEmailConnection();

      expect(result.success).toBe(true);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/email/test",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  describe("sendTestEmail", () => {
    it("sends POST /api/v1/system/email/send-test", async () => {
      const mockResponse: TestEmailResponse = {
        success: true,
        message: "Test email sent",
        messageId: "msg-123",
      };

      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockResponse),
      });

      const result = await client.system.sendTestEmail({
        toEmail: "test@example.com",
      });

      expect(result.success).toBe(true);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/email/send-test",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  describe("getMaliciousIpBlacklist", () => {
    it("sends GET /api/v1/system/security/malicious-ip-blacklist", async () => {
      const mockEntry: MaliciousIpBlacklistEntry = {
        id: "entry-1",
        tenantId: "tenant-1",
        ipAddress: "10.0.0.1",
        reason: "Brute force",
        createdAt: "2026-01-01T00:00:00Z",
        updatedAt: "2026-01-01T00:00:00Z",
      };

      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: [mockEntry] }),
      });

      const result = await client.system.getMaliciousIpBlacklist();

      expect(result).toEqual([mockEntry]);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/security/malicious-ip-blacklist",
        expect.objectContaining({ method: "GET" }),
      );
    });
  });

  describe("updateMaliciousIpBlacklist", () => {
    it("sends PUT /api/v1/system/security/malicious-ip-blacklist", async () => {
      const mockEntry: MaliciousIpBlacklistEntry = {
        id: "entry-1",
        tenantId: "tenant-1",
        ipAddress: "10.0.0.1",
        reason: "Suspicious",
        createdAt: "2026-01-01T00:00:00Z",
        updatedAt: "2026-01-01T00:00:00Z",
      };

      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: [mockEntry] }),
      });

      const result = await client.system.updateMaliciousIpBlacklist({
        entries: [{ ipAddress: "10.0.0.1", reason: "Suspicious" }],
      });

      expect(result).toEqual([mockEntry]);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/security/malicious-ip-blacklist",
        expect.objectContaining({ method: "PUT" }),
      );
    });
  });
});
