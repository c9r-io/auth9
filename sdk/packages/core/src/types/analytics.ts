export interface LoginStats {
  totalLogins: number;
  successfulLogins: number;
  failedLogins: number;
  uniqueUsers: number;
  byEventType: Record<string, number>;
  byDeviceType: Record<string, number>;
  periodStart: string;
  periodEnd: string;
}

export interface LoginEvent {
  id: number;
  userId?: string;
  email?: string;
  tenantId?: string;
  eventType: string;
  ipAddress?: string;
  userAgent?: string;
  deviceType?: string;
  location?: string;
  sessionId?: string;
  failureReason?: string;
  createdAt: string;
}

export interface AuditLog {
  id: number;
  actorId?: string;
  actorEmail?: string;
  actorDisplayName?: string;
  action: string;
  resourceType: string;
  resourceId?: string;
  oldValue?: unknown;
  newValue?: unknown;
  ipAddress?: string;
  createdAt: string;
}

export interface SecurityAlert {
  id: string;
  userId?: string;
  tenantId?: string;
  alertType: "bruteForce" | "slowBruteForce" | "newDevice" | "impossibleTravel" | "suspiciousIp";
  severity: "low" | "medium" | "high" | "critical";
  details?: Record<string, unknown>;
  resolvedAt?: string;
  resolvedBy?: string;
  createdAt: string;
}

// --- Query & Pagination types for Phase 4 sub-clients ---

export interface AuditLogQuery {
  actorId?: string;
  action?: string;
  resourceType?: string;
  resourceId?: string;
  fromDate?: string;
  toDate?: string;
  page?: number;
  perPage?: number;
}

export interface AuditLogPage {
  data: AuditLog[];
  pagination: {
    page: number;
    perPage: number;
    total: number;
    totalPages: number;
  };
}

export interface LoginStatsQuery {
  period?: string;
  days?: number;
  start?: string;
  end?: string;
  tenantId?: string;
}

export interface LoginEventsQuery {
  page?: number;
  perPage?: number;
  email?: string;
  tenantId?: string;
}

export interface DailyTrendQuery {
  days?: number;
  start?: string;
  end?: string;
  tenantId?: string;
}

export interface DailyTrendPoint {
  date: string;
  total: number;
  successful: number;
  failed: number;
}

export interface SecurityAlertsQuery {
  page?: number;
  perPage?: number;
  unresolvedOnly?: boolean;
  severity?: "low" | "medium" | "high" | "critical";
  alertType?: "brute_force" | "slow_brute_force" | "new_device" | "impossible_travel" | "suspicious_ip";
}
