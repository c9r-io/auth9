# Phase 2 FR2: Hosted Login API

**类型**: 架构演进
**严重程度**: High
**影响范围**: auth9-core (Backend), auth9-portal (Frontend), auth9-oidc, tests
**前置依赖**:
- `keycloak_phase1_fr1_identity_engine_interfaces_and_state_injection.md`
- `keycloak_phase1_fr2_keycloak_adapter_layer.md`
- `keycloak_phase2_fr1_hosted_login_routes_and_branding.md`
**被依赖**:
- `keycloak_phase2_fr3_browser_session_and_refresh_chain.md`
- `keycloak_phase3_local_credentials_and_mfa.md`

---

## 背景

前端页面被 Auth9 托管后，还需要与之配套的自有认证 API。否则前端仍会被迫直接依赖旧的 Keycloak 承载路径，无法形成统一的 Hosted Login 编排入口。

本 FR 的目标是让认证表单提交、登出、密码重置等交互统一落到 Auth9 API 域名下，并可在 backend flag 下切换到 Keycloak adapter 或 `auth9-oidc`。

---

## 期望行为

### R1: 新增 Hosted Login API

新增至少以下接口：

- `POST /api/v1/hosted-login/password`
- `POST /api/v1/hosted-login/logout`
- `POST /api/v1/hosted-login/start-password-reset`
- `POST /api/v1/hosted-login/complete-password-reset`

要求：

- API 统一由 Auth9 域名承载
- 请求/响应结构适合前端表单调用，而不是 Keycloak Admin API 风格

### R2: Backend flag 可切换实现

要求：

- 在 `IDENTITY_BACKEND=keycloak|auth9_oidc` 下，Hosted Login API 能分发到不同 backend
- `keycloak` 模式下走 Keycloak adapter
- `auth9_oidc` 模式下走 `auth9-oidc` stub/真实实现

### R3: 错误语义统一

要求：

- 前端可得到统一的错误码/错误消息，不暴露 Keycloak 原始错误结构
- 密码错误、账号不存在、token 失效、重置链接失效等常见场景应有稳定语义

### R4: OpenAPI 与回归测试同步

要求：

- 新接口需补齐 OpenAPI
- HTTP regression tests 覆盖成功和失败场景
- Portal action/loader 可直接调用这些 API

**涉及文件**:
- `auth9-core/src/domains/identity/api/...`
- `auth9-core/src/openapi.rs`
- `auth9-portal/app/routes/...`
- `auth9-core/tests/...`

---

## 非目标

- 本 FR 不要求 browser session cookie 已本地化
- 本 FR 不要求彻底去除服务端内部对 Keycloak 的依赖
- 本 FR 不要求 social / enterprise SSO 全量 Hosted API

---

## 验证方法

```bash
cd auth9-core && cargo test hosted_login
cd auth9-core && cargo test openapi_spec -- --nocapture
cd auth9-core && cargo build
cd auth9-portal && npm run test
```

手动验证：

1. `/login` 等页面提交表单时只访问 Auth9 域名接口
2. `IDENTITY_BACKEND=keycloak` 与 `IDENTITY_BACKEND=auth9_oidc` 下，Hosted Login API 都能完成请求分发
3. 错误提示不直接暴露 Keycloak 原始错误内容
