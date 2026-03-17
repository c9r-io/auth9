# Phase 3 FR5: Token Issuance 与认证服务器核心链路

**类型**: 核心能力替换
**严重程度**: Critical
**影响范围**: `auth9-oidc`, auth9-core (Backend), auth9-portal (Frontend), tests, docs/qa/auth, docs/security
**前置依赖**:
- `keycloak_phase3_fr1_local_credential_store.md`
- `keycloak_phase3_fr2_password_and_reset_flow.md`
- `keycloak_phase3_fr4_local_mfa.md`
- `keycloak_phase2_fr3_browser_session_and_refresh_chain.md`
**被依赖**:
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

Phase 3 中最重的一块不是 password 或 MFA，而是认证服务器核心链路本身。只要 authorization code、token issuance、refresh、userinfo 仍然内部依赖 Keycloak，Auth9 就没有真正接管 OIDC 核心。

因此这部分必须单独治理，单独验证，单独回滚。

---

## 期望行为

### R1: Auth9 自己完成 authorization code 存储与消费

要求：

- authorization code 由 Auth9 自己签发、存储、校验和消费
- code replay 必须被拒绝
- 与 session / user / client 状态保持一致

### R2: Auth9 自己签发 token

至少覆盖：

- access token
- refresh token
- id token

要求：

- token claims 由 Auth9 自己生成
- refresh path 不再内部调用 Keycloak token endpoint

### R3: OIDC 核心校验由 Auth9 自己完成

至少覆盖：

- nonce
- PKCE
- state

要求：

- 校验逻辑不能再依赖 Keycloak
- 失败语义需要稳定且可审计

### R4: `userinfo` 由 Auth9 自己生成

要求：

- `userinfo` 直接基于 Auth9 token claims / 本地用户数据
- 不再内部调用 Keycloak userinfo endpoint

### R5: 安全回归与兼容性测试

要求：

- PKCE / nonce / state regression
- refresh rotation / replay / expiry
- authorization code replay
- token claim correctness

均需自动化覆盖。

---

## 非目标

- 本 FR 不要求处理 social login / enterprise federation token broker
- 本 FR 不要求完成 Keycloak 完全下线
- 本 FR 不要求一步替换所有外部协议实现

---

## 验证方法

```bash
cd auth9-oidc && cargo test token
cd auth9-core && cargo test auth
cd auth9-core && cargo test oidc_flow
```

手动验证：

1. 关闭 Keycloak token / userinfo 主链路后，Auth9 仍能完成登录、refresh、userinfo
2. PKCE、nonce、state 校验均由 Auth9 自己处理
3. authorization code 和 refresh token 的 replay 会被拒绝
