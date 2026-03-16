import { describe, it, expect, vi, beforeEach } from "vitest";
import { Auth9Client } from "../../auth9-client.js";
import type { EmailTemplateWithContent, RenderedEmailPreview } from "../../types/email-template.js";

const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

beforeEach(() => {
  vi.clearAllMocks();
});

describe("EmailTemplatesClient", () => {
  const client = new Auth9Client({
    baseUrl: "https://auth9.example.com",
    apiKey: "test-token", // pragma: allowlist secret
  });

  const mockTemplate: EmailTemplateWithContent = {
    metadata: {
      templateType: "invitation",
      name: "User Invitation",
      description: "Sent when inviting users",
      variables: [{ name: "inviteLink", description: "Invitation URL", example: "https://example.com/invite" }],
    },
    content: {
      subject: "You are invited",
      htmlBody: "<h1>Welcome</h1>",
      textBody: "Welcome",
    },
    isCustomized: false,
  };

  describe("list", () => {
    it("sends GET /api/v1/system/email-templates", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: [mockTemplate] }),
      });

      const result = await client.emailTemplates.list();

      expect(result).toEqual([mockTemplate]);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/email-templates",
        expect.objectContaining({ method: "GET" }),
      );
    });
  });

  describe("get", () => {
    it("sends GET /api/v1/system/email-templates/{type}", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockTemplate }),
      });

      const result = await client.emailTemplates.get("invitation");

      expect(result).toEqual(mockTemplate);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/email-templates/invitation",
        expect.objectContaining({ method: "GET" }),
      );
    });
  });

  describe("update", () => {
    it("sends PUT /api/v1/system/email-templates/{type}", async () => {
      const updated = { ...mockTemplate, isCustomized: true };

      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: updated }),
      });

      const result = await client.emailTemplates.update("invitation", {
        subject: "You are invited",
        htmlBody: "<h1>Welcome</h1>",
        textBody: "Welcome",
      });

      expect(result.isCustomized).toBe(true);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/email-templates/invitation",
        expect.objectContaining({ method: "PUT" }),
      );
    });
  });

  describe("reset", () => {
    it("sends DELETE /api/v1/system/email-templates/{type}", async () => {
      const reset = { ...mockTemplate, isCustomized: false };

      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: reset }),
      });

      const result = await client.emailTemplates.reset("invitation");

      expect(result.isCustomized).toBe(false);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/email-templates/invitation",
        expect.objectContaining({ method: "DELETE" }),
      );
    });
  });

  describe("preview", () => {
    it("sends POST /api/v1/system/email-templates/{type}/preview", async () => {
      const mockPreview: RenderedEmailPreview = {
        subject: "Rendered subject",
        htmlBody: "<h1>Rendered</h1>",
        textBody: "Rendered",
      };

      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockPreview }),
      });

      const result = await client.emailTemplates.preview("invitation", {
        subject: "Template {{name}}",
        htmlBody: "<h1>Hello {{name}}</h1>",
        textBody: "Hello {{name}}",
      });

      expect(result).toEqual(mockPreview);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/email-templates/invitation/preview",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  describe("sendTest", () => {
    it("sends POST /api/v1/system/email-templates/{type}/send-test", async () => {
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () =>
          Promise.resolve({ success: true, message: "Test email sent" }),
      });

      const result = await client.emailTemplates.sendTest("invitation", {
        toEmail: "test@example.com",
        subject: "Test",
        htmlBody: "<h1>Test</h1>",
        textBody: "Test",
      });

      expect(result.success).toBe(true);
      expect(mockFetch).toHaveBeenCalledWith(
        "https://auth9.example.com/api/v1/system/email-templates/invitation/send-test",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });
});
