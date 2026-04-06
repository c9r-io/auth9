# 身份提供商 - 租户级企业 SSO 连接器管理测试

**模块**: 身份提供商
**测试范围**: 租户级企业连接器创建、更新、删除、测试与域名唯一约束
**场景数**: 5
**优先级**: 高

---

## 背景说明

新增租户级企业 SSO 管理能力，管理员可在租户维度配置 SAML/OIDC/LDAP 连接器。

> LDAP 连接器专属测试参见 `identity-provider/04-enterprise-ldap-connectors.md` 和 `identity-provider/05-ldap-group-role-mappings.md`

端点：
- `GET /api/v1/tenants/{tenant_id}/sso/connectors`
- `POST /api/v1/tenants/{tenant_id}/sso/connectors`
- `PUT /api/v1/tenants/{tenant_id}/sso/connectors/{connector_id}`
- `DELETE /api/v1/tenants/{tenant_id}/sso/connectors/{connector_id}`
- `POST /api/v1/tenants/{tenant_id}/sso/connectors/{connector_id}/test`

**SCIM Provisioning**（连接器创建后可配置）：
- `POST /api/v1/tenants/{tid}/sso/connectors/{cid}/scim/tokens` — 生成 SCIM Bearer Token
- `GET /api/v1/tenants/{tid}/sso/connectors/{cid}/scim/tokens` — 列出 SCIM Token
- `DELETE /api/v1/tenants/{tid}/sso/connectors/{cid}/scim/tokens/{id}` — 吊销 SCIM Token

> 详细 SCIM 测试参见 `docs/qa/provisioning/01-scim-token-management.md`

---

## 场景 1：创建 SAML 连接器成功

### 初始状态
- 已存在租户 `{tenant_id}`
- 当前租户尚未使用 alias `{connector_alias}`

### 目的
验证 SAML 连接器创建与域名绑定成功落库

### 测试操作流程
1. 在租户详情页进入「Enterprise SSO」
2. 确认「Provider Type」控件使用项目统一 Selector 组件，而非浏览器原生 `<select>`
3. 点击「Create Connector」并填写：
   - Alias：`{connector_alias}`
   - Provider Type：`saml`
   - Domains：`{corp_domain}`
   - SAML Entity ID：`{entity_id}`
   - SAML SSO URL：`{sso_url}`
   - SAML 签名证书：`{certificate}`
4. 点击「Create Connector」提交
5. 可选：在 `http://localhost:3002/dashboard` 的「Enterprise SSO QA Panel」执行同等操作（Create SAML Connector）

### 预期结果
- 页面提示创建成功
- 连接器列表出现新记录
- 展示 provider type 为 `SAML`，域名包含 `{corp_domain}`
- 「Provider Type」切换为 `saml` 时，仅显示 SAML 字段组，不显示 OIDC 字段组

### 预期数据状态
```sql
SELECT id, tenant_id, alias, provider_type, enabled, provider_alias
FROM enterprise_sso_connectors
WHERE tenant_id = '{tenant_id}' AND alias = '{connector_alias}';
-- 预期: 返回 1 行，provider_type='saml'

SELECT domain, connector_id
FROM enterprise_sso_domains
WHERE domain = '{corp_domain}';
-- 预期: 返回 1 行，connector_id 对应上方连接器
```

---

## 场景 2：创建 OIDC 连接器成功

### 初始状态
- 已存在租户 `{tenant_id}`

### 目的
验证 OIDC 连接器字段校验与创建

