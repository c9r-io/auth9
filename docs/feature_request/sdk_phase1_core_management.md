# SDK Phase 1: 核心管理 API 客户端

**类型**: 功能增强
**严重程度**: High
**影响范围**: sdk/packages/core (@auth9/core)
**前置依赖**: 无

---

## 背景

Auth9 SDK 的 `Auth9Client` 当前仅实现了 Actions API 客户端（10 个端点）。auth9-core 提供了 160+ 个 REST 端点，但 SDK 没有对应的管理 API 封装，开发者必须手动构造 HTTP 请求来管理租户、用户、服务等核心资源。

SDK 已定义了 `Tenant`、`User`、`Service`、`Client`、`Role`、`Permission` 等类型，但这些类型没有被任何客户端方法使用（orphaned types）。

### 设计原则

- 遵循现有 Actions API 的子客户端模式：`client.tenants.list()`、`client.users.get(id)`
- 复用 `Auth9HttpClient` 基础设施（重试、超时、错误映射）
- 每个域一个子客户端类，通过 `Auth9Client` 聚合暴露
- Input 类型与 auth9-core 的请求体对齐

---

## 期望行为

### R1: Tenants 子客户端

封装 `/api/v1/tenants` 端点：

```typescript
client.tenants.list(): Promise<Tenant[]>
client.tenants.get(id: string): Promise<Tenant>
client.tenants.create(input: CreateTenantInput): Promise<Tenant>
client.tenants.update(id: string, input: UpdateTenantInput): Promise<Tenant>
client.tenants.delete(id: string): Promise<void>
client.tenants.listUsers(tenantId: string): Promise<User[]>
client.tenants.getMaliciousIpBlacklist(tenantId: string): Promise<MaliciousIpBlacklist>
client.tenants.updateMaliciousIpBlacklist(tenantId: string, input: UpdateMaliciousIpBlacklistInput): Promise<MaliciousIpBlacklist>
```

新增 Input 类型：

```typescript
interface CreateTenantInput {
  name: string;
  slug: string;
  logoUrl?: string;
  settings?: Record<string, unknown>;
}

interface UpdateTenantInput {
  name?: string;
  slug?: string;
  logoUrl?: string;
  settings?: Record<string, unknown>;
  status?: "active" | "inactive" | "suspended";
}
```

**涉及文件**:
- `sdk/packages/core/src/clients/tenants.ts` — 新增
- `sdk/packages/core/src/types/tenant.ts` — 补充 Input 类型
- `sdk/packages/core/src/auth9-client.ts` — 注册子客户端

### R2: Users 子客户端

封装 `/api/v1/users` 端点：

```typescript
client.users.list(): Promise<User[]>
client.users.get(id: string): Promise<User>
client.users.getMe(): Promise<User>
client.users.updateMe(input: UpdateUserInput): Promise<User>
client.users.create(input: CreateUserInput): Promise<User>
client.users.update(id: string, input: UpdateUserInput): Promise<User>
client.users.delete(id: string): Promise<void>
client.users.enableMfa(id: string): Promise<void>
client.users.disableMfa(id: string): Promise<void>
client.users.getTenants(id: string): Promise<Tenant[]>
client.users.addToTenant(id: string, input: AddUserToTenantInput): Promise<void>
client.users.removeFromTenant(userId: string, tenantId: string): Promise<void>
client.users.updateRoleInTenant(userId: string, tenantId: string, input: UpdateUserRoleInput): Promise<void>
```

**涉及文件**:
- `sdk/packages/core/src/clients/users.ts` — 新增
- `sdk/packages/core/src/types/user.ts` — 补充 Input 类型

### R3: Services 子客户端

封装 `/api/v1/services` 端点：

