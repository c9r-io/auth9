import { describe, it, expect, vi, beforeEach } from "vitest";
import { Auth9Client } from "../../auth9-client.js";
import type { AuditLog } from "../../types/analytics.js";

const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("AuditLogsClient", () => {
  const client = new Auth9Client({
    baseUrl: "https://auth9.example.com",
    apiKey: "test-token", // pragma: allowlist secret
  });

  const mockAuditLog: AuditLog = {
    id: 1,
    actorId: "user-1",
    actorEmail: "admin@example.com",
    actorDisplayName: "Admin User",
    action: "tenant.create",
    resourceType: "tenant",
    resourceId: "tenant-1",
    ipAddress: "192.168.1.1",
    createdAt: "2026-01-01T00:00:00Z",
  };

  describe("list", () => {
    it("sends GET /api/v1/audit-logs", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () =>
          Promise.resolve({
            data: [mockAuditLog],
            pagination: { page: 1, per_page: 50, total: 1, total_pages: 1 },
          }),
      });

      const result = await client.auditLogs.list();

      expect(result.data).toEqual([mockAuditLog]);
      expect(result.pagination).toBeDefined();
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/audit-logs",
        expect.objectContaining({ method: "GET" }),
      );
    });

    it("passes query params", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () =>
          Promise.resolve({
            data: [],
            pagination: { page: 2, per_page: 10, total: 0, total_pages: 0 },
          }),
      });

      await client.auditLogs.list({
        actorId: "user-1",
        action: "tenant.create",
        page: 2,
        perPage: 10,
      });

      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining("actor_id=user-1"),
        expect.objectContaining({ method: "GET" }),
      );
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining("action=tenant.create"),
        expect.any(Object),
      );
    });
  });
});
