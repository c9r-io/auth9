import {
  API_BASE_URL,
  ApiResponseError,
  getHeaders,
  handleResponse,
  type ApiError,
  type PaginatedResponse,
} from "./client";

export type InvitationStatus = "pending" | "accepted" | "expired" | "revoked";

export interface Invitation {
  id: string;
  tenant_id: string;
  email: string;
  role_ids: string[];
  invited_by: string;
  status: InvitationStatus;
  expires_at: string;
  accepted_at?: string;
  created_at: string;
}

export interface CreateInvitationInput {
  email: string;
  role_ids: string[];
  expires_in_hours?: number;
}

export type InvitationStatusFilter =
  | "pending"
  | "accepted"
  | "expired"
  | "revoked";

export const invitationApi = {
  list: async (
    tenantId: string,
    page = 1,
    perPage = 20,
    status?: InvitationStatusFilter,
    accessToken?: string
  ): Promise<PaginatedResponse<Invitation>> => {
    const params = new URLSearchParams({
      page: page.toString(),
      per_page: perPage.toString(),
    });
    if (status) {
      params.set("status", status);
    }
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/invitations?${params.toString()}`,
      { headers: getHeaders(accessToken) }
    );
    return handleResponse(response);
  },

  create: async (
    tenantId: string,
    input: CreateInvitationInput,
    accessToken?: string
  ): Promise<{ data: Invitation }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/invitations`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
        body: JSON.stringify(input),
      }
    );
    return handleResponse(response);
  },

  get: async (
    id: string,
    accessToken?: string
  ): Promise<{ data: Invitation }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/invitations/${id}`,
      {
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },

  revoke: async (
    id: string,
    accessToken?: string
  ): Promise<{ data: Invitation }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/invitations/${id}/revoke`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },

  resend: async (
    id: string,
    accessToken?: string
  ): Promise<{ data: Invitation }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/invitations/${id}/resend`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },

  delete: async (id: string, accessToken?: string): Promise<void> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/invitations/${id}`,
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

  accept: async (input: {
    token: string;
    email?: string;
    password?: string;
    display_name?: string;
  }): Promise<{ data: Invitation }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/invitations/accept`,
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(input),
      }
    );
    return handleResponse(response);
  },
};
