# SDK Phase 2: 安全与企业功能 API 客户端

**类型**: 功能增强
**严重程度**: Medium
**影响范围**: sdk/packages/core (@auth9/core), sdk/packages/node (@auth9/node)
**前置依赖**: `sdk_phase1_core_management.md`

---

## 背景

Phase 1 覆盖了核心 CRUD 管理 API。Phase 2 聚焦企业级安全功能：身份提供商管理、SSO 连接器、SAML 应用、ABAC 策略、会话管理、Webhook 管理。这些功能是企业客户部署 Auth9 时的关键需求。

---

## 期望行为

### R1: Identity Providers 子客户端

封装 `/api/v1/identity-providers` 端点（社交登录 / 外部 IdP 管理）：

```typescript
client.identityProviders.list(): Promise<IdentityProvider[]>
client.identityProviders.get(alias: string): Promise<IdentityProvider>
client.identityProviders.create(input: CreateIdentityProviderInput): Promise<IdentityProvider>
client.identityProviders.update(alias: string, input: UpdateIdentityProviderInput): Promise<IdentityProvider>
client.identityProviders.delete(alias: string): Promise<void>
client.identityProviders.getTemplates(): Promise<IdentityProviderTemplate[]>

// 用户关联身份
client.identityProviders.listMyLinkedIdentities(): Promise<LinkedIdentity[]>
client.identityProviders.unlinkIdentity(id: string): Promise<void>
```

新增类型：

```typescript
interface IdentityProvider {
  alias: string;
  displayName: string;
  providerId: string; // "google", "github", "oidc", "saml"
  enabled: boolean;
  config: Record<string, string>;
  createdAt: string;
}

interface LinkedIdentity {
  id: string;
  provider: string;
  providerUserId: string;
  email?: string;
  linkedAt: string;
}
```

**涉及文件**:
- `sdk/packages/core/src/clients/identity-providers.ts` — 新增
- `sdk/packages/core/src/types/identity-provider.ts` — 新增

### R2: SSO Connectors 子客户端

封装 `/api/v1/tenants/{tenant_id}/sso/connectors` 端点（企业 SSO 配置）：

```typescript
client.sso.listConnectors(tenantId: string): Promise<SSOConnector[]>
client.sso.createConnector(tenantId: string, input: CreateSSOConnectorInput): Promise<SSOConnector>
client.sso.updateConnector(tenantId: string, connectorId: string, input: UpdateSSOConnectorInput): Promise<SSOConnector>
client.sso.deleteConnector(tenantId: string, connectorId: string): Promise<void>
client.sso.testConnector(tenantId: string, connectorId: string): Promise<SSOTestResult>
```

新增类型：

```typescript
interface SSOConnector {
  id: string;
  tenantId: string;
  name: string;
  protocol: "saml" | "oidc";
  domains: string[];
  config: Record<string, string>;
  enabled: boolean;
  createdAt: string;
  updatedAt: string;
}
```

**涉及文件**:
- `sdk/packages/core/src/clients/sso.ts` — 新增
- `sdk/packages/core/src/types/sso.ts` — 新增

### R3: SAML Applications 子客户端

封装 `/api/v1/tenants/{tenant_id}/saml-apps` 端点（SAML IdP 出站签发）：

```typescript
client.saml.list(tenantId: string): Promise<SamlApplication[]>
client.saml.get(tenantId: string, appId: string): Promise<SamlApplication>
client.saml.create(tenantId: string, input: CreateSamlApplicationInput): Promise<SamlApplication>
client.saml.update(tenantId: string, appId: string, input: UpdateSamlApplicationInput): Promise<SamlApplication>
client.saml.delete(tenantId: string, appId: string): Promise<void>
client.saml.getMetadata(tenantId: string, appId: string): Promise<string>  // XML
client.saml.getCertificate(tenantId: string, appId: string): Promise<string>  // PEM
client.saml.getCertificateInfo(tenantId: string, appId: string): Promise<SamlCertificateInfo>
```

**涉及文件**:
- `sdk/packages/core/src/clients/saml.ts` — 新增
- `sdk/packages/core/src/types/saml.ts` — 新增

### R4: ABAC Policies 子客户端

