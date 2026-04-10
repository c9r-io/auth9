# 认证流程 - IdP First-Login Default Policy Security Hardening 测试

**模块**: 认证流程
**测试范围**: IdP first_login_policy 默认值安全加固（create_new 默认、auto_merge 显式设置、策略更新、Portal UI 下拉默认、安全警告面板）
**场景数**: 5

---

## 背景知识

Auth9 的 Identity Provider 支持三种 `first_login_policy` 策略：

| 策略 | 行为 |
|------|------|
| `create_new` | 社交登录首次使用时创建独立账号，不关联已有用户（**新默认值**） |
| `prompt_confirm` | 邮箱匹配时重定向到确认页面，由用户决定是否关联 |
| `auto_merge` | 邮箱匹配时自动合并到已有账号（存在账户劫持风险） |

本次安全加固变更：

1. **Backend**: 所有 `first_login_policy` 默认值从 `auto_merge` 改为 `create_new`
2. **DB Migration**: 列默认值改为 `create_new`（已有行数据不变）
3. **Portal UI**: IdP 创建/编辑对话框新增 `first_login_policy` 下拉选择和 `trust_email` 开关
4. **安全警告**: 选择 `auto_merge` 或启用 `trust_email` 时显示红色警告面板
5. **Telemetry**: AutoMerge 代码路径新增 `tracing::warn!` 日志

---

## 场景 1：创建 IdP 不指定 first_login_policy — 验证默认值为 create_new

### 步骤 0：Gate Check

```bash
# 确认 Auth9 Core 正在运行
curl -sf http://localhost:8080/health | jq .

# 获取管理员 Token
TOKEN=$(.claude/skills/tools/gen-admin-token.sh)
echo "$TOKEN" | head -c 20
```

### 初始状态
- Auth9 Core 正在运行
- 持有有效的管理员 Token

### 目的
验证创建 IdP 时不指定 `first_login_policy` 字段，API 自动使用 `create_new` 作为默认值

### 测试操作流程

```bash
TOKEN=$(.claude/skills/tools/gen-admin-token.sh)

# 创建 IdP，不指定 first_login_policy
curl -s -X POST http://localhost:8080/api/v1/identity-providers \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "alias": "test-default-policy",
    "display_name": "Test Default Policy IdP",
    "provider_id": "oidc",
    "enabled": false,
    "config": {
      "client_id": "test-client-id",
      "client_secret": "test-client-secret",
      "authorization_url": "https://sso.example.com/authorize",
      "token_url": "https://sso.example.com/token",
      "user_info_url": "https://sso.example.com/userinfo"
    }
  }' | jq .

# 验证返回的 first_login_policy 为 create_new
curl -s http://localhost:8080/api/v1/identity-providers \
  -H "Authorization: Bearer $TOKEN" | jq '.data[] | select(.alias == "test-default-policy") | {alias, first_login_policy, trust_email}'
```

### 预期结果
- POST 返回 201 Created
- 返回的 JSON 中 `first_login_policy` = `"create_new"`
- `trust_email` = `false`（默认值）

### 预期数据状态

```sql
-- 验证数据库中的默认值
SELECT alias, first_login_policy, trust_email
FROM identity_providers
WHERE alias = 'test-default-policy';
```

**断言**:
- `first_login_policy` = `create_new`
- `trust_email` = `0`（false）

### 清理

```bash
# 删除测试 IdP
curl -s -X DELETE http://localhost:8080/api/v1/identity-providers/test-default-policy \
  -H "Authorization: Bearer $TOKEN" | jq .
```

---

## 场景 2：创建 IdP 显式设置 auto_merge — 验证 API 接受并存储

### 步骤 0：Gate Check

```bash
TOKEN=$(.claude/skills/tools/gen-admin-token.sh)
curl -sf http://localhost:8080/health | jq .
```

### 初始状态
- Auth9 Core 正在运行
- 持有有效的管理员 Token

### 目的
验证 API 仍然接受 `auto_merge` 作为显式值（向后兼容），并正确存储

### 测试操作流程

```bash
TOKEN=$(.claude/skills/tools/gen-admin-token.sh)

# 创建 IdP，显式指定 first_login_policy 为 auto_merge
curl -s -X POST http://localhost:8080/api/v1/identity-providers \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "alias": "test-auto-merge",
    "display_name": "Test Auto Merge IdP",
    "provider_id": "oidc",
    "enabled": false,
    "first_login_policy": "auto_merge",
    "trust_email": true,
    "config": {
      "client_id": "test-client-id",
      "client_secret": "test-client-secret",
      "authorization_url": "https://sso.example.com/authorize",
      "token_url": "https://sso.example.com/token",
      "user_info_url": "https://sso.example.com/userinfo"
    }
  }' | jq '{first_login_policy, trust_email}'

# 验证 GET 返回一致
curl -s http://localhost:8080/api/v1/identity-providers \
  -H "Authorization: Bearer $TOKEN" | jq '.data[] | select(.alias == "test-auto-merge") | {alias, first_login_policy, trust_email}'
```

### 预期结果
- POST 返回 201 Created
- `first_login_policy` = `"auto_merge"`
- `trust_email` = `true`

### 预期数据状态

```sql
-- 验证显式值被正确存储
SELECT alias, first_login_policy, trust_email
FROM identity_providers
WHERE alias = 'test-auto-merge';
```

**断言**:
- `first_login_policy` = `auto_merge`
- `trust_email` = `1`（true）

### 清理

```bash
curl -s -X DELETE http://localhost:8080/api/v1/identity-providers/test-auto-merge \
  -H "Authorization: Bearer $TOKEN" | jq .
```

---

