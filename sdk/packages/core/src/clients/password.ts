import type { Auth9HttpClient } from "../http-client.js";
import type {
  PasswordPolicy,
  ForgotPasswordInput,
  ResetPasswordInput,
  ChangePasswordInput,
  AdminSetPasswordInput,
  UpdatePasswordPolicyInput,
} from "../types/password.js";

export class PasswordClient {
  constructor(private http: Auth9HttpClient) {}

  async forgotPassword(input: ForgotPasswordInput): Promise<void> {
    await this.http.post("/api/v1/auth/forgot-password", input);
  }

  async resetPassword(input: ResetPasswordInput): Promise<void> {
    await this.http.post("/api/v1/auth/reset-password", input);
  }

  async changeMyPassword(input: ChangePasswordInput): Promise<void> {
    await this.http.post("/api/v1/users/me/password", input);
  }

  async adminSetPassword(
    userId: string,
    input: AdminSetPasswordInput,
  ): Promise<void> {
    await this.http.put(`/api/v1/users/${userId}/password`, input);
  }

  async getPolicy(tenantId: string): Promise<PasswordPolicy> {
    const result = await this.http.get<{ data: PasswordPolicy }>(
      `/api/v1/tenants/${tenantId}/password-policy`,
    );
    return result.data;
  }

  async updatePolicy(
    tenantId: string,
    input: UpdatePasswordPolicyInput,
  ): Promise<PasswordPolicy> {
    const result = await this.http.put<{ data: PasswordPolicy }>(
      `/api/v1/tenants/${tenantId}/password-policy`,
      input,
    );
    return result.data;
  }
}
