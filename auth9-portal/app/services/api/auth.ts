import { API_BASE_URL, getHeaders, handleResponse } from "./client";

export interface TenantTokenExchangeResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  refresh_token?: string;
}

export const authApi = {
  exchangeTenantToken: async (
    tenantId: string,
    serviceId: string,
    identityToken: string
  ): Promise<TenantTokenExchangeResponse> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/auth/tenant-token`, {
      method: "POST",
      headers: getHeaders(identityToken),
      body: JSON.stringify({ tenant_id: tenantId, service_id: serviceId }),
    });
    return handleResponse(response);
  },
};
