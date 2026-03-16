import type { Auth9HttpClient } from "../http-client.js";
import type {
  Organization,
  CreateOrganizationInput,
} from "../types/organization.js";
import type { Tenant } from "../types/tenant.js";

export class OrganizationsClient {
  constructor(private http: Auth9HttpClient) {}

  async create(input: CreateOrganizationInput): Promise<Organization> {
    const result = await this.http.post<{ data: Organization }>(
      "/api/v1/organizations",
      input,
    );
    return result.data;
  }

  async getMyTenants(serviceId?: string): Promise<Tenant[]> {
    const params = serviceId ? { service_id: serviceId } : undefined;
    const result = await this.http.get<{ data: Tenant[] }>(
      "/api/v1/users/me/tenants",
      params,
    );
    return result.data;
  }
}
