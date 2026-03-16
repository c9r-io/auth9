import { describe, it, expect, vi, beforeEach } from "vitest";
import { Auth9Client } from "../../auth9-client.js";
import type { SecurityAlert } from "../../types/analytics.js";

const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("SecurityAlertsClient", () => {
  const client = new Auth9Client({
    baseUrl: "https://auth9.example.com",
    apiKey: "test-token", // pragma: allowlist secret
  });

  const mockAlert: SecurityAlert = {
    id: "alert-1",
    userId: "user-1",
    tenantId: "tenant-1",
    alertType: "bruteForce",
    severity: "high",
    details: { attempts: 10 },
    createdAt: "2026-01-01T00:00:00Z",
  };

  describe("list", () => {
    it("sends GET /api/v1/security/alerts", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () =>
          Promise.resolve({
            data: [mockAlert],
            pagination: { page: 1, per_page: 50, total: 1, total_pages: 1 },
          }),
      });

      const result = await client.securityAlerts.list();

      expect(result.data).toEqual([mockAlert]);
      expect(result.pagination).toBeDefined();
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/security/alerts",
        expect.objectContaining({ method: "GET" }),
      );
    });

    it("passes filter params", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () =>
          Promise.resolve({
            data: [],
            pagination: { page: 1, per_page: 50, total: 0, total_pages: 0 },
          }),
      });

      await client.securityAlerts.list({
        severity: "critical",
        alertType: "brute_force",
        unresolvedOnly: true,
      });

      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining("severity=critical"),
        expect.any(Object),
      );
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining("alert_type=brute_force"),
        expect.any(Object),
      );
    });
  });

  describe("resolve", () => {
    it("sends POST /api/v1/security/alerts/{id}/resolve", async () => {
      const resolved = { ...mockAlert, resolvedAt: "2026-01-02T00:00:00Z", resolvedBy: "admin-1" };

      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: resolved }),
      });

      const result = await client.securityAlerts.resolve("alert-1");

      expect(result.resolvedAt).toBeDefined();
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/security/alerts/alert-1/resolve",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });
});
