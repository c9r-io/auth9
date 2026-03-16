import type { Auth9HttpClient } from "../http-client.js";
import type {
  SecurityAlert,
  SecurityAlertsQuery,
} from "../types/analytics.js";
import type { PaginatedResponse } from "../types/responses.js";

export class SecurityAlertsClient {
  constructor(private http: Auth9HttpClient) {}

  async list(
    options?: SecurityAlertsQuery,
  ): Promise<PaginatedResponse<SecurityAlert>> {
    const params: Record<string, string> = {};
    if (options?.page !== undefined) params.page = String(options.page);
    if (options?.perPage !== undefined) params.per_page = String(options.perPage);
    if (options?.unresolvedOnly !== undefined)
      params.unresolved_only = String(options.unresolvedOnly);
    if (options?.severity) params.severity = options.severity;
    if (options?.alertType) params.alert_type = options.alertType;

    const hasParams = Object.keys(params).length > 0;
    return this.http.get<PaginatedResponse<SecurityAlert>>(
      "/api/v1/security/alerts",
      hasParams ? params : undefined,
    );
  }

  async resolve(id: string): Promise<SecurityAlert> {
    const result = await this.http.post<{ data: SecurityAlert }>(
      `/api/v1/security/alerts/${id}/resolve`,
    );
    return result.data;
  }
}
