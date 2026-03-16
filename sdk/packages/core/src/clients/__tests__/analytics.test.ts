import { describe, it, expect, vi, beforeEach } from "vitest";
import { Auth9Client } from "../../auth9-client.js";
import type { LoginStats, LoginEvent, DailyTrendPoint } from "../../types/analytics.js";

const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("AnalyticsClient", () => {
  const client = new Auth9Client({
    baseUrl: "https://auth9.example.com",
    apiKey: "test-token", // pragma: allowlist secret
  });

  describe("getLoginStats", () => {
    it("sends GET /api/v1/analytics/login-stats", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () =>
          Promise.resolve({
            data: {
              total_logins: 100,
              successful_logins: 90,
              failed_logins: 10,
              unique_users: 50,
              by_event_type: { success: 90, failed_password: 10 },
              by_device_type: { desktop: 70, mobile: 30 },
              period_start: "2026-01-01T00:00:00Z",
              period_end: "2026-01-31T00:00:00Z",
            },
          }),
      });

      const result = await client.analytics.getLoginStats();

      expect(result.totalLogins).toBe(100);
      expect(result.failedLogins).toBe(10);
      expect(result.uniqueUsers).toBe(50);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/analytics/login-stats",
        expect.objectContaining({ method: "GET" }),
      );
    });

    it("passes query params", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: {} }),
      });

      await client.analytics.getLoginStats({ tenantId: "t-1", days: 7 });

      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining("tenant_id=t-1"),
        expect.any(Object),
      );
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining("days=7"),
        expect.any(Object),
      );
    });
  });

  describe("listLoginEvents", () => {
    it("sends GET /api/v1/analytics/login-events", async () => {
      const mockEvent: LoginEvent = {
        id: 1,
        email: "user@example.com",
        eventType: "success",
        createdAt: "2026-01-01T00:00:00Z",
      };

      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () =>
          Promise.resolve({
            data: [mockEvent],
            pagination: { page: 1, per_page: 50, total: 1, total_pages: 1 },
          }),
      });

      const result = await client.analytics.listLoginEvents();

      expect(result.data).toEqual([mockEvent]);
      expect(result.pagination).toBeDefined();
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/analytics/login-events",
        expect.objectContaining({ method: "GET" }),
      );
    });
  });

  describe("getDailyTrend", () => {
    it("sends GET /api/v1/analytics/daily-trend", async () => {
      const mockPoints: DailyTrendPoint[] = [
        { date: "2026-01-01", total: 100, successful: 90, failed: 10 },
        { date: "2026-01-02", total: 120, successful: 110, failed: 10 },
      ];

      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockPoints }),
      });

      const result = await client.analytics.getDailyTrend();

      expect(result).toEqual(mockPoints);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/analytics/daily-trend",
        expect.objectContaining({ method: "GET" }),
      );
    });
  });
});
