import { API_BASE_URL, getHeaders, handleResponse } from "./client";
import type { Tenant } from "./tenant";

export interface CreateOrganizationInput {
  name: string;
  slug: string;
  domain: string;
  logo_url?: string;
}

export const organizationApi = {
  create: async (
    data: CreateOrganizationInput,
    token: string
  ): Promise<{ data: Tenant }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/organizations`, {
      method: "POST",
      headers: getHeaders(token),
      body: JSON.stringify(data),
    });
    return handleResponse(response);
  },
};
