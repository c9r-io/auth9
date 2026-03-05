import {
  API_BASE_URL,
  getHeaders,
  handleResponse,
  type PaginatedResponse,
} from "./client";

export type AlertSeverity = "low" | "medium" | "high" | "critical";
export type SecurityAlertType =
  | "brute_force"
  | "new_device"
  | "impossible_travel"
  | "suspicious_ip";

export interface SecurityAlert {
  id: string;
  user_id?: string;
  tenant_id?: string;
  alert_type: SecurityAlertType;
  severity: AlertSeverity;
  details?: Record<string, unknown>;
  resolved_at?: string;
  resolved_by?: string;
  created_at: string;
}

export const securityAlertApi = {
  list: async (
    page = 1,
    perPage = 50,
    unresolvedOnly = false,
    accessToken?: string,
    severity?: AlertSeverity,
    alertType?: SecurityAlertType
  ): Promise<PaginatedResponse<SecurityAlert>> => {
    let url = `${API_BASE_URL}/api/v1/security/alerts?page=${page}&per_page=${perPage}`;
    if (unresolvedOnly) url += "&unresolved_only=true";
    if (severity) url += `&severity=${severity}`;
    if (alertType) url += `&alert_type=${alertType}`;
    const response = await fetch(url, {
      headers: getHeaders(accessToken),
    });
    return handleResponse(response);
  },

  resolve: async (
    id: string,
    accessToken?: string
  ): Promise<{ data: SecurityAlert }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/security/alerts/${id}/resolve`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },
};
