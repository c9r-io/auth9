import { API_BASE_URL, getHeaders, handleResponse } from "./client";

export interface SessionInfo {
  id: string;
  device_type?: string;
  device_name?: string;
  ip_address?: string;
  location?: string;
  last_active_at: string;
  created_at: string;
  is_current: boolean;
}

export const sessionApi = {
  listMySessions: async (
    accessToken: string
  ): Promise<{ data: SessionInfo[] }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/users/me/sessions`,
      {
        headers: { Authorization: `Bearer ${accessToken}` },
      }
    );
    return handleResponse(response);
  },

  revokeSession: async (
    sessionId: string,
    accessToken: string
  ): Promise<{ message: string }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/users/me/sessions/${sessionId}`,
      {
        method: "DELETE",
        headers: { Authorization: `Bearer ${accessToken}` },
      }
    );
    return handleResponse(response);
  },

  revokeOtherSessions: async (
    accessToken: string
  ): Promise<{ message: string }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/users/me/sessions`,
      {
        method: "DELETE",
        headers: { Authorization: `Bearer ${accessToken}` },
      }
    );
    return handleResponse(response);
  },

  forceLogoutUser: async (
    userId: string,
    accessToken?: string
  ): Promise<{ message: string }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/admin/users/${userId}/logout`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },
};
