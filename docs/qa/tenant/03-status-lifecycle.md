# 租户管理 - 状态生命周期测试

**模块**: 租户管理
**测试范围**: 租户状态流转（Active/Inactive/Suspended）及其业务影响
**场景数**: 5
**优先级**: 高

---

## 背景说明

Auth9 租户有三种状态：

| 状态 | 说明 |
|------|------|
| `active` | 正常运行，所有功能可用（默认） |
| `inactive` | 已停用，业务功能应受限 |
| `suspended` | 已暂停，可能因违规或欠费 |

**重要说明**：
- 状态变更**必须通过 API 进行**，不要直接修改数据库。直接修改数据库不会触发缓存清除和审计日志。
- 通过 API 变更状态时，Redis 缓存会**自动清除**，审计日志会**自动记录**。
- Portal UI 租户详情页提供状态下拉选择器，可直接编辑状态。
- 非 active 状态的租户会**阻止写操作**（创建 Webhook、发送邀请、添加用户）和 **Token Exchange**。

状态通过 `PUT /api/v1/tenants/{id}` 更新：
```json
{
  "status": "suspended"
}
```

---

## JWT Key Synchronization

> **JWT Token Generation**: Always use `node .claude/skills/tools/gen_token.js` which reads the private key from `.env` (matching the Docker container). Other scripts may use hardcoded key paths that don't match.

| 症状 | 原因 | 修复方法 |
|------|------|----------|
| JWT 签名验证失败 (401) | 使用了 hardcoded key path 的脚本，与 Docker 容器中的 key 不一致 | 改用 `node .claude/skills/tools/gen_token.js`，它从 `.env` 读取私钥 |

---

## 场景 1：入口可见性 - 从租户列表进入状态编辑并设为 Inactive

### 初始状态
- 存在租户 id=`{tenant_id}`，status=`active`
- 该租户有关联用户和服务

### 目的
验证从 Portal 可见入口进入租户详情并将状态切换为 inactive

### 测试操作流程
1. 从 Dashboard 左侧导航点击「Tenants」
2. 在租户列表中点击目标租户名称进入详情页
3. 修改状态为 `Inactive`（或调用 API）：
   ```bash
   PUT /api/v1/tenants/{tenant_id}
   { "status": "inactive" }
   ```
4. 刷新页面确认状态显示

### 预期结果
- 状态更新成功
- 租户详情页显示状态为 `Inactive`
- 审计日志记录状态变更

### 预期数据状态
```sql
SELECT status FROM tenants WHERE id = '{tenant_id}';
-- 预期: inactive

SELECT action, old_value, new_value FROM audit_logs
WHERE resource_type = 'tenant' AND resource_id = '{tenant_id}'
ORDER BY created_at DESC LIMIT 1;
-- 预期: action = 'tenant.update'，包含 status 变更
```

---

## 场景 2：将租户状态设为 Suspended

### 初始状态
- 存在租户 id=`{tenant_id}`，status=`active`

### 目的
验证租户可以被暂停

### 测试操作流程
1. 调用 API 暂停租户：
   ```bash
   PUT /api/v1/tenants/{tenant_id}
   { "status": "suspended" }
   ```
2. 查看租户列表，确认状态标识

### 预期结果
- 状态更新成功
- 租户列表中显示 `Suspended` 状态标识

### 预期数据状态
```sql
SELECT status FROM tenants WHERE id = '{tenant_id}';
-- 预期: suspended
```

---

## 场景 3：恢复 Suspended 租户为 Active

### 初始状态
- 存在租户 id=`{tenant_id}`，status=`suspended`

### 目的
验证租户可以从暂停状态恢复

### 测试操作流程
1. 调用 API 恢复租户：
   ```bash
   PUT /api/v1/tenants/{tenant_id}
   { "status": "active" }
   ```
2. 验证租户下的用户能否正常操作

### 预期结果
- 状态更新为 `active`
- 租户功能恢复正常

