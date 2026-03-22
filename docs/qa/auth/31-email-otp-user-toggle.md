# 用户级 Email OTP 开关

**模块**: auth (MFA Portal)
**测试范围**: 用户级 Email OTP 启用/停用、MFA 状态响应扩展、登录双重检查（系统级 + 用户级）、Portal MFA 页面 Email OTP 卡片
**场景数**: 5
**优先级**: 高

---

## 背景说明

Auth9 原有 Email OTP 仅作为系统级（租户级）无密码登录方式存在，由 `BrandingConfig.email_otp_enabled` 控制。本次新增**用户级** `email_otp_enabled` 字段，允许用户在 MFA 自助管理页面自主开关 Email OTP。

**双重检查逻辑**：Email OTP 登录需同时满足：
1. 系统级 `BrandingConfig.email_otp_enabled = true`
2. 用户级 `users.email_otp_enabled = true`

**新增端点**（需认证，Identity Token）：
- `POST /api/v1/mfa/email-otp/enable` — 启用用户级 Email OTP
- `POST /api/v1/mfa/email-otp/disable` — 停用用户级 Email OTP

**变更端点**：
- `GET /api/v1/mfa/status` — 响应新增 `email_otp_enabled` 字段

**数据库变更**：
- `users` 表新增 `email_otp_enabled BOOLEAN NOT NULL DEFAULT FALSE`

**Portal 页面**：`/dashboard/account/mfa` 新增 Email OTP 卡片

---

## 场景 1：MFA 页面 Email OTP 卡片入口可见性

### 初始状态
- 用户已登录 Portal，进入 Dashboard
- 用户尚未启用 Email OTP（默认 `email_otp_enabled = false`）

### 目的
验证 MFA 页面中 Email OTP 卡片正确展示，且初始状态为未启用

### 测试操作流程

#### 步骤 0: 验证环境状态
```bash
# 确认 auth9-core 运行中
curl -sf http://localhost:8080/health | jq .
# 预期: {"status":"ok",...}

# 确认 MFA status 包含 email_otp_enabled 字段
TOKEN=$(.claude/skills/tools/gen-admin-token.sh)
curl -s http://localhost:8080/api/v1/mfa/status \
  -H "Authorization: Bearer $TOKEN" | python3 -m json.tool
# 预期: 响应中包含 "email_otp_enabled": false
```

#### Portal UI 流程
1. 进入「Account」页面（点击侧边栏用户头像或导航至 `/dashboard/account`）
2. 点击左侧导航中「MFA」项
3. 确认页面跳转至 `/dashboard/account/mfa`
4. 观察页面卡片列表

### 预期结果
- 页面显示四个卡片区域（从上到下）：
  - **TOTP 验证器**：原有卡片不变
  - **Email OTP**（新增）：状态标签显示「Not enabled」（灰色 Badge），可见「Enable Email OTP」按钮
  - **恢复码**：原有卡片不变
  - **通行密钥**：原有卡片不变
- Email OTP 卡片描述文字：「Receive a one-time verification code via email to sign in.」

### 预期数据状态
```sql
SELECT email_otp_enabled FROM users
WHERE email = 'admin@auth9.local';
-- 预期: email_otp_enabled = 0 (FALSE)
```

---

## 场景 2：启用用户级 Email OTP

### 初始状态
- 用户已登录，位于 `/dashboard/account/mfa`
- `email_otp_enabled = false`

### 目的
验证通过 Portal UI 和 API 均可成功启用 Email OTP

### 测试操作流程

#### Portal UI 流程
1. 在 Email OTP 卡片中点击「Enable Email OTP」按钮
2. 等待页面刷新

#### API 流程
```bash
TOKEN=$(.claude/skills/tools/gen-admin-token.sh)
curl -s -X POST http://localhost:8080/api/v1/mfa/email-otp/enable \
  -H "Authorization: Bearer $TOKEN" | python3 -m json.tool
```

### 预期结果
- Portal UI：
  - 显示成功消息「Email OTP enabled」（绿色横幅）
  - Email OTP 卡片状态标签变为「Enabled」（绿色 Badge）
  - 「Enable Email OTP」按钮替换为「Disable Email OTP」按钮（红色文字样式）
- API 响应（200）：
  ```json
  {
    "data": {
      "totp_enabled": false,
      "webauthn_enabled": false,
      "recovery_codes_remaining": 0,
      "email_otp_enabled": true
    }
  }
  ```
- MFA status 确认：
  ```bash
  curl -s http://localhost:8080/api/v1/mfa/status \
    -H "Authorization: Bearer $TOKEN" | jq '.data.email_otp_enabled'
  # 预期: true
  ```

### 预期数据状态
```sql
SELECT email_otp_enabled, updated_at FROM users
WHERE email = 'admin@auth9.local';
-- 预期: email_otp_enabled = 1, updated_at 为最近时间
```

---

## 场景 3：停用用户级 Email OTP

### 初始状态
- 用户已登录，位于 `/dashboard/account/mfa`
- `email_otp_enabled = true`（场景 2 已启用）

