import type { Auth9HttpClient } from "../http-client.js";
import type {
  Passkey,
  PasskeyRegistrationOptions,
  PasskeyRegistrationResult,
  PasskeyAuthStartInput,
  PasskeyAuthenticationOptions,
  PasskeyAuthenticationResult,
} from "../types/passkey.js";
import type { AuthTokenResponse } from "../types/auth.js";

export class PasskeysClient {
  constructor(private http: Auth9HttpClient) {}

  async list(): Promise<Passkey[]> {
    const result = await this.http.get<{ data: Passkey[] }>(
      "/api/v1/users/me/passkeys",
    );
    return result.data;
  }

  async delete(id: string): Promise<void> {
    await this.http.delete(`/api/v1/users/me/passkeys/${id}`);
  }

  async startRegistration(): Promise<PasskeyRegistrationOptions> {
    const result = await this.http.post<{ data: PasskeyRegistrationOptions }>(
      "/api/v1/users/me/passkeys/register/start",
    );
    return result.data;
  }

  async completeRegistration(
    input: PasskeyRegistrationResult,
  ): Promise<Passkey> {
    const result = await this.http.post<{ data: Passkey }>(
      "/api/v1/users/me/passkeys/register/complete",
      input,
    );
    return result.data;
  }

  async startAuthentication(
    input?: PasskeyAuthStartInput,
  ): Promise<PasskeyAuthenticationOptions> {
    const result = await this.http.post<{
      data: PasskeyAuthenticationOptions;
    }>("/api/v1/auth/webauthn/authenticate/start", input);
    return result.data;
  }

  async completeAuthentication(
    input: PasskeyAuthenticationResult,
  ): Promise<AuthTokenResponse> {
    const result = await this.http.post<{ data: AuthTokenResponse }>(
      "/api/v1/auth/webauthn/authenticate/complete",
      input,
    );
    return result.data;
  }
}
