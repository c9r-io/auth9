# Phase 2 FR3: Browser Session 与 Refresh 主链路本地化

**类型**: 架构演进 + 安全加固
**严重程度**: High
**影响范围**: auth9-core (Backend), auth9-portal (Frontend), auth9-oidc, tests, docs/qa/session
**前置依赖**:
- `keycloak_phase2_fr1_hosted_login_routes_and_branding.md`
- `keycloak_phase2_fr2_hosted_login_api.md`
- `keycloak_phase1_fr3_neutral_model_schema.md`
**被依赖**:
- `keycloak_phase3_local_credentials_and_mfa.md`
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

仅把页面托管到 Auth9，还不足以真正摆脱浏览器对 Keycloak session 的依赖。若浏览器侧仍依赖 Keycloak cookie、refresh 和 logout 流程，Phase 2 的收益会被打折。

因此需要把 browser session、session rotation、logout、userinfo 与 refresh 的主链路统一收敛到 Auth9 自身。

---

## 期望行为

### R1: 引入 Auth9 browser session cookie

要求：

- 使用 Auth9 自己的 browser session cookie
- Cookie 属性满足：
  - `HttpOnly`
  - `Secure`
  - `SameSite=Lax` 或 `SameSite=None`（按部署模式）
- 支持 session rotation
- 支持 remember-me

### R2: 本地 session 与数据库模型对齐

要求：

- `sessions` 表与 browser session 一一对应
- 若底层仍使用 Keycloak，则本地 session 与 provider session 可关联
- logout 时既能撤销本地 session，也能在需要时撤销 provider session

### R3: `userinfo` 与 refresh 主链路走 Auth9

要求：

- 登录后前端获取用户态信息时只调用 Auth9
- refresh token / session refresh 主链路不再要求浏览器直连 Keycloak
- 若底层仍需访问 Keycloak，只允许在服务端内部发生

### R4: Portal 认证守卫切到本地 session 语义

要求：

- Portal 的认证态判断、未登录跳转、session 失效处理统一基于 Auth9 session
- 不能再依赖 Keycloak browser cookie 是否存在

### R5: 安全回归覆盖

要求：

- 覆盖 logout、session revoke、refresh 失效、state/CSRF、cookie 属性等关键场景
- 既要验证 happy path，也要验证 session fixation / stale refresh / revoked session 等异常路径

---

## 非目标

- 本 FR 不要求 Auth9 已完全自管密码验证实现
- 本 FR 不要求移除服务端内部全部 Keycloak 依赖
- 本 FR 不要求社交登录和企业联邦的 session 流程全部切换

---

## 验证方法

```bash
cd auth9-core && cargo test session
cd auth9-core && cargo test logout
cd auth9-core && cargo test token_exchange
cd auth9-portal && npm run test
```

手动验证：

1. 登录后浏览器持有的是 Auth9 session cookie，而不是依赖 Keycloak browser cookie
2. 登出后本地 session 立即失效
3. 刷新用户态或刷新 token 时，浏览器只访问 Auth9 域名
4. session 过期或被撤销时，Portal 正确回到 `/login`
