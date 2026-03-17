# Phase 4 FR1: Social Login Broker

**类型**: 核心能力替换
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core (Backend), auth9-portal (Frontend), tests, docs/qa/auth
**前置依赖**:
- `keycloak_phase1_fr1_identity_engine_interfaces_and_state_injection.md`
- `keycloak_phase2_fr3_browser_session_and_refresh_chain.md`
- `keycloak_phase3_fr5_token_issuance_and_auth_server_core.md`
**被依赖**:
- `keycloak_phase4_fr4_federated_identity_linking.md`
- `keycloak_phase4_fr5_federation_audit_and_security_events.md`

---

## 背景

当前 social login 的真实 broker 仍是 Keycloak identity provider。只要 Google、GitHub、Microsoft 等社交登录还是通过 Keycloak 代理，Auth9 就没有真正接管外部身份入口。

---

## 期望行为

### R1: 接管主流 social providers

至少覆盖：

- Google
- GitHub
- Microsoft
- 通用 OIDC Social Provider

### R2: Provider 配置存储在 Auth9

要求：

- provider 配置存储在 Auth9，而不是 Keycloak identity provider 实例
- 包括 client_id / secret、authorize/token/userinfo endpoint、scope、mapping 配置

### R3: Broker 运行时由 Auth9 执行

要求：

- 授权跳转、callback、token exchange、profile mapping 全部由 Auth9 执行
- 浏览器地址栏不应暴露 Keycloak broker endpoint 为主路径

### R4: UI 与回归测试同步

要求：

- Portal 登录页和账号绑定页可驱动新的 social broker 流程
- social provider mock 测试覆盖成功、取消、失败路径

---

## 非目标

- 本 FR 不要求处理 enterprise OIDC / enterprise SAML
- 本 FR 不要求处理联邦账号自动合并策略
- 本 FR 不要求立刻支持所有冷门社交身份源

---

## 验证方法

```bash
cd auth9-core && cargo test social
cd auth9-oidc && cargo test social
cd auth9-portal && npm run test
```

手动验证：

1. 新建 social provider 时不再在 Keycloak 中创建 identity provider
2. social 登录 callback 不再走 Keycloak broker endpoint
3. 登录成功后由 Auth9 自己拿到外部 profile 并继续认证链路