封装 `/api/v1/tenants/{tenant_id}/abac` 端点（属性访问控制策略）：

```typescript
client.abac.listPolicies(tenantId: string): Promise<AbacPolicy[]>
client.abac.createPolicy(tenantId: string, input: CreateAbacPolicyInput): Promise<AbacPolicy>
client.abac.updatePolicy(tenantId: string, versionId: string, input: UpdateAbacPolicyInput): Promise<AbacPolicy>
client.abac.publishPolicy(tenantId: string, versionId: string): Promise<AbacPolicy>
client.abac.rollbackPolicy(tenantId: string, versionId: string): Promise<AbacPolicy>
client.abac.simulate(tenantId: string, input: SimulateAbacInput): Promise<AbacSimulationResult>
```

**涉及文件**:
- `sdk/packages/core/src/clients/abac.ts` — 新增
- `sdk/packages/core/src/types/abac.ts` — 新增

### R5: Sessions 子客户端

封装 `/api/v1/users/me/sessions` 和 admin 端点：

```typescript
client.sessions.listMy(): Promise<SessionInfo[]>
client.sessions.revoke(id: string): Promise<void>
client.sessions.revokeAllOther(): Promise<void>
client.sessions.forceLogout(userId: string): Promise<void>
```

**涉及文件**:
- `sdk/packages/core/src/clients/sessions.ts` — 新增
- `sdk/packages/core/src/types/session.ts` — 已有，确认字段对齐

### R6: Webhooks 子客户端

封装 `/api/v1/tenants/{tenant_id}/webhooks` 端点：

```typescript
client.webhooks.list(tenantId: string): Promise<Webhook[]>
client.webhooks.get(tenantId: string, webhookId: string): Promise<Webhook>
client.webhooks.create(tenantId: string, input: CreateWebhookInput): Promise<Webhook>
client.webhooks.update(tenantId: string, webhookId: string, input: UpdateWebhookInput): Promise<Webhook>
client.webhooks.delete(tenantId: string, webhookId: string): Promise<void>
client.webhooks.test(tenantId: string, webhookId: string): Promise<WebhookTestResult>
client.webhooks.regenerateSecret(tenantId: string, webhookId: string): Promise<Webhook>
```

**涉及文件**:
- `sdk/packages/core/src/clients/webhooks.ts` — 新增
- `sdk/packages/core/src/types/webhook.ts` — 已有，补充 Input 类型

### R7: SCIM Admin 子客户端

封装 SCIM token 管理和日志查询端点：

```typescript
client.scim.listTokens(tenantId: string, connectorId: string): Promise<ScimToken[]>
client.scim.createToken(tenantId: string, connectorId: string, input: CreateScimTokenInput): Promise<ScimTokenWithValue>
client.scim.revokeToken(tenantId: string, connectorId: string, tokenId: string): Promise<void>
client.scim.listLogs(tenantId: string, connectorId: string, options?: ScimLogQuery): Promise<ScimLog[]>
client.scim.listGroupMappings(tenantId: string, connectorId: string): Promise<ScimGroupMapping[]>
client.scim.updateGroupMappings(tenantId: string, connectorId: string, mappings: ScimGroupMapping[]): Promise<ScimGroupMapping[]>
```

**涉及文件**:
- `sdk/packages/core/src/clients/scim.ts` — 新增
- `sdk/packages/core/src/types/scim.ts` — 新增

### R8: Tenant Services 子客户端

封装 `/api/v1/tenants/{tenant_id}/services` 端点：

```typescript
client.tenantServices.list(tenantId: string): Promise<TenantServiceInfo[]>
client.tenantServices.toggle(tenantId: string, input: ToggleTenantServiceInput): Promise<void>
client.tenantServices.getEnabled(tenantId: string): Promise<Service[]>
```

**涉及文件**:
- `sdk/packages/core/src/clients/tenant-services.ts` — 新增
- `sdk/packages/core/src/types/tenant-service.ts` — 新增

---

## 验收标准

- [ ] 8 个子客户端全部实现并通过 `Auth9Client` 暴露
- [ ] 所有新增类型完整定义并从 `@auth9/core` 导出
- [ ] 单元测试覆盖所有方法
- [ ] `npm run build` 通过