### 预期数据状态
```sql
SELECT status FROM tenants WHERE id = '{tenant_id}';
-- 预期: active

-- 审计日志应记录两次状态变更
SELECT action, old_value, new_value FROM audit_logs
WHERE resource_type = 'tenant' AND resource_id = '{tenant_id}'
  AND action = 'tenant.update'
ORDER BY created_at DESC LIMIT 2;
-- 预期: 2 条记录，分别为 suspended→active 和 active→suspended
```

---

## 场景 4：Inactive 租户的 Token Exchange 行为

### 初始状态
- 租户 id=`{tenant_id}`，status=`inactive`
- 用户已登录，持有 Identity Token
- 用户是该租户的成员

### 目的
验证非 active 状态的租户在 Token Exchange 时的行为

### 测试操作流程
1. 调用 gRPC Token Exchange 请求该租户的 Access Token：
   ```protobuf
   ExchangeTokenRequest {
     identity_token: "<Identity Token>"
     tenant_id: "{tenant_id}"
   }
   ```
2. 检查响应

### 预期结果
- 返回错误「Tenant is not active (status: 'inactive')」，HTTP 403 / gRPC PermissionDenied
- 拒绝发放 Token

### 预期数据状态
```sql
SELECT status FROM tenants WHERE id = '{tenant_id}';
-- 预期: inactive
```

---

## 场景 5：租户状态对管理操作的影响

### 初始状态
- 租户 id=`{tenant_id}`，status=`suspended`

### 目的
验证暂停状态的租户能否继续进行管理操作

### 测试操作流程
1. 尝试在该租户下创建用户：
   ```bash
   POST /api/v1/users
   { "email": "new@example.com", ... }
   ```
   然后添加到该租户
2. 尝试在该租户下创建邀请：
   ```bash
   POST /api/v1/tenants/{tenant_id}/invitations
   { "email": "invite@example.com", ... }
   ```
3. 尝试在该租户下创建 Webhook：
   ```bash
   POST /api/v1/tenants/{tenant_id}/webhooks
   { "url": "https://example.com/hook", ... }
   ```

### 预期结果
- 所有写操作应被阻止，返回 HTTP 403 错误
- 错误信息包含「Tenant is not active (status: 'suspended'). Write operations are not allowed on non-active tenants.」

> **故障排除：收到 422 而非 403**
>
> 租户状态检查（`require_active()`）在 handler 内部、请求体反序列化**之后**执行。如果请求体格式不正确，axum 会在 handler 代码执行前返回 422 验证错误，根本不会触发租户状态检查。
>
> **Invitation 端点** (`POST /api/v1/tenants/{id}/invitations`) 要求 `email` 和 `role_ids`（UUID 数组），不是 `role`（字符串）：
> ```json
> {
>   "email": "invite@example.com",
>   "role_ids": ["aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"]
> }
> ```
>
> **Webhook 端点** (`POST /api/v1/tenants/{id}/webhooks`) 要求 `name`、`url` 和 `events`（字符串数组）：
> ```json
> {
>   "name": "My Webhook",
>   "url": "https://example.com/hook",
>   "events": ["user.created", "user.updated"]
> }
> ```
>
> 使用错误的 payload 格式（如 `{ "email": "...", "role": "admin" }`）会导致 422，这不是 bug，而是请求体不符合 API schema。

> **故障排除：Invitation 端点返回 404**
>
> `POST /api/v1/tenants/{tenant_id}/invitations` 位于 `protected_routes`，**必须** 携带
> 有效的 `Authorization: Bearer <token>`。没有 token 时 axum 的 auth middleware 会拒绝
> 请求（401）；但若 `tenant_id` 路径参数不是有效 UUID 或数据库中不存在，则处理链路里
> 的 `require_active` / policy 检查会返回 404 Resource not found。排查步骤：
>
> 1. 确认 token 有效：`curl -sf http://localhost:8080/api/v1/users/me -H "Authorization: Bearer $TOKEN"` 返回 200。
> 2. 确认 `{tenant_id}` 是标准 UUID 且在 `tenant_users` 表中与该 token 用户有关联。
> 3. 平台租户 (`platform` slug) 不是普通业务租户，不要在本场景使用——使用「测试数据准备 SQL」中创建的 `aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee`。

