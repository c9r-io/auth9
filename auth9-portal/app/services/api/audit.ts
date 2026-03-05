import {
  API_BASE_URL,
  getHeaders,
  handleResponse,
  type PaginatedResponse,
} from "./client";

export interface AuditLog {
  id: number;
  actor_id?: string;
  actor_email?: string;
  actor_display_name?: string;
  action: string;
  resource_type: string;
  resource_id?: string;
  old_value?: unknown;
  new_value?: unknown;
  ip_address?: string;
  created_at: string;
}

export const auditApi = {
  list: async (
    page = 1,
    perPage = 50,
    accessToken?: string
  ): Promise<PaginatedResponse<AuditLog>> => {
    const offset = (page - 1) * perPage;
    const response = await fetch(
      `${API_BASE_URL}/api/v1/audit-logs?limit=${perPage}&offset=${offset}`,
      { headers: getHeaders(accessToken) }
    );
    return handleResponse(response);
  },
};
