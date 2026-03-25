# Adaptive MFA - 风险驱动多因素认证策略与可信设备管理

**模块**: 认证流程 / Adaptive MFA
**测试范围**: Adaptive MFA 策略 API（GET/PUT）、可信设备列表 API、持久化验证、未授权访问拒绝
**场景数**: 5
**优先级**: 高

---

## 背景说明

Adaptive MFA（风险驱动多因素认证）允许租户管理员配置 MFA 策略模式（always / adaptive / disabled / optional_enroll），根据风险评分决定是否触发 MFA 挑战。可信设备管理允许用户查看和撤销已信任的设备。

端点：

- `GET /api/v1/mfa/adaptive-policy` — 获取当前租户的 Adaptive MFA 策略（需认证 + SecurityAlertRead 权限）
- `PUT /api/v1/mfa/adaptive-policy` — 更新当前租户的 Adaptive MFA 策略（需认证 + SecurityAlertResolve 权限）
- `GET /api/v1/mfa/trusted-devices` — 获取当前用户的可信设备列表（需认证）
- `DELETE /api/v1/mfa/trusted-devices/{id}` — 撤销指定可信设备（需认证）
- `DELETE /api/v1/mfa/trusted-devices` — 撤销所有可信设备（需认证）

默认策略（首次查询时无数据库记录，返回内存默认值）：

```json
{
  "mode": "always",
  "risk_threshold": 40,
  "always_require_for_admins": true,
  "trust_device_days": 30,
  "step_up_operations": ["change_password", "modify_security_settings"]
}
```

MFA 模式枚举值：`disabled` / `always` / `adaptive` / `optional_enroll`

---

## 场景 1：获取默认 Adaptive MFA 策略

### 初始状态
- 数据库中该租户无 `adaptive_mfa_policies` 记录
- 已获取有效的 Tenant Access Token（具备 SecurityAlertRead 权限）

### 目的
验证未配置策略时，API 返回内存默认策略（mode=always）

### 测试操作流程
1. 使用有效 Token 调用 GET 端点：
   ```bash
   curl -s -X GET http://localhost:8080/api/v1/mfa/adaptive-policy \
     -H "Authorization: Bearer ${ACCESS_TOKEN}" \
     -H "Content-Type: application/json" | jq .
   ```

### 预期结果
- HTTP 状态码 `200`
- 响应 `data.mode` 为 `"always"`
- 响应 `data.risk_threshold` 为 `40`
- 响应 `data.always_require_for_admins` 为 `true`
- 响应 `data.trust_device_days` 为 `30`
- 响应 `data.step_up_operations` 包含 `"change_password"` 和 `"modify_security_settings"`
- 响应 `data.tenant_id` 为当前租户 ID

---

## 场景 2：更新 Adaptive MFA 策略为 adaptive 模式

### 初始状态
- 已获取有效的 Tenant Access Token（具备 SecurityAlertResolve 权限）

### 目的
验证 PUT 端点可正确更新策略配置为 adaptive 模式并自定义阈值

### 测试操作流程
1. 调用 PUT 端点更新策略：
   ```bash
   curl -s -X PUT http://localhost:8080/api/v1/mfa/adaptive-policy \
     -H "Authorization: Bearer ${ACCESS_TOKEN}" \
     -H "Content-Type: application/json" \
     -d '{
       "mode": "adaptive",
       "risk_threshold": 60,
       "always_require_for_admins": false,
       "trust_device_days": 14,
       "step_up_operations": ["change_password", "delete_account"]
     }' | jq .
   ```

### 预期结果
- HTTP 状态码 `200`
- 响应 `data.mode` 为 `"adaptive"`
- 响应 `data.risk_threshold` 为 `60`
- 响应 `data.always_require_for_admins` 为 `false`
- 响应 `data.trust_device_days` 为 `14`
- 响应 `data.step_up_operations` 为 `["change_password", "delete_account"]`

### 预期数据状态
```sql
SELECT mode, risk_threshold, always_require_for_admins, trust_device_days
FROM adaptive_mfa_policies
WHERE tenant_id = '{tenant_id}';
-- 预期: mode='adaptive', risk_threshold=60, always_require_for_admins=0, trust_device_days=14
```

---

## 场景 3：查询可信设备列表（初始为空）

### 初始状态
- 用户从未通过 MFA 挑战信任过设备
- 已获取有效的 Identity Token

### 目的
验证新用户查询可信设备列表时返回空数组

### 测试操作流程
1. 调用可信设备列表端点：
   ```bash
   curl -s -X GET http://localhost:8080/api/v1/mfa/trusted-devices \
     -H "Authorization: Bearer ${IDENTITY_TOKEN}" \
     -H "Content-Type: application/json" | jq .
   ```

### 预期结果
- HTTP 状态码 `200`
- 响应 `data` 为空数组 `[]`

---

## 场景 4：更新后再次获取策略（持久化验证）

### 初始状态
- 已执行场景 2 的策略更新操作
- 已获取有效的 Tenant Access Token（具备 SecurityAlertRead 权限）

### 目的
验证策略更新已持久化到数据库，再次 GET 时返回更新后的值

### 测试操作流程
1. 再次调用 GET 端点：
   ```bash
   curl -s -X GET http://localhost:8080/api/v1/mfa/adaptive-policy \
     -H "Authorization: Bearer ${ACCESS_TOKEN}" \
     -H "Content-Type: application/json" | jq .
   ```
2. 对比返回值与场景 2 中 PUT 提交的值

### 预期结果
- HTTP 状态码 `200`
- 响应 `data.mode` 为 `"adaptive"`（与场景 2 PUT 时一致）
- 响应 `data.risk_threshold` 为 `60`
- 响应 `data.always_require_for_admins` 为 `false`
- 响应 `data.trust_device_days` 为 `14`
- 响应 `data.step_up_operations` 为 `["change_password", "delete_account"]`

---

## 场景 5：未授权访问策略端点

### 初始状态
- 无有效 Token（或使用过期 / 无效 Token）

### 目的
验证策略端点的认证保护，无有效凭证时拒绝访问

### 测试操作流程
1. 不携带 Token 调用 GET 端点：
   ```bash
   curl -s -X GET http://localhost:8080/api/v1/mfa/adaptive-policy \
     -H "Content-Type: application/json" | jq .
   ```
2. 携带无效 Token 调用 PUT 端点：
   ```bash
   curl -s -X PUT http://localhost:8080/api/v1/mfa/adaptive-policy \
     -H "Authorization: Bearer invalid_token_abc123" \
     -H "Content-Type: application/json" \
     -d '{"mode": "disabled"}' | jq .
   ```

### 预期结果
- 步骤 1：HTTP 状态码 `401`，响应包含未授权错误信息
- 步骤 2：HTTP 状态码 `401`，响应包含 Token 无效错误信息
- 策略配置不应被修改

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | 获取默认 Adaptive MFA 策略 | ☐ | | | |
| 2 | 更新策略为 adaptive 模式 | ☐ | | | |
| 3 | 查询可信设备列表（初始为空） | ☐ | | | |
| 4 | 更新后再次获取策略（持久化验证） | ☐ | | | |
| 5 | 未授权访问策略端点 | ☐ | | | |
