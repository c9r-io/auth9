import { API_BASE_URL, getHeaders, handleResponse } from "./client";

export interface AdaptiveMfaPolicy {
  tenant_id: string;
  mode: "disabled" | "always" | "adaptive" | "optional_enroll";
  risk_threshold: number;
  always_require_for_admins: boolean;
  trust_device_days: number;
  step_up_operations: string[];
}

export interface UpdateAdaptiveMfaPolicyInput {
  mode?: "disabled" | "always" | "adaptive" | "optional_enroll";
  risk_threshold?: number;
  always_require_for_admins?: boolean;
  trust_device_days?: number;
  step_up_operations?: string[];
}

export interface TrustedDevice {
  id: string;
  user_id: string;
  device_fingerprint: string;
  device_name: string | null;
  trusted_at: string;
  expires_at: string;
  last_used_at: string;
  revoked: boolean;
}

export const adaptiveMfaApi = {
  getPolicy: async (
    accessToken?: string
  ): Promise<{ data: AdaptiveMfaPolicy }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/mfa/adaptive-policy`, {
      headers: getHeaders(accessToken),
    });
    return handleResponse(response);
  },

  updatePolicy: async (
    input: UpdateAdaptiveMfaPolicyInput,
    accessToken?: string
  ): Promise<{ data: AdaptiveMfaPolicy }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/mfa/adaptive-policy`, {
      method: "PUT",
      headers: getHeaders(accessToken),
      body: JSON.stringify(input),
    });
    return handleResponse(response);
  },

  listTrustedDevices: async (
    accessToken?: string
  ): Promise<{ data: TrustedDevice[] }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/mfa/trusted-devices`, {
      headers: getHeaders(accessToken),
    });
    return handleResponse(response);
  },

  revokeTrustedDevice: async (
    id: string,
    accessToken?: string
  ): Promise<void> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/mfa/trusted-devices/${id}`,
      {
        method: "DELETE",
        headers: getHeaders(accessToken),
      }
    );
    await handleResponse(response);
  },

  revokeAllTrustedDevices: async (
    accessToken?: string
  ): Promise<void> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/mfa/trusted-devices`,
      {
        method: "DELETE",
        headers: getHeaders(accessToken),
      }
    );
    await handleResponse(response);
  },
};
