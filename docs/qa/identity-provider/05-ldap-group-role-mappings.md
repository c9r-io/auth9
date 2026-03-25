# 身份提供商 - LDAP 组角色映射管理

**模块**: 身份提供商
**测试范围**: LDAP 组到 Auth9 角色映射的创建、列表、删除、唯一约束
**场景数**: 4
**优先级**: 中

---

## 背景说明

LDAP 组角色映射允许管理员将 LDAP 目录中的组 DN 映射到 Auth9 角色，用户每次通过 LDAP 登录时自动同步角色。

端点：
- `GET /api/v1/tenants/{tenant_id}/sso/connectors/{connector_id}/ldap-group-mappings` — 列出映射
- `POST /api/v1/tenants/{tenant_id}/sso/connectors/{connector_id}/ldap-group-mappings` — 创建映射
- `DELETE /api/v1/tenants/{tenant_id}/sso/connectors/{connector_id}/ldap-group-mappings/{mapping_id}` — 删除映射
- `POST /api/v1/tenants/{tenant_id}/sso/connectors/{connector_id}/ldap-search-users` — 搜索 LDAP 用户

**前置依赖**: 需先创建 LDAP 连接器（参见 `identity-provider/04-enterprise-ldap-connectors.md` 场景 1）

## 数据库表结构参考

```sql
CREATE TABLE ldap_group_role_mappings (
    id CHAR(36) PRIMARY KEY,
    tenant_id CHAR(36) NOT NULL,
    connector_id CHAR(36) NOT NULL,
    ldap_group_dn VARCHAR(512) NOT NULL,
    ldap_group_display_name VARCHAR(255),
    role_id CHAR(36) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE INDEX idx_ldap_grm_connector_group_role (connector_id, ldap_group_dn, role_id)
);
```

---

## 步骤 0（Gate Check）

```bash
# 1. 获取管理员 Token
TOKEN=$(.claude/skills/tools/gen-admin-token.sh)

# 2. 获取租户 ID
TENANT_ID=$(curl -sf http://localhost:8080/api/v1/tenants \
  -H "Authorization: Bearer $TOKEN" | jq -r '.data[0].id')

# 3. 获取或创建 LDAP 连接器
CONNECTOR_ID=$(curl -sf "http://localhost:8080/api/v1/tenants/${TENANT_ID}/sso/connectors" \
  -H "Authorization: Bearer $TOKEN" | jq -r '.data[] | select(.provider_type=="ldap") | .id' | head -1)

# 如果没有 LDAP 连接器，先创建一个（见 04 文档场景 1）

# 4. 获取一个角色 ID（用于映射）
ROLE_ID=$(curl -sf "http://localhost:8080/api/v1/tenants/${TENANT_ID}/roles" \
  -H "Authorization: Bearer $TOKEN" | jq -r '.data[0].id // empty')
echo "Tenant=$TENANT_ID Connector=$CONNECTOR_ID Role=$ROLE_ID"
```

---

## 场景 1：创建 LDAP 组角色映射成功

### 初始状态
- 已存在 LDAP 连接器 `{connector_id}`
- 已存在角色 `{role_id}`
- 当前无映射记录

### 目的
验证创建 LDAP 组到角色映射落库成功

### 测试操作流程

**方式一：Portal UI**
1. 进入「Tenants → {tenant} → Enterprise SSO」
2. 在 LDAP 连接器卡片点击「Group Mappings」
3. 填写 LDAP Group DN: `cn=developers,ou=groups,dc=corp,dc=example,dc=com`
4. 填写 Display Name: `Developers`
5. 填写 Role ID: `{role_id}`
6. 点击「Add Mapping」

**方式二：API**
```bash
curl -X POST "http://localhost:8080/api/v1/tenants/${TENANT_ID}/sso/connectors/${CONNECTOR_ID}/ldap-group-mappings" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "ldap_group_dn": "cn=developers,ou=groups,dc=corp,dc=example,dc=com",
    "ldap_group_display_name": "Developers",
    "role_id": "'"$ROLE_ID"'"
  }'
```

### 预期结果
- HTTP 200
- 返回 `data.ldap_group_dn` 和 `data.role_id` 匹配
- Portal 映射列表显示新增记录

### 预期数据状态
```sql
SELECT id, ldap_group_dn, ldap_group_display_name, role_id
FROM ldap_group_role_mappings
WHERE connector_id = '{connector_id}';
-- 预期: 1 行，ldap_group_dn='cn=developers,...', role_id='{role_id}'
```

---

## 场景 2：重复映射被拒绝（唯一约束）

### 初始状态
- 已存在映射: connector_id + ldap_group_dn + role_id 组合

### 目的
验证 `UNIQUE INDEX idx_ldap_grm_connector_group_role` 约束生效

### 测试操作流程
```bash
# 再次创建相同的映射
curl -X POST "http://localhost:8080/api/v1/tenants/${TENANT_ID}/sso/connectors/${CONNECTOR_ID}/ldap-group-mappings" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "ldap_group_dn": "cn=developers,ou=groups,dc=corp,dc=example,dc=com",
    "role_id": "'"$ROLE_ID"'"
  }'
```

### 预期结果
- HTTP 409 Conflict
- 错误消息提示重复映射

### 预期数据状态
```sql
SELECT COUNT(*) AS cnt FROM ldap_group_role_mappings
WHERE connector_id = '{connector_id}'
  AND ldap_group_dn = 'cn=developers,ou=groups,dc=corp,dc=example,dc=com'
  AND role_id = '{role_id}';
-- 预期: cnt=1（不增加）
```

---

## 场景 3：非 LDAP 连接器创建映射被拒绝

### 初始状态
- 已存在 SAML 或 OIDC 连接器 `{saml_connector_id}`

### 目的
验证只有 provider_type=ldap 的连接器才能创建组角色映射

### 测试操作流程
```bash
# 对 SAML 连接器尝试创建 LDAP 映射
curl -X POST "http://localhost:8080/api/v1/tenants/${TENANT_ID}/sso/connectors/${SAML_CONNECTOR_ID}/ldap-group-mappings" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "ldap_group_dn": "cn=test,dc=example,dc=com",
    "role_id": "'"$ROLE_ID"'"
  }'
```

### 预期结果
- HTTP 400 或 422
- 错误消息包含 "only supported for LDAP connectors"

---

## 场景 4：删除组角色映射

### 初始状态
- 已存在映射 `{mapping_id}`

### 目的
验证映射删除成功、UI 实时更新

### 测试操作流程

**方式一：Portal UI**
1. 进入 LDAP 连接器的「Group Mappings」页面
2. 点击映射行的删除图标（垃圾桶）

**方式二：API**
```bash
curl -X DELETE "http://localhost:8080/api/v1/tenants/${TENANT_ID}/sso/connectors/${CONNECTOR_ID}/ldap-group-mappings/${MAPPING_ID}" \
  -H "Authorization: Bearer $TOKEN"
```

### 预期结果
- HTTP 200
- Portal 映射列表中不再显示该记录

### 预期数据状态
```sql
SELECT * FROM ldap_group_role_mappings WHERE id = '{mapping_id}';
-- 预期: 0 行
```

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | 创建 LDAP 组角色映射成功 | ☐ | | | |
| 2 | 重复映射被拒绝 | ☐ | | | |
| 3 | 非 LDAP 连接器创建映射被拒绝 | ☐ | | | |
| 4 | 删除组角色映射 | ☐ | | | |
