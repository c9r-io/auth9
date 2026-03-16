# SDK Phase 4: 可观测性与系统配置 API 客户端

**类型**: 功能增强
**严重程度**: Low
**影响范围**: sdk/packages/core (@auth9/core)
**前置依赖**: `sdk_phase1_core_management.md`

---

## 背景

Phase 4 覆盖运维和管理面 API：审计日志、登录分析、安全告警、系统设置（邮件/品牌）、邮件模板管理。这些 API 通常由管理后台或运维工具调用，优先级低于核心管理和安全功能。

---

## 期望行为

### R1: Audit Logs 子客户端

封装 `/api/v1/audit-logs` 端点：

```typescript
client.auditLogs.list(options?: AuditLogQuery): Promise<AuditLogPage>
```

```typescript
interface AuditLogQuery {
  actorId?: string;
  action?: string;
  resourceType?: string;
  resourceId?: string;
  startDate?: string;  // ISO 8601
  endDate?: string;
  page?: number;
  pageSize?: number;
}

interface AuditLogPage {
  data: AuditLog[];
  total: number;
  page: number;
  pageSize: number;
}
```

**涉及文件**:
- `sdk/packages/core/src/clients/audit-logs.ts` — 新增
- `sdk/packages/core/src/types/analytics.ts` — 已有 AuditLog 类型，补充 Query/Page 类型

### R2: Analytics 子客户端

封装 `/api/v1/analytics` 端点：

```typescript
client.analytics.getLoginStats(options?: AnalyticsQuery): Promise<LoginStats>
client.analytics.listLoginEvents(options?: LoginEventQuery): Promise<LoginEvent[]>
client.analytics.getDailyTrend(options?: DailyTrendQuery): Promise<DailyTrendData[]>
```

```typescript
interface AnalyticsQuery {
  tenantId?: string;
  startDate?: string;
  endDate?: string;
}

interface DailyTrendData {
  date: string;
  totalLogins: number;
  uniqueUsers: number;
  failedLogins: number;
}
```

**涉及文件**:
- `sdk/packages/core/src/clients/analytics.ts` — 新增
- `sdk/packages/core/src/types/analytics.ts` — 已有部分类型，补充 DailyTrendData

### R3: Security Alerts 子客户端

封装 `/api/v1/security/alerts` 端点：

```typescript
client.securityAlerts.list(options?: SecurityAlertQuery): Promise<SecurityAlert[]>
client.securityAlerts.resolve(id: string, input?: ResolveAlertInput): Promise<SecurityAlert>
```

```typescript
interface SecurityAlertQuery {
  tenantId?: string;
  severity?: "low" | "medium" | "high" | "critical";
  resolved?: boolean;
  limit?: number;
}

interface ResolveAlertInput {
  resolution?: string;
}
```

**涉及文件**:
- `sdk/packages/core/src/clients/security-alerts.ts` — 新增
- `sdk/packages/core/src/types/analytics.ts` — 已有 SecurityAlert 类型

### R4: System Settings 子客户端

封装 `/api/v1/system` 端点（邮件配置、IP 黑名单）：

```typescript
client.system.getEmailSettings(): Promise<EmailSettings>
client.system.updateEmailSettings(input: UpdateEmailSettingsInput): Promise<EmailSettings>
client.system.testEmailConnection(): Promise<EmailTestResult>
client.system.sendTestEmail(input: SendTestEmailInput): Promise<void>
client.system.getMaliciousIpBlacklist(): Promise<MaliciousIpBlacklist>
client.system.updateMaliciousIpBlacklist(input: UpdateMaliciousIpBlacklistInput): Promise<MaliciousIpBlacklist>
```

新增类型：

```typescript
interface EmailSettings {
  provider: "smtp" | "ses" | "sendgrid";
  from: string;
  host?: string;
  port?: number;
  username?: string;
  encryption?: "tls" | "starttls" | "none";
  configured: boolean;
}
```

**涉及文件**:
- `sdk/packages/core/src/clients/system.ts` — 新增
- `sdk/packages/core/src/types/system.ts` — 新增

### R5: Email Templates 子客户端

封装 `/api/v1/system/email-templates` 端点：

```typescript
client.emailTemplates.list(): Promise<EmailTemplate[]>
client.emailTemplates.get(type: string): Promise<EmailTemplate>
client.emailTemplates.update(type: string, input: UpdateEmailTemplateInput): Promise<EmailTemplate>
client.emailTemplates.reset(type: string): Promise<void>
client.emailTemplates.preview(type: string, input?: PreviewTemplateInput): Promise<EmailPreview>
client.emailTemplates.sendTest(type: string, input: SendTestTemplateInput): Promise<void>
```

```typescript
interface EmailTemplate {
  type: string;  // "invitation", "password_reset", "email_verification", etc.
  subject: string;
  htmlBody: string;
  textBody?: string;
  isCustomized: boolean;
  updatedAt?: string;
}
```

**涉及文件**:
- `sdk/packages/core/src/clients/email-templates.ts` — 新增
- `sdk/packages/core/src/types/email-template.ts` — 新增

### R6: Branding 子客户端

封装品牌配置端点：

```typescript
// 系统级
client.branding.get(): Promise<BrandingConfig>
client.branding.update(input: UpdateBrandingInput): Promise<BrandingConfig>
client.branding.getPublic(): Promise<BrandingConfig>  // 无需认证

// Service 级
client.branding.getForService(serviceId: string): Promise<BrandingConfig>
client.branding.updateForService(serviceId: string, input: UpdateBrandingInput): Promise<BrandingConfig>
client.branding.deleteForService(serviceId: string): Promise<void>
```

```typescript
interface BrandingConfig {
  logoUrl?: string;
  faviconUrl?: string;
  primaryColor?: string;
  backgroundColor?: string;
  companyName?: string;
  supportEmail?: string;
  customCss?: string;
}
```

**涉及文件**:
- `sdk/packages/core/src/clients/branding.ts` — 新增
- `sdk/packages/core/src/types/branding.ts` — 新增

---

## 验收标准

- [ ] 6 个子客户端全部实现
- [ ] 分页类型（AuditLogPage）支持通用分页模式
- [ ] 单元测试覆盖所有方法
- [ ] `npm run build` 通过
- [ ] SDK README 更新，列出所有可用子客户端
