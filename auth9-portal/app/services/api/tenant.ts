import {
  API_BASE_URL,
  ApiResponseError,
  getHeaders,
  handleResponse,
  type ApiError,
  type PaginatedResponse,
} from "./client";
import type { User } from "./user";

export interface Tenant {
  id: string;
  name: string;
  slug: string;
  domain?: string;
  logo_url?: string;
  settings: Record<string, unknown>;
  status: "active" | "inactive" | "suspended" | "pending";
  created_at: string;
  updated_at: string;
}

export interface CreateTenantInput {
  name: string;
  slug: string;
  logo_url?: string;
  settings?: Record<string, unknown>;
}

export const tenantApi = {
  list: async (
    page = 1,
    perPage = 20,
    search?: string,
    accessToken?: string
  ): Promise<PaginatedResponse<Tenant>> => {
    let url = `${API_BASE_URL}/api/v1/tenants?page=${page}&per_page=${perPage}`;
    if (search) url += `&search=${encodeURIComponent(search)}`;
    const response = await fetch(url, { headers: getHeaders(accessToken) });
    return handleResponse(response);
  },

  get: async (
    id: string,
    accessToken?: string
  ): Promise<{ data: Tenant }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/tenants/${id}`, {
      headers: getHeaders(accessToken),
    });
    return handleResponse(response);
  },

  create: async (
    input: CreateTenantInput,
    accessToken?: string
  ): Promise<{ data: Tenant }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/tenants`, {
      method: "POST",
      headers: getHeaders(accessToken),
      body: JSON.stringify(input),
    });
    return handleResponse(response);
  },

  update: async (
    id: string,
    input: Partial<CreateTenantInput> & { status?: Tenant["status"] },
    accessToken?: string
  ): Promise<{ data: Tenant }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/tenants/${id}`, {
      method: "PUT",
      headers: getHeaders(accessToken),
      body: JSON.stringify(input),
    });
    return handleResponse(response);
  },

  delete: async (id: string, accessToken?: string): Promise<void> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/tenants/${id}`, {
      method: "DELETE",
      headers: {
        ...getHeaders(accessToken),
        "X-Confirm-Destructive": "true",
      },
    });
    if (!response.ok) {
      const error: ApiError = await response.json();
      throw new ApiResponseError(error, response.status);
    }
  },
};

// Tenant User API
export const tenantUserApi = {
  list: async (
    tenantId: string,
    accessToken?: string
  ): Promise<{ data: User[] }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/users`,
      {
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },
};