> **故障排除：创建用户返回 201 而非 403（挂起租户写入未被拦截）**
>
> `POST /api/v1/users` 在以下两种情况之一会执行 `tenant_service.require_active()`：
>
> 1. 请求体中显式传入了 `tenant_id` 字段；或
> 2. 请求使用的 Access Token 带有 `tenant_id` claim（Tenant Access Token）。
>
> 如果 QA 使用 **公共注册路径**（无 Authorization header）或 **Identity Token**，并且请求体
> 不包含 `tenant_id`，后端无法推断「属于哪个挂起租户」，因此不会走 `require_active` 分支，
> 会返回 201 创建一个全局用户。这**不是 bug**——被挂起的租户从未被涉及。
>
> 正确的 Scenario 5 测试命令：
>
> ```bash
> # 生成目标（suspended）租户的 Tenant Access Token
> TOKEN=$(node scripts/qa/gen-access-token.js "$ADMIN_USER_ID" "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee" "admin")
>
> curl -s -X POST http://localhost:8080/api/v1/users \
>   -H "Authorization: Bearer $TOKEN" \
>   -H "Content-Type: application/json" \
>   -d '{
>     "tenant_id": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
>     "user": {"email": "new@example.com", "display_name": "New"},
>     "password": "SecurePass123!"
>   }'
> # 预期: HTTP 403 "Tenant is not active (status: 'suspended')..."
> ```
>
> 关键：**请求体里一定要带 `tenant_id`**，否则你测的是公共注册，而不是「挂起租户下创建用户」。

---

## 测试数据准备 SQL

> **重要**: 所有 `id` 字段必须使用标准 UUID 格式，否则会导致 `ColumnDecode` 错误。

```sql
-- 创建测试租户（状态为 active）
INSERT INTO tenants (id, name, slug, status, settings, created_at, updated_at)
VALUES (
  'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee',
  'Status Test Tenant',
  'status-test',
  'active',
  '{}',
  NOW(),
  NOW()
);

-- 添加测试用户关联
INSERT INTO tenant_users (id, tenant_id, user_id, role_in_tenant, joined_at)
VALUES (
  UUID(),
  'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee',
  '{user_id}',
  'admin',
  NOW()
);

-- 清理
DELETE FROM tenants WHERE slug = 'status-test';
```

### 步骤 0: 验证测试数据完整性

```sql
SELECT COUNT(*) AS non_uuid_count FROM tenants
WHERE id NOT REGEXP '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
  AND slug = 'status-test';
-- 预期: 0
```

---

## 通用场景：认证状态检查

### 初始状态
- 用户已登录管理后台
- 页面正常显示

### 目的
验证页面正确检查认证状态，未登录或 session 失效时重定向到登录页

### 测试操作流程
1. 通过以下任一方式构造未认证状态：
   - 使用浏览器无痕/隐私窗口访问
   - 手动清除 auth9_session cookie
   - 在当前会话点击「Sign out」退出登录
2. 访问本页面对应的 URL

### 预期结果
- 页面自动重定向到 `/login`
- 不显示 dashboard 内容
- 登录后可正常访问原页面

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | 设为 Inactive | ☐ | | | |
| 2 | 设为 Suspended | ☐ | | | |
| 3 | 恢复为 Active | ☐ | | | |
| 4 | Inactive 租户 Token Exchange | ☐ | | | |
| 5 | Suspended 租户管理操作 | ☐ | | | |
| 6 | 认证状态检查 | ☐ | | | |
