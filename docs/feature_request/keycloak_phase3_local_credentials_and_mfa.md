# Phase 3: 本地账号、密码、MFA 与邮件动作闭环

**类型**: 核心能力替换
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core (Backend), auth9-portal (Frontend), migrations
**前置依赖**:
- `keycloak_phase1_identity_engine_abstraction.md`
- `keycloak_phase2_hosted_login_and_session_frontend.md`
**被依赖**:
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

只切掉页面还不够。只要本地账号、密码、TOTP、WebAuthn、密码重置、邮箱验证等核心动作仍由 Keycloak 存储和决策，Auth9 就没有真正拿回认证内核。

本阶段目标是让第一方账号体系完全由 Auth9 自己托管。

---

## 期望行为

### R1: 本地 Credential Store

新增中性凭据模型：

- password hash
- TOTP secret
- recovery codes
- WebAuthn credentials
- email verification status
- required actions / pending actions

要求：

- 不再依赖 Keycloak credentials API
- 数据模型不出现 `keycloak_*` 语义

**涉及文件**:
- `auth9-oidc/migrations/`
- `auth9-oidc/src/domain/credential/...`
- `auth9-core` 对接层

### R2: Password Granting / Password Reset

Auth9 自己完成：

- 密码验证
- 密码修改
- 管理员重置密码
- 忘记密码 token 签发与消费
- 密码策略校验

要求：

- 不再依赖 Keycloak realm password policy
- Password policy 由 Auth9 自己存储和执行
- 审计日志保留现有事件语义

### R3: Email Verification / Required Actions

Auth9 自己实现：

- verify email token
- force update password
- 强制补充 profile
- 首次登录 required action

要求：

- 不再依赖 Keycloak required actions 页面
- 前端流程全部由 Auth9 托管

### R4: MFA

接管以下能力：

- TOTP enroll
- TOTP verify
- TOTP reset
- WebAuthn enroll / verify / remove
- recovery code 生成与消费

要求：

- `mfa_enabled` 由 Auth9 自己决策
- 登录链路中的 MFA challenge 由 Auth9 自己签发和消费

### R5: Token Issuance

Auth9 自己完成：

- authorization code 存储与消费
- access token
- refresh token
- id token
- nonce / PKCE / state 校验

要求：

- `userinfo` 直接基于 Auth9 token claims
- refresh path 不再内部调用 Keycloak token endpoint

### R6: 测试覆盖

- password hash / verify
- reset token replay / expiry
- email verify replay / expiry
- TOTP challenge / skew / replay
- WebAuthn registration / assertion
- PKCE / nonce / state / refresh regression

---

## 非目标

- 本阶段不要求替换企业 OIDC/SAML broker
- 本阶段不要求替换社交登录
- 本阶段不要求切除所有 Keycloak 管理对象

---

## 验证方法

```bash
cd auth9-oidc && cargo test
cd auth9-core && cargo test auth
```

手动验证：

1. 关闭 Keycloak 的本地登录能力后，邮箱+密码登录仍可用
2. TOTP/WebAuthn enroll 与验证不依赖 Keycloak API
3. 密码重置、邮箱验证、required actions 全流程可在 Auth9 域名下完成

