import type { Auth9HttpClient } from "../http-client.js";
import type { BrandingConfig, UpdateBrandingInput } from "../types/branding.js";

export class BrandingClient {
  constructor(private http: Auth9HttpClient) {}

  async get(): Promise<BrandingConfig> {
    const result = await this.http.get<{ data: BrandingConfig }>(
      "/api/v1/system/branding",
    );
    return result.data;
  }

  async update(input: UpdateBrandingInput): Promise<BrandingConfig> {
    const result = await this.http.put<{ data: BrandingConfig }>(
      "/api/v1/system/branding",
      input,
    );
    return result.data;
  }

  async getPublic(clientId?: string): Promise<BrandingConfig> {
    const params: Record<string, string> = {};
    if (clientId) params.client_id = clientId;
    const hasParams = Object.keys(params).length > 0;
    const result = await this.http.get<{ data: BrandingConfig }>(
      "/api/v1/public/branding",
      hasParams ? params : undefined,
    );
    return result.data;
  }

  async getForService(serviceId: string): Promise<BrandingConfig> {
    const result = await this.http.get<{ data: BrandingConfig }>(
      `/api/v1/services/${serviceId}/branding`,
    );
    return result.data;
  }

  async updateForService(
    serviceId: string,
    input: UpdateBrandingInput,
  ): Promise<BrandingConfig> {
    const result = await this.http.put<{ data: BrandingConfig }>(
      `/api/v1/services/${serviceId}/branding`,
      input,
    );
    return result.data;
  }

  async deleteForService(serviceId: string): Promise<void> {
    await this.http.delete(`/api/v1/services/${serviceId}/branding`);
  }
}
