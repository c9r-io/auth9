# Phase 3 FR2: 密码验证、密码策略与重置流程

**类型**: 核心能力替换
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core (Backend), auth9-portal (Frontend), tests, docs/qa/auth
**前置依赖**:
- `keycloak_phase3_fr1_local_credential_store.md`
- `keycloak_phase2_fr2_hosted_login_api.md`
- `keycloak_phase2_fr3_browser_session_and_refresh_chain.md`
**被依赖**:
- `keycloak_phase3_fr3_email_verification_and_required_actions.md`
- `keycloak_phase3_fr5_token_issuance_and_auth_server_core.md`

---

## 背景

Auth9 当前已有密码修改、忘记密码、密码策略等接口与页面，但底层仍与 Keycloak password policy / credentials 能力纠缠。要真正实现第一方账号本地托管，密码验证与重置流程必须完全迁到 Auth9 自己控制。

---

## 期望行为

### R1: Password verify / change / admin reset 由 Auth9 自管

要求：

- 登录密码验证不再依赖 Keycloak credentials API
- 用户修改密码由 Auth9 自己校验当前密码和新密码
- 管理员重置密码由 Auth9 自己完成

### R2: 忘记密码 token 自管

要求：

- 忘记密码 token 的签发、存储、消费和失效检查全部由 Auth9 自己完成
- 支持 replay 防护与过期校验
- 保持现有重置页面与邮件入口语义

### R3: 密码策略由 Auth9 自己存储和执行

要求：

- 不再依赖 Keycloak realm password policy
- Password policy 由 Auth9 自己存储、读取、校验
- 审计日志保留现有事件语义

### R4: 前后端流程在 Auth9 域名下闭环

要求：

- `/forgot-password`、`/reset-password`、`/users/me/password` 等前后端流程都由 Auth9 承担
- 前端不再依赖 Keycloak required action 或 Keycloak 页面

### R5: 回归测试

要求：

- password hash / verify
- reset token replay / expiry
- admin reset password
- password policy validation

均需有自动化测试覆盖。

---

## 非目标

- 本 FR 不要求处理 email verification
- 本 FR 不要求处理 TOTP / WebAuthn / recovery codes
- 本 FR 不要求完成 authorization code / refresh token 主链路替换

---

## 验证方法

```bash
cd auth9-oidc && cargo test password
cd auth9-core && cargo test password
cd auth9-portal && npm run test
```

手动验证：

1. 关闭 Keycloak 本地密码能力后，邮箱/密码登录仍可用
2. 忘记密码与重置密码全流程可在 Auth9 域名下完成
3. 弱密码会被 Auth9 自己拒绝，而不是依赖 Keycloak policy
