import {
  API_BASE_URL,
  getHeaders,
  handleResponse,
  type PaginatedResponse,
} from "./client";

export interface LoginStats {
  total_logins: number;
  successful_logins: number;
  failed_logins: number;
  unique_users: number;
  by_event_type: Record<string, number>;
  by_device_type: Record<string, number>;
  period_start: string;
  period_end: string;
}

export interface DailyTrendPoint {
  date: string;
  total: number;
  successful: number;
  failed: number;
}

export interface LoginEvent {
  id: number;
  user_id?: string;
  email?: string;
  tenant_id?: string;
  event_type: string;
  ip_address?: string;
  user_agent?: string;
  device_type?: string;
  location?: string;
  session_id?: string;
  failure_reason?: string;
  created_at: string;
}

export const analyticsApi = {
  getStats: async (
    startDate?: string,
    endDate?: string,
    accessToken?: string
  ): Promise<{ data: LoginStats }> => {
    let url = `${API_BASE_URL}/api/v1/analytics/login-stats`;
    const params = new URLSearchParams();
    if (startDate) params.set("start", startDate);
    if (endDate) params.set("end", endDate);
    if (params.toString()) url += `?${params}`;
    const response = await fetch(url, { headers: getHeaders(accessToken) });
    return handleResponse(response);
  },

  getDailyTrend: async (
    days = 7,
    accessToken?: string,
    startDate?: string,
    endDate?: string
  ): Promise<{ data: DailyTrendPoint[] }> => {
    const params = new URLSearchParams();
    if (startDate && endDate) {
      params.set("start", startDate);
      params.set("end", endDate);
    } else {
      params.set("days", String(days));
    }
    const url = `${API_BASE_URL}/api/v1/analytics/daily-trend?${params}`;
    const response = await fetch(url, { headers: getHeaders(accessToken) });
    return handleResponse(response);
  },

  listEvents: async (
    page = 1,
    perPage = 50,
    email?: string,
    accessToken?: string
  ): Promise<PaginatedResponse<LoginEvent>> => {
    let url = `${API_BASE_URL}/api/v1/analytics/login-events?page=${page}&per_page=${perPage}`;
    if (email) url += `&email=${encodeURIComponent(email)}`;
    const response = await fetch(url, {
      headers: getHeaders(accessToken),
    });
    return handleResponse(response);
  },
};
