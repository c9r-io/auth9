import { API_BASE_URL, getHeaders, handleResponse } from "./client";

export interface TenantRiskPolicy {
  tenant_id: string;
  mfa_threshold: number;
  block_threshold: number;
  notify_admin: boolean;
  auto_lock_account: boolean;
}

export interface UpdateRiskPolicyInput {
  mfa_threshold?: number;
  block_threshold?: number;
  notify_admin?: boolean;
  auto_lock_account?: boolean;
}

export const riskApi = {
  getRiskPolicy: async (
    accessToken?: string
  ): Promise<{ data: TenantRiskPolicy }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/security/risk-policy`, {
      headers: getHeaders(accessToken),
    });
    return handleResponse(response);
  },

  updateRiskPolicy: async (
    input: UpdateRiskPolicyInput,
    accessToken?: string
  ): Promise<{ data: TenantRiskPolicy }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/security/risk-policy`, {
      method: "PUT",
      headers: getHeaders(accessToken),
      body: JSON.stringify(input),
    });
    return handleResponse(response);
  },
};