### 目的
验证通过 Portal UI 和 API 均可成功停用 Email OTP

### 测试操作流程

#### Portal UI 流程
1. 在 Email OTP 卡片中点击「Disable Email OTP」按钮（红色文字）
2. 等待页面刷新

#### API 流程
```bash
TOKEN=$(.claude/skills/tools/gen-admin-token.sh)
curl -s -X POST http://localhost:8080/api/v1/mfa/email-otp/disable \
  -H "Authorization: Bearer $TOKEN" | python3 -m json.tool
```

### 预期结果
- Portal UI：
  - 显示成功消息「Email OTP disabled」（绿色横幅）
  - Email OTP 卡片状态标签变为「Not enabled」（灰色 Badge）
  - 「Disable Email OTP」按钮替换为「Enable Email OTP」按钮
- API 响应（200）：
  ```json
  {
    "data": {
      "totp_enabled": false,
      "webauthn_enabled": false,
      "recovery_codes_remaining": 0,
      "email_otp_enabled": false
    }
  }
  ```

### 预期数据状态
```sql
SELECT email_otp_enabled FROM users
WHERE email = 'admin@auth9.local';
-- 预期: email_otp_enabled = 0
```

---

## 场景 4：登录双重检查 — 用户级关闭时 Email OTP 登录失败

### 初始状态
- 系统品牌设置 `email_otp_enabled = true`（系统级已开启）
- 用户 `email_otp_enabled = false`（用户级未开启）
- 邮件服务已配置

### 目的
验证 Email OTP 登录在用户级关闭时被拒绝，但发送验证码仍返回统一响应（防枚举）

### 测试操作流程

#### 步骤 0: 确认环境状态
```bash
# 确认系统级 Email OTP 已开启
TOKEN={admin_tenant_access_token}
curl -s http://localhost:8080/api/v1/system/branding \
  -H "Authorization: Bearer $TOKEN" | jq '.data.email_otp_enabled'
# 预期: true（如为 false，需先启用）

# 确认用户级 Email OTP 已关闭
```
```sql
SELECT email, email_otp_enabled FROM users
WHERE email = 'admin@auth9.local';
-- 预期: email_otp_enabled = 0
-- 若为 1，执行: UPDATE users SET email_otp_enabled = false WHERE email = 'admin@auth9.local';
```

#### 4a: 发送验证码（防枚举 — 不泄露用户级状态）
```bash
curl -s -X POST http://localhost:8080/api/v1/auth/email-otp/send \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@auth9.local"}' | python3 -m json.tool
```

#### 4b: 验证码验证（用户级关闭，应被拒绝）
```bash
# 从邮件中获取 OTP code，或从 Redis/日志中获取
curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST http://localhost:8080/api/v1/auth/email-otp/verify \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@auth9.local", "code": "{otp_code}"}'
```

#### 4c: 启用用户级后重试
```bash
# 先启用用户级
IDENTITY_TOKEN=$(.claude/skills/tools/gen-admin-token.sh)
curl -s -X POST http://localhost:8080/api/v1/mfa/email-otp/enable \
  -H "Authorization: Bearer $IDENTITY_TOKEN" > /dev/null

# 重新发送验证码
curl -s -X POST http://localhost:8080/api/v1/auth/email-otp/send \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@auth9.local"}' | python3 -m json.tool

# 使用新验证码验证
curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST http://localhost:8080/api/v1/auth/email-otp/verify \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@auth9.local", "code": "{new_otp_code}"}'
```

### 预期结果
- **4a**: 返回 200，响应为 `"If this email is registered, a verification code has been sent."`（与正常用户完全相同，不泄露用户级开关状态）
- **4b**: 返回 401，`"Authentication failed."`（验证码正确但用户级未开启）
- **4c**: 返回 200，包含 `access_token`（用户级开启后可正常登录）

---

## 场景 5：未认证访问 Email OTP 开关端点

### 初始状态
- 无有效 Token 或使用过期 Token

### 目的
验证 Email OTP 开关端点需要有效 Identity Token 认证

### 测试操作流程
```bash
# 5a: 无 Token
curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST http://localhost:8080/api/v1/mfa/email-otp/enable

# 5b: 无效 Token
curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST http://localhost:8080/api/v1/mfa/email-otp/enable \
  -H "Authorization: Bearer invalid_token_here"

# 5c: disable 端点同理
curl -s -w "\nHTTP_STATUS:%{http_code}" -X POST http://localhost:8080/api/v1/mfa/email-otp/disable
```

### 预期结果
- **5a**: 返回 401（缺少 Authorization header）
- **5b**: 返回 401（Token 无效）
- **5c**: 返回 401（缺少 Authorization header）

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | MFA 页面 Email OTP 卡片入口可见性 | ☐ | | | |
| 2 | 启用用户级 Email OTP | ☐ | | | |
| 3 | 停用用户级 Email OTP | ☐ | | | |
| 4 | 登录双重检查 — 用户级关闭时登录失败 | ☐ | | | |
| 5 | 未认证访问 Email OTP 开关端点 | ☐ | | | |
