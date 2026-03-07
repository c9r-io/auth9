import { API_BASE_URL, ApiResponseError, getHeaders, handleResponse, type ApiError } from "./client";

export type ActionTrigger =
  | "post-login"
  | "pre-user-registration"
  | "post-user-registration"
  | "post-change-password"
  | "post-email-verification"
  | "pre-token-refresh";

export interface Action {
  id: string;
  service_id: string;
  name: string;
  description?: string;
  trigger_id: ActionTrigger;
  script: string;
  enabled: boolean;
  execution_order: number;
  timeout_ms: number;
  last_executed_at?: string;
  execution_count: number;
  error_count: number;
  last_error?: string;
  created_at: string;
  updated_at: string;
}

export interface CreateActionInput {
  name: string;
  description?: string;
  trigger_id: ActionTrigger;
  script: string;
  enabled?: boolean;
  execution_order?: number;
  timeout_ms?: number;
}

export interface UpdateActionInput {
  name?: string;
  description?: string;
  script?: string;
  enabled?: boolean;
  execution_order?: number;
  timeout_ms?: number;
}

export interface ActionExecution {
  id: string;
  action_id: string;
  service_id: string;
  trigger_id: ActionTrigger;
  user_id?: string;
  success: boolean;
  duration_ms: number;
  error_message?: string;
  executed_at: string;
}

export interface ActionStats {
  execution_count: number;
  error_count: number;
  success_rate: number;
  avg_duration_ms: number;
  last_24h_count: number;
  last_executed_at?: string;
}

export const actionApi = {
  list: async (
    serviceId: string,
    trigger?: ActionTrigger,
    accessToken?: string
  ): Promise<{ data: Action[] }> => {
    let url = `${API_BASE_URL}/api/v1/services/${serviceId}/actions`;
    if (trigger) url += `?trigger_id=${trigger}`;
    const response = await fetch(url, {
      headers: getHeaders(accessToken),
    });
    return handleResponse(response);
  },

  get: async (
    serviceId: string,
    actionId: string,
    accessToken?: string
  ): Promise<{ data: Action }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/services/${serviceId}/actions/${actionId}`,
      {
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },

  create: async (
    serviceId: string,
    input: CreateActionInput,
    accessToken?: string
  ): Promise<{ data: Action }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/services/${serviceId}/actions`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
        body: JSON.stringify(input),
      }
    );
    return handleResponse(response);
  },

  update: async (
    serviceId: string,
    actionId: string,
    input: UpdateActionInput,
    accessToken?: string
  ): Promise<{ data: Action }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/services/${serviceId}/actions/${actionId}`,
      {
        method: "PATCH",
        headers: getHeaders(accessToken),
        body: JSON.stringify(input),
      }
    );
    return handleResponse(response);
  },

  delete: async (
    serviceId: string,
    actionId: string,
    accessToken?: string
  ): Promise<void> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/services/${serviceId}/actions/${actionId}`,
      {
        method: "DELETE",
        headers: getHeaders(accessToken),
      }
    );
    if (!response.ok) {
      const error: ApiError = await response.json();
      throw new ApiResponseError(error, response.status);
    }
  },

  logs: async (
    serviceId: string,
    actionId?: string,
    success?: boolean,
    limit = 50,
    accessToken?: string
  ): Promise<{
    data: ActionExecution[];
    pagination: {
      page: number;
      per_page: number;
      total: number;
      total_pages: number;
    };
  }> => {
    let url = `${API_BASE_URL}/api/v1/services/${serviceId}/actions/logs?limit=${limit}`;
    if (actionId) url += `&action_id=${actionId}`;
    if (success !== undefined) url += `&success=${success}`;
    const response = await fetch(url, {
      headers: getHeaders(accessToken),
    });
    return handleResponse(response);
  },

  stats: async (
    serviceId: string,
    actionId: string,
    accessToken?: string
  ): Promise<{ data: ActionStats }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/services/${serviceId}/actions/${actionId}/stats`,
      {
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },

  triggers: async (
    accessToken?: string
  ): Promise<{ data: ActionTrigger[] }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/actions/triggers`, {
      headers: getHeaders(accessToken),
    });
    return handleResponse(response);
  },
};
