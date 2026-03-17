# Phase 3 FR3: 邮箱验证与 Required Actions 本地化

**类型**: 核心能力替换
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core (Backend), auth9-portal (Frontend), email templates, tests
**前置依赖**:
- `keycloak_phase3_fr1_local_credential_store.md`
- `keycloak_phase3_fr2_password_and_reset_flow.md`
- `keycloak_phase2_fr1_hosted_login_routes_and_branding.md`
**被依赖**:
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

即使密码已经本地化，只要邮箱验证、首次登录动作、强制更新密码或补充 profile 仍依赖 Keycloak required actions 页面，Auth9 的认证流程仍然没有真正闭环。

---

## 期望行为

### R1: Auth9 自己实现 email verification

要求：

- verify email token 由 Auth9 自己签发和消费
- 支持 replay 防护、过期校验和成功后的状态更新
- 前端验证页面由 Auth9 托管

### R2: Required actions 由 Auth9 管理

至少覆盖：

- force update password
- 强制补充 profile
- 首次登录 required action

要求：

- required actions 状态存储在本地 credential / account 模型中
- 不再依赖 Keycloak required actions 页面

### R3: Hosted 前端流程闭环

要求：

- 所有 required actions 页面由 Auth9 域名托管
- 登录后若存在 pending action，Auth9 自己决定跳转到哪个流程

### R4: 邮件模板与审计同步

要求：

- 邮件模板、通知文案、审计事件与现有 Auth9 语义保持一致
- 不把 Keycloak 原始 required action 文案泄露到新流程

---

## 非目标

- 本 FR 不要求实现 TOTP / WebAuthn challenge
- 本 FR 不要求 token issuance 完整替换
- 本 FR 不要求处理社交登录或企业联邦登录的 required actions

---

## 验证方法

```bash
cd auth9-oidc && cargo test email_verification
cd auth9-core && cargo test required_action
cd auth9-portal && npm run test
```

手动验证：

1. verify email 流程可在 Auth9 域名下完成
2. 首次登录 required actions 不再跳转到 Keycloak 页面
3. force update password / 补充 profile 由 Auth9 自己决策和渲染
