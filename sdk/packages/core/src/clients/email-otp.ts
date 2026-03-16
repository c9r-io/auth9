import type { Auth9HttpClient } from "../http-client.js";
import type {
  SendEmailOtpInput,
  VerifyEmailOtpInput,
} from "../types/email-otp.js";
import type { AuthTokenResponse } from "../types/auth.js";

export class EmailOtpClient {
  constructor(private http: Auth9HttpClient) {}

  async send(input: SendEmailOtpInput): Promise<void> {
    await this.http.post("/api/v1/auth/email-otp/send", input);
  }

  async verify(input: VerifyEmailOtpInput): Promise<AuthTokenResponse> {
    const result = await this.http.post<{ data: AuthTokenResponse }>(
      "/api/v1/auth/email-otp/verify",
      input,
    );
    return result.data;
  }
}