```typescript
client.services.list(): Promise<Service[]>
client.services.get(id: string): Promise<Service>
client.services.create(input: CreateServiceInput): Promise<Service>
client.services.update(id: string, input: UpdateServiceInput): Promise<Service>
client.services.delete(id: string): Promise<void>
client.services.getIntegrationInfo(id: string): Promise<ServiceIntegration>
client.services.listClients(serviceId: string): Promise<Client[]>
client.services.createClient(serviceId: string, input: CreateClientInput): Promise<ClientWithSecret>
client.services.deleteClient(serviceId: string, clientId: string): Promise<void>
client.services.regenerateClientSecret(serviceId: string, clientId: string): Promise<ClientWithSecret>
```

**涉及文件**:
- `sdk/packages/core/src/clients/services.ts` — 新增
- `sdk/packages/core/src/types/service.ts` — 补充 Input 类型和 ServiceIntegration 类型

### R4: Roles & Permissions 子客户端

封装 RBAC 管理端点：

```typescript
// Roles
client.roles.list(serviceId: string): Promise<Role[]>
client.roles.get(id: string): Promise<RoleWithPermissions>
client.roles.create(input: CreateRoleInput): Promise<Role>
client.roles.update(id: string, input: UpdateRoleInput): Promise<Role>
client.roles.delete(id: string): Promise<void>
client.roles.assignPermission(roleId: string, permissionId: string): Promise<void>
client.roles.removePermission(roleId: string, permissionId: string): Promise<void>

// Permissions
client.permissions.list(serviceId: string): Promise<Permission[]>
client.permissions.create(input: CreatePermissionInput): Promise<Permission>
client.permissions.delete(id: string): Promise<void>

// RBAC Assignment
client.rbac.assignRoles(input: AssignRolesInput): Promise<void>
client.rbac.getUserRoles(userId: string, tenantId: string): Promise<UserRolesInTenant>
client.rbac.getUserAssignedRoles(userId: string, tenantId: string): Promise<Role[]>
client.rbac.unassignRole(userId: string, tenantId: string, roleId: string): Promise<void>
```

**涉及文件**:
- `sdk/packages/core/src/clients/roles.ts` — 新增
- `sdk/packages/core/src/clients/permissions.ts` — 新增
- `sdk/packages/core/src/clients/rbac.ts` — 新增
- `sdk/packages/core/src/types/rbac.ts` — 补充 Input 类型

### R5: Invitations 子客户端

封装 `/api/v1/invitations` 和 `/api/v1/tenants/{id}/invitations` 端点：

```typescript
client.invitations.list(tenantId: string): Promise<Invitation[]>
client.invitations.get(id: string): Promise<Invitation>
client.invitations.create(tenantId: string, input: CreateInvitationInput): Promise<Invitation>
client.invitations.delete(id: string): Promise<void>
client.invitations.revoke(id: string): Promise<void>
client.invitations.resend(id: string): Promise<void>
client.invitations.validate(token: string): Promise<InvitationValidation>
client.invitations.accept(input: AcceptInvitationInput): Promise<void>
```

**涉及文件**:
- `sdk/packages/core/src/clients/invitations.ts` — 新增
- `sdk/packages/core/src/types/invitation.ts` — 补充 Input 类型

### R6: 测试覆盖

每个子客户端需配套单元测试，mock HTTP 层验证：
- 正确的 HTTP method 和 path
- 请求体序列化
- 响应反序列化
- 错误映射（404 → NotFoundError，409 → ConflictError 等）

**涉及文件**:
- `sdk/packages/core/src/clients/__tests__/tenants.test.ts`
- `sdk/packages/core/src/clients/__tests__/users.test.ts`
- `sdk/packages/core/src/clients/__tests__/services.test.ts`
- `sdk/packages/core/src/clients/__tests__/roles.test.ts`
- `sdk/packages/core/src/clients/__tests__/invitations.test.ts`

---

## 验收标准

- [ ] 5 个子客户端全部实现并通过 `Auth9Client` 暴露
- [ ] 所有 orphaned types（Tenant, User, Service, Role, Permission, Invitation）被客户端方法引用
- [ ] Input 类型与 auth9-core 请求体一致
- [ ] 单元测试覆盖所有方法的 happy path 和主要错误场景
- [ ] `@auth9/core` 的 `index.ts` 导出所有新增类型
- [ ] `npm run build` 通过，无类型错误
