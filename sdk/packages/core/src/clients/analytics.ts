import type { Auth9HttpClient } from "../http-client.js";
import type {
  LoginStats,
  LoginStatsQuery,
  LoginEvent,
  LoginEventsQuery,
  DailyTrendPoint,
  DailyTrendQuery,
} from "../types/analytics.js";
import type { PaginatedResponse } from "../types/responses.js";

export class AnalyticsClient {
  constructor(private http: Auth9HttpClient) {}

  async getLoginStats(options?: LoginStatsQuery): Promise<LoginStats> {
    const params: Record<string, string> = {};
    if (options?.period) params.period = options.period;
    if (options?.days !== undefined) params.days = String(options.days);
    if (options?.start) params.start = options.start;
    if (options?.end) params.end = options.end;
    if (options?.tenantId) params.tenant_id = options.tenantId;

    const hasParams = Object.keys(params).length > 0;
    const result = await this.http.get<{ data: LoginStats }>(
      "/api/v1/analytics/login-stats",
      hasParams ? params : undefined,
    );
    return result.data;
  }

  async listLoginEvents(
    options?: LoginEventsQuery,
  ): Promise<PaginatedResponse<LoginEvent>> {
    const params: Record<string, string> = {};
    if (options?.page !== undefined) params.page = String(options.page);
    if (options?.perPage !== undefined) params.per_page = String(options.perPage);
    if (options?.email) params.email = options.email;
    if (options?.tenantId) params.tenant_id = options.tenantId;

    const hasParams = Object.keys(params).length > 0;
    return this.http.get<PaginatedResponse<LoginEvent>>(
      "/api/v1/analytics/login-events",
      hasParams ? params : undefined,
    );
  }

  async getDailyTrend(options?: DailyTrendQuery): Promise<DailyTrendPoint[]> {
    const params: Record<string, string> = {};
    if (options?.days !== undefined) params.days = String(options.days);
    if (options?.start) params.start = options.start;
    if (options?.end) params.end = options.end;
    if (options?.tenantId) params.tenant_id = options.tenantId;

    const hasParams = Object.keys(params).length > 0;
    const result = await this.http.get<{ data: DailyTrendPoint[] }>(
      "/api/v1/analytics/daily-trend",
      hasParams ? params : undefined,
    );
    return result.data;
  }
}
