import { API_BASE_URL, getHeaders, handleResponse, type ApiError } from "./client";

export interface Webhook {
  id: string;
  tenant_id: string;
  name: string;
  url: string;
  secret?: string;
  events: string[];
  enabled: boolean;
  last_triggered_at?: string;
  failure_count: number;
  created_at: string;
}

export interface CreateWebhookInput {
  name: string;
  url: string;
  secret?: string;
  events: string[];
  enabled?: boolean;
}

export interface WebhookTestResult {
  success: boolean;
  status_code?: number;
  response_time_ms?: number;
  error?: string;
}

export const webhookApi = {
  list: async (
    tenantId: string,
    accessToken?: string
  ): Promise<{ data: Webhook[] }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/webhooks`,
      {
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },

  get: async (
    tenantId: string,
    id: string,
    accessToken?: string
  ): Promise<{ data: Webhook }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/webhooks/${id}`,
      {
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },

  create: async (
    tenantId: string,
    input: CreateWebhookInput,
    accessToken?: string
  ): Promise<{ data: Webhook }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/webhooks`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
        body: JSON.stringify(input),
      }
    );
    return handleResponse(response);
  },

  update: async (
    tenantId: string,
    id: string,
    input: Partial<CreateWebhookInput>,
    accessToken?: string
  ): Promise<{ data: Webhook }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/webhooks/${id}`,
      {
        method: "PUT",
        headers: getHeaders(accessToken),
        body: JSON.stringify(input),
      }
    );
    return handleResponse(response);
  },

  delete: async (
    tenantId: string,
    id: string,
    accessToken?: string
  ): Promise<void> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/webhooks/${id}`,
      {
        method: "DELETE",
        headers: getHeaders(accessToken),
      }
    );
    if (!response.ok) {
      const error: ApiError = await response.json();
      throw new Error(error.message);
    }
  },

  test: async (
    tenantId: string,
    id: string,
    accessToken?: string
  ): Promise<{ data: WebhookTestResult }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/webhooks/${id}/test`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },

  regenerateSecret: async (
    tenantId: string,
    id: string,
    accessToken?: string
  ): Promise<{ data: Webhook }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/webhooks/${id}/regenerate-secret`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },
};
