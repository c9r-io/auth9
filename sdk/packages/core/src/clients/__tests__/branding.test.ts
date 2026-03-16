import { describe, it, expect, vi, beforeEach } from "vitest";
import { Auth9Client } from "../../auth9-client.js";
import type { BrandingConfig } from "../../types/branding.js";

const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("BrandingClient", () => {
  const client = new Auth9Client({
    baseUrl: "https://auth9.example.com",
    apiKey: "test-token", // pragma: allowlist secret
  });

  const mockBranding: BrandingConfig = {
    primaryColor: "#1a73e8",
    secondaryColor: "#ffffff",
    backgroundColor: "#f5f5f5",
    textColor: "#333333",
    companyName: "Test Corp",
    allowRegistration: true,
    emailOtpEnabled: false,
  };

  describe("get", () => {
    it("sends GET /api/v1/system/branding", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockBranding }),
      });

      const result = await client.branding.get();

      expect(result).toEqual(mockBranding);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/branding",
        expect.objectContaining({ method: "GET" }),
      );
    });
  });

  describe("update", () => {
    it("sends PUT /api/v1/system/branding", async () => {
      const updated = { ...mockBranding, companyName: "Updated Corp" };

      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: updated }),
      });

      const result = await client.branding.update({ config: updated });

      expect(result.companyName).toBe("Updated Corp");
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/branding",
        expect.objectContaining({ method: "PUT" }),
      );
    });
  });

  describe("getPublic", () => {
    it("sends GET /api/v1/public/branding", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockBranding }),
      });

      const result = await client.branding.getPublic();

      expect(result).toEqual(mockBranding);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/public/branding",
        expect.objectContaining({ method: "GET" }),
      );
    });

    it("passes client_id param", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockBranding }),
      });

      await client.branding.getPublic("client-123");

      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining("client_id=client-123"),
        expect.any(Object),
      );
    });
  });

  describe("getForService", () => {
    it("sends GET /api/v1/services/{id}/branding", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockBranding }),
      });

      const result = await client.branding.getForService("svc-1");

      expect(result).toEqual(mockBranding);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/services/svc-1/branding",
        expect.objectContaining({ method: "GET" }),
      );
    });
  });

  describe("updateForService", () => {
    it("sends PUT /api/v1/services/{id}/branding", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockBranding }),
      });

      const result = await client.branding.updateForService("svc-1", {
        config: mockBranding,
      });

      expect(result).toEqual(mockBranding);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/services/svc-1/branding",
        expect.objectContaining({ method: "PUT" }),
      );
    });
  });

  describe("deleteForService", () => {
    it("sends DELETE /api/v1/services/{id}/branding", async () => {
      mockFetch.mockResolvedValue({ ok: true, status: 200, json: () => Promise.resolve({ message: "Deleted" }) });

      await client.branding.deleteForService("svc-1");

      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/services/svc-1/branding",
        expect.objectContaining({ method: "DELETE" }),
      );
    });
  });
});
