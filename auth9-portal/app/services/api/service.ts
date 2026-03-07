import {
  API_BASE_URL,
  ApiResponseError,
  getHeaders,
  handleResponse,
  type ApiError,
  type PaginatedResponse,
} from "./client";

export interface Service {
  id: string;
  tenant_id?: string;
  name: string;
  base_url?: string;
  redirect_uris: string[];
  logout_uris: string[];
  status: "active" | "inactive";
  created_at: string;
  updated_at: string;
}

export interface Client {
  id: string;
  service_id: string;
  client_id: string;
  name?: string;
  created_at: string;
}

// Note: Backend uses #[serde(flatten)] so Client fields are flattened
export interface ClientWithSecret extends Client {
  client_secret: string;
}

export interface CreateClientInput {
  name?: string;
}

export interface CreateServiceInput {
  name: string;
  client_id?: string;
  base_url?: string;
  redirect_uris?: string[];
  logout_uris?: string[];
  tenant_id?: string;
}

// Integration info types
export interface ServiceIntegrationInfo {
  service: {
    id: string;
    name: string;
    base_url?: string;
    redirect_uris: string[];
    logout_uris: string[];
  };
  clients: ClientIntegrationInfo[];
  endpoints: EndpointInfo;
  grpc: GrpcInfo;
  environment_variables: EnvVar[];
}

export interface ClientIntegrationInfo {
  client_id: string;
  name?: string;
  public_client: boolean;
  client_secret?: string;
  created_at: string;
}

export interface EndpointInfo {
  auth9_domain: string;
  auth9_public_url: string;
  authorize: string;
  token: string;
  callback: string;
  logout: string;
  userinfo: string;
  openid_configuration: string;
  jwks: string;
}

export interface GrpcInfo {
  address: string;
  auth_mode: string;
}

export interface EnvVar {
  key: string;
  value: string;
  description: string;
}

export const serviceApi = {
  list: async (
    tenantId?: string,
    page = 1,
    perPage = 20,
    accessToken?: string
  ): Promise<PaginatedResponse<Service>> => {
    let url = `${API_BASE_URL}/api/v1/services?page=${page}&per_page=${perPage}`;
    if (tenantId) url += `&tenant_id=${tenantId}`;
    const response = await fetch(url, { headers: getHeaders(accessToken) });
    return handleResponse(response);
  },

  get: async (
    id: string,
    accessToken?: string
  ): Promise<{ data: Service }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/services/${id}`, {
      headers: getHeaders(accessToken),
    });
    return handleResponse(response);
  },

  // Note: Backend uses #[serde(flatten)] on ServiceWithClient, so Service fields are at root level.
  // Some code paths can return service data without client details.
  create: async (
    input: CreateServiceInput,
    accessToken?: string
  ): Promise<{ data: Service & { client?: ClientWithSecret } }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/services`, {
      method: "POST",
      headers: getHeaders(accessToken),
      body: JSON.stringify(input),
    });
    return handleResponse(response);
  },

  update: async (
    id: string,
    input: Partial<CreateServiceInput>,
    accessToken?: string
  ): Promise<{ data: Service }> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/services/${id}`, {
      method: "PUT",
      headers: getHeaders(accessToken),
      body: JSON.stringify(input),
    });
    return handleResponse(response);
  },

  delete: async (id: string, accessToken?: string): Promise<void> => {
    const response = await fetch(`${API_BASE_URL}/api/v1/services/${id}`, {
      method: "DELETE",
      headers: getHeaders(accessToken),
    });
    if (!response.ok) {
      const error: ApiError = await response.json();
      throw new ApiResponseError(error, response.status);
    }
  },

  listClients: async (
    serviceId: string,
    accessToken?: string
  ): Promise<{ data: Client[] }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/services/${serviceId}/clients`,
      {
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },

  createClient: async (
    serviceId: string,
    input: CreateClientInput,
    accessToken?: string
  ): Promise<{ data: ClientWithSecret }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/services/${serviceId}/clients`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
        body: JSON.stringify(input),
      }
    );
    return handleResponse(response);
  },

  deleteClient: async (
    serviceId: string,
    clientId: string,
    accessToken?: string
  ): Promise<void> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/services/${serviceId}/clients/${clientId}`,
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

  regenerateClientSecret: async (
    serviceId: string,
    clientId: string,
    accessToken?: string
  ): Promise<{ data: { client_id: string; client_secret: string } }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/services/${serviceId}/clients/${clientId}/regenerate-secret`,
      {
        method: "POST",
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },

  getIntegration: async (
    serviceId: string,
    accessToken?: string
  ): Promise<{ data: ServiceIntegrationInfo }> => {
    const response = await fetch(
      `${API_BASE_URL}/api/v1/services/${serviceId}/integration`,
      {
        headers: getHeaders(accessToken),
      }
    );
    return handleResponse(response);
  },
};
