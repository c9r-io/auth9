# 风险策略 (Risk Policy) QA 测试

**模块**: settings / security
**功能**: 风险评分引擎与自动响应策略（Risk Scoring Engine & Auto Response）
**FR**: FR-004
**场景数**: 5
**优先级**: 高

---

## 背景说明

本功能为增强型异常检测，包含：

1. **风险策略 API**: 每个租户可配置 MFA 触发阈值和自动阻断阈值
2. **风险评分**: 每次登录事件计算 0-100 的 `risk_score`
3. **GeoIP 信息**: 登录事件记录 `latitude`、`longitude`、`country_code`
4. **用户登录画像**: `user_login_profiles` 表跟踪行为基线
5. **高风险告警**: `security_alerts` 新增 `high_risk_login` 告警类型

端点：
- `GET /api/v1/security/risk-policy` — 获取当前租户风险策略
- `PUT /api/v1/security/risk-policy` — 更新当前租户风险策略

---

## 数据库表结构参考

### login_events 表（新增字段）
| 字段 | 类型 | 说明 |
|------|------|------|
| risk_score | INT | 风险评分 (0-100) |
| latitude | DOUBLE | GeoIP 纬度 |
| longitude | DOUBLE | GeoIP 经度 |
| country_code | VARCHAR(2) | GeoIP 国家代码 |

### user_login_profiles 表
| 字段 | 类型 | 说明 |
|------|------|------|
| id | CHAR(36) | UUID 主键 |
| user_id | CHAR(36) | 用户 ID |
| tenant_id | CHAR(36) | 租户 ID |
| known_ips | JSON | 历史已知 IP 列表 |
| known_countries | JSON | 历史已知国家列表 |
| known_user_agents | JSON | 历史已知 User-Agent 列表 |
| updated_at | DATETIME | 最后更新时间 |

### security_alerts 表（新增告警类型）
| alert_type | 说明 |
|------------|------|
| high_risk_login | 高风险登录告警（risk_score 超过阈值） |

---

## 场景 1：获取默认风险策略（未配置自定义策略）

### 初始状态
- 租户已创建，未修改过风险策略
- 已获取有效 `$TOKEN`

### 目的
验证未配置自定义策略时，API 返回系统默认值

### 测试操作流程

1. 请求获取当前租户的风险策略：

```bash
curl -sf http://localhost:8080/api/v1/security/risk-policy \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### 预期结果

- HTTP 状态码：200
- 响应包含默认阈值配置：
  - `mfa_threshold`: 默认值（如 50）
  - `block_threshold`: 默认值（如 80）
- 响应包含 `tenant_id` 字段，与当前租户匹配

### 预期数据状态
```sql
-- 可能没有自定义记录（返回代码内默认值），或有一条默认记录
SELECT * FROM risk_policies WHERE tenant_id = '{tenant_id}';
-- 预期: 0 或 1 条记录
```

---

## 场景 2：更新风险策略（自定义阈值）

### 初始状态
- 租户已创建，使用默认风险策略
- 已获取有效 `$TOKEN`

### 目的
验证管理员可以为租户配置自定义 MFA 和阻断阈值

### 测试操作流程

1. 更新风险策略阈值：

```bash
curl -sf -X PUT http://localhost:8080/api/v1/security/risk-policy \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "mfa_threshold": 40,
    "block_threshold": 75
  }' | jq .
```

### 预期结果

- HTTP 状态码：200
- 响应体中 `mfa_threshold` = 40
- 响应体中 `block_threshold` = 75

### 预期数据状态
```sql
SELECT mfa_threshold, block_threshold FROM risk_policies WHERE tenant_id = '{tenant_id}';
-- 预期: mfa_threshold = 40, block_threshold = 75
```

---

## 场景 3：获取已更新的风险策略（验证持久化）

### 初始状态
- 场景 2 已执行，风险策略已更新为自定义阈值
- 已获取有效 `$TOKEN`

### 目的
验证更新后的策略正确持久化，再次读取返回更新后的值

### 测试操作流程

1. 重新获取风险策略：

```bash
curl -sf http://localhost:8080/api/v1/security/risk-policy \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### 预期结果

- HTTP 状态码：200
- `mfa_threshold` = 40（场景 2 设置的值）
- `block_threshold` = 75（场景 2 设置的值）
- 值与场景 2 写入的完全一致，确认持久化成功

---

## 场景 4：部分更新（仅修改 mfa_threshold）

### 初始状态
- 当前风险策略为 `mfa_threshold = 40, block_threshold = 75`
- 已获取有效 `$TOKEN`

### 目的
验证部分更新时，未传递的字段保持原值不被覆盖

### 测试操作流程

1. 仅更新 `mfa_threshold`：

```bash
curl -sf -X PUT http://localhost:8080/api/v1/security/risk-policy \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "mfa_threshold": 60
  }' | jq .
```

2. 验证完整策略：

```bash
curl -sf http://localhost:8080/api/v1/security/risk-policy \
  -H "Authorization: Bearer $TOKEN" | jq .
```

### 预期结果

- 步骤 1 返回 HTTP 200
- 步骤 2 返回 `mfa_threshold` = 60（已更新）
- 步骤 2 返回 `block_threshold` = 75（保持不变）

### 预期数据状态
```sql
SELECT mfa_threshold, block_threshold FROM risk_policies WHERE tenant_id = '{tenant_id}';
-- 预期: mfa_threshold = 60, block_threshold = 75
```

---

## 场景 5：未认证访问（无 Token）

### 初始状态
- 不携带 Authorization 头

### 目的
验证风险策略端点受认证保护，未认证请求被拒绝

### 测试操作流程

1. 无 Token 请求 GET 端点：

```bash
curl -sf http://localhost:8080/api/v1/security/risk-policy \
  -w "\n%{http_code}"
```

2. 无 Token 请求 PUT 端点：

```bash
curl -sf -X PUT http://localhost:8080/api/v1/security/risk-policy \
  -H "Content-Type: application/json" \
  -d '{"mfa_threshold": 50, "block_threshold": 80}' \
  -w "\n%{http_code}"
```

### 预期结果

- 两个请求均返回 HTTP 401
- 响应体包含认证错误信息（如 `"Unauthorized"` 或 `"Missing authorization header"`）
- 不返回任何策略数据

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | 获取默认风险策略 | ☐ | | | |
| 2 | 更新风险策略（自定义阈值） | ☐ | | | |
| 3 | 获取已更新的风险策略（持久化验证） | ☐ | | | |
| 4 | 部分更新（仅 mfa_threshold） | ☐ | | | |
| 5 | 未认证访问（无 Token） | ☐ | | | |
