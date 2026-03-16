import type { Auth9HttpClient } from "../http-client.js";
import type { AuditLogQuery, AuditLogPage } from "../types/analytics.js";

export class AuditLogsClient {
  constructor(private http: Auth9HttpClient) {}

  async list(options?: AuditLogQuery): Promise<AuditLogPage> {
    const params: Record<string, string> = {};
    if (options?.actorId) params.actor_id = options.actorId;
    if (options?.action) params.action = options.action;
    if (options?.resourceType) params.resource_type = options.resourceType;
    if (options?.resourceId) params.resource_id = options.resourceId;
    if (options?.fromDate) params.from_date = options.fromDate;
    if (options?.toDate) params.to_date = options.toDate;
    if (options?.page !== undefined) params.page = String(options.page);
    if (options?.perPage !== undefined) params.per_page = String(options.perPage);

    const hasParams = Object.keys(params).length > 0;
    return this.http.get<AuditLogPage>(
      "/api/v1/audit-logs",
      hasParams ? params : undefined,
    );
  }
}