## 场景 3：更新 IdP 的 first_login_policy 从 create_new 到 prompt_confirm

### 步骤 0：Gate Check

```bash
TOKEN=$(.claude/skills/tools/gen-admin-token.sh)
curl -sf http://localhost:8080/health | jq .
```

### 初始状态
- Auth9 Core 正在运行
- 已存在一个 `first_login_policy = create_new` 的 IdP（场景 1 可复用，或新建）

### 目的
验证通过 PUT API 可以将 IdP 的 `first_login_policy` 从 `create_new` 更新为 `prompt_confirm`，且更新后 GET 返回新值

### 测试操作流程

```bash
TOKEN=$(.claude/skills/tools/gen-admin-token.sh)

# 1. 创建一个默认策略的 IdP
curl -s -X POST http://localhost:8080/api/v1/identity-providers \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "alias": "test-update-policy",
    "display_name": "Test Update Policy IdP",
    "provider_id": "oidc",
    "enabled": false,
    "config": {
      "client_id": "test-client-id",
      "client_secret": "test-client-secret",
      "authorization_url": "https://sso.example.com/authorize",
      "token_url": "https://sso.example.com/token",
      "user_info_url": "https://sso.example.com/userinfo"
    }
  }' | jq '{alias, first_login_policy}'

# 2. 确认初始值为 create_new
curl -s http://localhost:8080/api/v1/identity-providers \
  -H "Authorization: Bearer $TOKEN" | jq '.data[] | select(.alias == "test-update-policy") | .first_login_policy'
# 预期: "create_new"

# 3. 更新为 prompt_confirm
curl -s -X PUT http://localhost:8080/api/v1/identity-providers/test-update-policy \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"first_login_policy": "prompt_confirm"}' | jq '{first_login_policy}'

# 4. 验证更新成功
curl -s http://localhost:8080/api/v1/identity-providers \
  -H "Authorization: Bearer $TOKEN" | jq '.data[] | select(.alias == "test-update-policy") | {alias, first_login_policy}'
```

### 预期结果
- 步骤 2：`first_login_policy` = `"create_new"`
- 步骤 3：PUT 返回 200，`first_login_policy` = `"prompt_confirm"`
- 步骤 4：GET 确认值已持久化为 `"prompt_confirm"`

### 预期数据状态

```sql
-- 验证更新已持久化
SELECT alias, first_login_policy
FROM identity_providers
WHERE alias = 'test-update-policy';
```

**断言**:
- `first_login_policy` = `prompt_confirm`

### 清理

```bash
curl -s -X DELETE http://localhost:8080/api/v1/identity-providers/test-update-policy \
  -H "Authorization: Bearer $TOKEN" | jq .
```

---

## 场景 4：Portal UI — 创建 IdP 时 first_login_policy 下拉默认为 create_new

### 初始状态
- Portal 正在运行（`http://localhost:3000`）
- 管理员已登录 Portal

### 目的
验证在 Portal UI 中创建新 IdP 时，`first_login_policy` 下拉选择框的默认选中值为 `create_new`

### 测试操作流程
1. 登录 Portal，导航到身份提供商管理页面
2. 点击创建新 IdP 按钮，打开创建对话框
3. 观察 `first_login_policy` 下拉选择框的默认值
4. 确认下拉选项包含三个值：`create_new`、`prompt_confirm`、`auto_merge`
5. 确认 `trust_email` 开关默认为关闭状态

### 预期结果
- `first_login_policy` 下拉默认选中 `create_new`
- 下拉包含全部三个选项
- `trust_email` 开关默认关闭（OFF）
- 默认状态下**不显示**红色警告面板

---

## 场景 5：Portal UI — 选择 auto_merge 或开启 trust_email 时显示红色警告

### 初始状态
- Portal 正在运行（`http://localhost:3000`）
- 管理员已登录 Portal
- 正在 IdP 创建或编辑对话框中

### 目的
验证当用户选择 `auto_merge` 策略或开启 `trust_email` 时，Portal 显示红色安全警告面板，提醒管理员潜在风险

### 测试操作流程

**测试 A — auto_merge 警告**:
1. 在 IdP 创建/编辑对话框中，将 `first_login_policy` 从 `create_new` 切换为 `auto_merge`
2. 观察表单中是否出现红色警告面板
3. 切换回 `create_new`，观察警告是否消失
4. 切换到 `prompt_confirm`，确认无警告

**测试 B — trust_email 警告**:
1. 保持 `first_login_policy` 为 `create_new`
2. 开启 `trust_email` 开关
3. 观察表单中是否出现红色警告面板
4. 关闭 `trust_email` 开关，观察警告是否消失

**测试 C — 同时触发**:
1. 将 `first_login_policy` 设为 `auto_merge` 并开启 `trust_email`
2. 确认警告面板可见（可能显示一个或两个警告）

### 预期结果
- 选择 `auto_merge` 时显示红色警告面板，提示账户自动合并的安全风险
- 开启 `trust_email` 时显示红色警告面板，提示信任外部邮箱的安全风险
- 选择 `create_new` 或 `prompt_confirm`（且 `trust_email` 关闭）时**不显示**警告
- 警告面板的显示/隐藏响应下拉/开关的变化，无需刷新页面

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | 创建 IdP 默认 first_login_policy = create_new | ☐ | | | API 测试 |
| 2 | 显式设置 auto_merge 被接受并存储 | ☐ | | | API 测试，向后兼容 |
| 3 | 更新 first_login_policy 到 prompt_confirm | ☐ | | | API 测试 |
| 4 | Portal UI 下拉默认 create_new | ☐ | | | UI 测试 |
| 5 | auto_merge / trust_email 红色警告面板 | ☐ | | | UI 测试 |
