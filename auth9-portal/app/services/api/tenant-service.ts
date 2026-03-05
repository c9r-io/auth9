import { API_BASE_URL, getHeaders, handleResponse } from "./client";

export interface ServiceWithStatus {
  id: string;
  name: string;
  base_url?: string;
  status: string;
  enabled: boolean;
}

export const tenantServiceApi = {
  listServices: async (
    tenantId: string,
    accessToken?: string
  ): Promise<{ data: ServiceWithStatus[] }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/services`,
      {
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },

  toggleService: async (
    tenantId: string,
    serviceId: string,
    enabled: boolean,
    accessToken?: string
  ): Promise<{ data: ServiceWithStatus[] }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/services`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
        body: JSON.stringify({ service_id: serviceId, enabled }),
      }
    );
    return handleResponse(response);
  },

  getEnabledServices: async (
    tenantId: string,
    accessToken?: string
  ): Promise<{ data: ServiceWithStatus[] }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/services/enabled`,
      {
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },
};
