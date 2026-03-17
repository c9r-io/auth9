# Phase 3 FR4: 本地 MFA 能力

**类型**: 核心能力替换 + 安全加固
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core (Backend), auth9-portal (Frontend), tests, docs/qa/auth
**前置依赖**:
- `keycloak_phase3_fr1_local_credential_store.md`
- `keycloak_phase2_fr3_browser_session_and_refresh_chain.md`
- `keycloak_phase2_fr1_hosted_login_routes_and_branding.md`
**被依赖**:
- `keycloak_phase3_fr5_token_issuance_and_auth_server_core.md`
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

MFA 是认证内核最敏感的部分之一。只要 TOTP、WebAuthn 和 recovery codes 的签发、验证、重置仍依赖 Keycloak，Auth9 就仍然只是一个前端壳层，而不是认证决策者。

---

## 期望行为

### R1: 接管 TOTP 生命周期

至少覆盖：

- TOTP enroll
- TOTP verify
- TOTP reset

要求：

- TOTP secret 存储在本地 credential store
- challenge 签发与消费由 Auth9 自己完成
- 支持合理的 clock skew 容忍与 replay 防护

### R2: 接管 WebAuthn 生命周期

至少覆盖：

- WebAuthn enroll
- WebAuthn verify
- WebAuthn remove

要求：

- WebAuthn 凭据由 Auth9 自己存储和校验
- 不依赖 Keycloak WebAuthn credential

### R3: 接管 recovery codes

要求：

- recovery code 由 Auth9 生成、显示、消费、失效
- recovery code 使用记录可审计

### R4: `mfa_enabled` 由 Auth9 自己决策

要求：

- `mfa_enabled` 不再由 Keycloak 状态倒推
- 登录链路中的 MFA challenge 由 Auth9 自己决定是否触发

### R5: MFA 回归测试

要求：

- TOTP challenge / skew / replay
- WebAuthn registration / assertion
- recovery code 生成 / 消费 / 重放

均应具备自动化测试。

---

## 非目标

- 本 FR 不要求实现社交登录 MFA step-up
- 本 FR 不要求替换 enterprise federation MFA 协调逻辑
- 本 FR 不要求完整 token issuance 重写

---

## 验证方法

```bash
cd auth9-oidc && cargo test mfa
cd auth9-core && cargo test webauthn
cd auth9-core && cargo test otp
cd auth9-portal && npm run test
```

手动验证：

1. TOTP enroll / verify / reset 不依赖 Keycloak API
2. WebAuthn enroll / verify / remove 不依赖 Keycloak credential
3. recovery code 可正常生成和消费