### 测试操作流程
1. 在租户详情页进入「Enterprise SSO」
2. 将「Provider Type」切换为 `oidc`
3. 确认页面显示 `OIDC Client ID`、`OIDC Client Secret`、`OIDC Authorization URL`、`OIDC Token URL`
4. 调用 demo 代理创建接口（推荐）：
```bash
curl -X POST 'http://localhost:3002/demo/enterprise/connectors' \
  -H 'Content-Type: application/json' \
  -d '{
    "tenantId":"{tenant_id}",
    "alias":"{oidc_alias}",
    "provider_type":"oidc",
    "enabled":true,
    "priority":100,
    "domains":["{oidc_domain}"],
    "config":{
      "clientId":"{client_id}",
      "clientSecret":"{client_secret}",
      "authorizationUrl":"{authorization_url}",
      "tokenUrl":"{token_url}"
    }
  }'
```
5. 或直连 core 接口：
```bash
curl -X POST 'http://localhost:8080/api/v1/tenants/{tenant_id}/sso/connectors' \
  -H 'Authorization: Bearer {tenant_access_token}' \
  -H 'Content-Type: application/json' \
  -d '{
    "alias":"{oidc_alias}",
    "provider_type":"oidc",
    "enabled":true,
    "priority":100,
    "domains":["{oidc_domain}"],
    "config":{
      "clientId":"{client_id}",
      "clientSecret":"{client_secret}",
      "authorizationUrl":"{authorization_url}",
      "tokenUrl":"{token_url}"
    }
  }'
```
6. 查询连接器列表确认返回

### 预期结果
- HTTP 状态码 `200`
- 返回的 `data.alias` 为 `{oidc_alias}`
- 可在列表中看到 OIDC 连接器
- Portal 表单切换到 `oidc` 后，不再展示 `SAML Entity ID` / `SAML SSO URL` / `Certificate`

### 预期数据状态
```sql
SELECT alias, provider_type, JSON_EXTRACT(config, '$.authorizationUrl') AS authorization_url
FROM enterprise_sso_connectors
WHERE tenant_id = '{tenant_id}' AND alias = '{oidc_alias}';
-- 预期: provider_type='oidc' 且 authorization_url 非空
```

---

## 场景 3：域名冲突时创建失败

### 初始状态
- 域名 `{corp_domain}` 已被租户 A 的连接器占用

### 目的
验证 `enterprise_sso_domains.domain` 全局唯一约束

### 测试操作流程
1. 在租户 B 再次创建连接器（可通过 `POST /demo/enterprise/connectors`），domains 包含 `{corp_domain}`
2. 观察接口响应

### 预期结果
- HTTP 状态码 `409`
- 错误提示为域名/连接器重复冲突

### 预期数据状态
```sql
SELECT tenant_id, connector_id, domain
FROM enterprise_sso_domains
WHERE domain = '{corp_domain}';
-- 预期: 仅保留原有 1 条绑定，不新增第二条
```

---

## 场景 4：Portal 内联编辑连接器域名

### 初始状态
- 已存在连接器 `{connector_id}`，domains 包含 `corp.example.com`
- 用户已登录 Portal 并进入「Tenants → {tenant} → Enterprise SSO」页面

### 目的
验证 Portal UI 内联域名编辑功能的完整流程，包括正常编辑、输入校验、自动去重与页面刷新

### 测试操作流程

#### 4a. 正常编辑域名（Happy Path）
1. 在连接器卡片的域名区域，点击域名旁的铅笔图标（Pencil Icon）
2. 确认域名区域切换为内联编辑模式，显示输入框和「Save」「Cancel」按钮
3. 将输入框内容修改为 `new-corp.example.com`
4. 点击「Save」
5. 确认页面自动刷新（React Router revalidation），连接器卡片域名更新为 `new-corp.example.com`

#### 4b. 校验：空域名
1. 点击铅笔图标进入编辑模式
2. 清空输入框内容
3. 点击「Save」
4. 确认页面显示校验错误提示（i18n key: `domainsValidationError`）

#### 4c. 校验：无效格式（不含点号）
1. 点击铅笔图标进入编辑模式
2. 输入 `invalidformat`（不含 `.`）
3. 点击「Save」
4. 确认页面显示校验错误提示

#### 4d. 自动去重
1. 点击铅笔图标进入编辑模式
2. 输入 `sso.example.com, sso.example.com, api.example.com`（包含重复域名）
3. 点击「Save」
4. 确认保存成功，连接器卡片域名仅显示 `sso.example.com` 和 `api.example.com`（去重后 2 个）

#### 4e. 取消编辑
1. 点击铅笔图标进入编辑模式
2. 修改输入框内容
3. 点击「Cancel」
4. 确认域名恢复为编辑前的值，不触发 API 请求

