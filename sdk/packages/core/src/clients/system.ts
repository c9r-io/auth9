import type { Auth9HttpClient } from "../http-client.js";
import type {
  EmailSettings,
  UpdateEmailSettingsInput,
  TestEmailResponse,
  SendTestEmailInput,
} from "../types/system.js";
import type {
  MaliciousIpBlacklistEntry,
  UpdateMaliciousIpBlacklistInput,
} from "../types/tenant.js";

export class SystemClient {
  constructor(private http: Auth9HttpClient) {}

  async getEmailSettings(): Promise<EmailSettings> {
    const result = await this.http.get<{ data: EmailSettings }>(
      "/api/v1/system/email",
    );
    return result.data;
  }

  async updateEmailSettings(
    input: UpdateEmailSettingsInput,
  ): Promise<EmailSettings> {
    const result = await this.http.put<{ data: EmailSettings }>(
      "/api/v1/system/email",
      input,
    );
    return result.data;
  }

  async testEmailConnection(): Promise<TestEmailResponse> {
    return this.http.post<TestEmailResponse>("/api/v1/system/email/test");
  }

  async sendTestEmail(input: SendTestEmailInput): Promise<TestEmailResponse> {
    return this.http.post<TestEmailResponse>(
      "/api/v1/system/email/send-test",
      input,
    );
  }

  async getMaliciousIpBlacklist(): Promise<MaliciousIpBlacklistEntry[]> {
    const result = await this.http.get<{
      data: MaliciousIpBlacklistEntry[];
    }>("/api/v1/system/security/malicious-ip-blacklist");
    return result.data;
  }

  async updateMaliciousIpBlacklist(
    input: UpdateMaliciousIpBlacklistInput,
  ): Promise<MaliciousIpBlacklistEntry[]> {
    const result = await this.http.put<{
      data: MaliciousIpBlacklistEntry[];
    }>("/api/v1/system/security/malicious-ip-blacklist", input);
    return result.data;
  }
}
