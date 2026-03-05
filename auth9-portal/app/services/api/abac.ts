import { API_BASE_URL, getHeaders, handleResponse } from "./client";

export type AbacMode = "disabled" | "shadow" | "enforce";

export interface AbacPolicyRule {
  id: string;
  effect: "allow" | "deny";
  actions: string[];
  resource_types: string[];
  priority?: number;
  condition?: unknown;
}

export interface AbacPolicyDocument {
  rules: AbacPolicyRule[];
}

export interface AbacPolicySetSummary {
  policy_set_id: string;
  tenant_id: string;
  mode: AbacMode;
  published_version_id?: string;
  published_version_no?: number;
}

export interface AbacPolicyVersionSummary {
  id: string;
  policy_set_id: string;
  version_no: number;
  status: string;
  change_note?: string;
  created_by?: string;
  created_at: string;
  published_at?: string;
}

export interface AbacPolicyListPayload {
  policy_set: AbacPolicySetSummary | null;
  versions: AbacPolicyVersionSummary[];
}

export interface AbacSimulationInput {
  action: string;
  resource_type: string;
  subject?: Record<string, unknown>;
  resource?: Record<string, unknown>;
  request?: Record<string, unknown>;
  env?: Record<string, unknown>;
}

export interface AbacSimulationResult {
  decision: "allow" | "deny" | string;
  matched_allow_rule_ids: string[];
  matched_deny_rule_ids: string[];
}

export const abacApi = {
  listPolicies: async (
    tenantId: string,
    accessToken?: string
  ): Promise<{ data: AbacPolicyListPayload }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/abac/policies`,
      {
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },

  createDraft: async (
    tenantId: string,
    input: { change_note?: string; policy: AbacPolicyDocument },
    accessToken?: string
  ): Promise<{
    data: {
      id: string;
      policy_set_id: string;
      version_no: number;
      status: string;
    };
  }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/abac/policies`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
        body: JSON.stringify(input),
      }
    );
    return handleResponse(response);
  },

  updateDraft: async (
    tenantId: string,
    versionId: string,
    input: { change_note?: string; policy: AbacPolicyDocument },
    accessToken?: string
  ): Promise<{ message: string }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/abac/policies/${versionId}`,
      {
        method: "PUT",
        headers: getHeaders(accessToken),
        body: JSON.stringify(input),
      }
    );
    return handleResponse(response);
  },

  publish: async (
    tenantId: string,
    versionId: string,
    mode: AbacMode,
    accessToken?: string
  ): Promise<{ message: string }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/abac/policies/${versionId}/publish`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
        body: JSON.stringify({ mode }),
      }
    );
    return handleResponse(response);
  },

  rollback: async (
    tenantId: string,
    versionId: string,
    mode: AbacMode,
    accessToken?: string
  ): Promise<{ message: string }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/abac/policies/${versionId}/rollback`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
        body: JSON.stringify({ mode }),
      }
    );
    return handleResponse(response);
  },

  simulate: async (
    tenantId: string,
    input: { policy?: AbacPolicyDocument; simulation: AbacSimulationInput },
    accessToken?: string
  ): Promise<{ data: AbacSimulationResult }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/tenants/${tenantId}/abac/simulate`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
        body: JSON.stringify(input),
      }
    );
    return handleResponse(response);
  },
};