#### 4f. API 验证
直连 core 接口验证域名更新：
```bash
curl -X PUT 'http://localhost:8080/api/v1/tenants/{tenant_id}/sso/connectors/{connector_id}' \
  -H 'Authorization: Bearer {tenant_access_token}' \
  -H 'Content-Type: application/json' \
  -d '{"domains": ["new-corp.example.com", "sso.example.com"]}'
```
- 预期 HTTP 状态码 `200`
- 返回的 `data.domains` 包含 `new-corp.example.com` 和 `sso.example.com`

### 预期结果
- 铅笔图标点击后进入内联编辑模式，显示逗号分隔的域名输入框
- 保存成功后页面自动刷新（无需手动 F5），连接器卡片域名即时更新
- 空域名或格式无效时显示校验错误，不发送 API 请求
- 重复域名自动去重后保存
- 取消编辑不触发 API 调用
- i18n 支持：错误提示在 en-US / zh-CN / ja 三种语言下均正确显示

### 预期数据状态
```sql
-- 4a 正常编辑后
SELECT domain FROM enterprise_sso_domains WHERE connector_id = '{connector_id}';
-- 预期: 1 行，domain = 'new-corp.example.com'

-- 4d 去重后
SELECT domain FROM enterprise_sso_domains WHERE connector_id = '{connector_id}' ORDER BY domain;
-- 预期: 2 行，分别为 'api.example.com' 和 'sso.example.com'

-- 4b/4c 校验失败后
SELECT domain FROM enterprise_sso_domains WHERE connector_id = '{connector_id}';
-- 预期: 域名未变更，保持编辑前的值
```

### Troubleshooting

| 现象 | 原因 | 解决方法 |
|------|------|----------|
| 铅笔图标不可见 | 前端组件未渲染 Pencil2Icon | 检查连接器卡片 JSX 中 `editDomains` 相关代码是否正确引入 |
| 保存后页面未自动刷新 | React Router revalidation 未触发 | 检查 Form action 返回后是否调用 `revalidator` 或返回 redirect |
| UI 显示 "Identity provider updated" 但 `enterprise_sso_domains` 未变 | **操作了错误的页面**：Identity Providers 页面（`/dashboard/settings/identity-providers`）只更新全局 IdP 配置 | 在 **Tenant SSO Connectors** 页面（`/dashboard/tenants/{tenantId}/sso`）操作，成功消息应为 "Connector updated" |
| 去重后仍显示重复域名 | 前端未在提交前执行 `[...new Set(...)]` | 检查 `intent=update_domains` action handler 中的去重逻辑 |

> **注意**：系统有两个类似但不同的 SSO 管理页面：
> - **Identity Providers**（`Settings → Identity Providers`）：管理全局 IdP 配置，不涉及 `enterprise_sso_connectors` 表
> - **Tenant SSO Connectors**（`Tenants → {tenant} → Enterprise SSO`）：管理租户级别的 SSO 连接器，更新 `enterprise_sso_connectors` 表和 Auth9 内置 OIDC 引擎

---

## 场景 5：测试连接与删除连接器

### 初始状态
- 已存在连接器 `{connector_id}`

### 目的
验证「Test」与「Delete」操作可正常执行并清理数据

### 测试操作流程
1. 在连接器卡片点击「Test」或调用 `POST /demo/enterprise/connectors/{connector_id}/test`
2. 记录返回消息（成功或失败原因）
3. 点击「Delete」或调用 `DELETE /demo/enterprise/connectors/{connector_id}?tenantId={tenant_id}` 删除同一连接器
4. 重新请求连接器列表

### 预期结果
- 「Test」返回结构化结果：`ok` + `message`
- 删除后列表中不再显示该连接器

### 预期数据状态
```sql
SELECT * FROM enterprise_sso_connectors WHERE id = '{connector_id}';
-- 预期: 0 行

SELECT * FROM enterprise_sso_domains WHERE connector_id = '{connector_id}';
-- 预期: 0 行
```

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | 创建 SAML 连接器成功 | ☐ | | | |
| 2 | 创建 OIDC 连接器成功 | ☐ | | | |
| 3 | 域名冲突时创建失败 | ☐ | | | |
| 4 | Portal 内联编辑连接器域名 | ☐ | | | |
| 5 | 测试连接与删除连接器 | ☐ | | | |
