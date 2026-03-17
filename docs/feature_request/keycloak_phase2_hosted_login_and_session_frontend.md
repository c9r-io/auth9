# Phase 2: Auth9 Hosted Login 与本地 Session 前端化

**类型**: 性能优化 + 架构演进
**严重程度**: High
**影响范围**: auth9-portal (Frontend), auth9-core (Backend), `auth9-oidc`
**前置依赖**: `keycloak_phase1_identity_engine_abstraction.md`
**被依赖**:
- `keycloak_phase3_local_credentials_and_mfa.md`
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

用户当前最直观的性能问题是“页面跳到 Keycloak 就慢”。因此第二阶段应优先切掉浏览器对 Keycloak 托管页面的依赖，而不是先追求后端完全替换。

本阶段允许 Keycloak 继续作为后端身份源，但浏览器侧登录、登出、注册、忘记密码、MFA 验证页面必须由 Auth9 托管。

---

## 期望行为

### R1: Portal 托管登录页

新增 Auth9 自有登录路由与 UI：

- `/login`
- `/register`
- `/forgot-password`
- `/reset-password`
- `/mfa/verify`

要求：

- 浏览器主链路不再重定向到 Keycloak hosted page
- Branding 直接由 Portal/Auth9 自己渲染，不再通过 Keycloak theme 二次拉取
- 保留旧 Keycloak 登录页作为 fallback，不默认使用

**涉及文件**:
- `auth9-portal/app/routes/...`
- `auth9-portal/app/components/...`
- `auth9-keycloak-theme/` 仅保留兼容模式，不再是主路径

### R2: 新增 Hosted Login API

新增由 Auth9 自己服务的认证表单接口：

- `POST /api/v1/hosted-login/password`
- `POST /api/v1/hosted-login/logout`
- `POST /api/v1/hosted-login/start-password-reset`
- `POST /api/v1/hosted-login/complete-password-reset`

要求：

- API 统一由 Auth9 域名承载
- 可在 backend flag 下分别走 Keycloak adapter 或 `auth9-oidc`

### R3: 本地 Browser Session

引入 Auth9 自己的 browser session cookie：

- HttpOnly
- Secure
- SameSite=Lax`/`None（按部署模式）
- 由 Auth9 管理 session rotation、logout、remember-me

要求：

- Portal 用户态不再依赖 Keycloak browser session cookie
- `sessions` 表和 browser session 一一对应
- 若底层仍使用 Keycloak，本地 session 与 provider session 需可关联

### R4: 本地 `userinfo` 与 refresh 主链路

要求：

- 登录后前端获取用户态信息时只调用 Auth9
- refresh token / session refresh 主链路不再要求浏览器直接触达 Keycloak
- 若底层仍需调用 Keycloak，应只发生在服务端内部

### R5: 性能埋点

新增两组观测指标：

1. 登录首屏性能
2. 登录完成端到端耗时

对比维度：

- `mode=keycloak_hosted`
- `mode=auth9_hosted`

要求：

- 必须能证明本阶段上线后，用户感知延迟下降

### R6: 兼容回滚

要求：

- 支持按环境 / tenant / 百分比切回 Keycloak hosted login
- 回滚不需要数据迁移

---

## 非目标

- 本阶段不要求 Auth9 自己存密码
- 本阶段不要求完全移除 Keycloak refresh / userinfo 内部依赖
- 本阶段不要求迁移 social / enterprise SSO broker

---

## 验证方法

```bash
cd auth9-portal && npm run test
cd auth9-core && cargo test hosted_login
```

手动验证：

1. 登录页访问时浏览器地址栏始终停留在 Auth9 域名
2. Branding 不再经过 Keycloak theme
3. 登录、登出、忘记密码、MFA 页面均由 Auth9 渲染
4. 关闭 Keycloak 对外公开入口后，已迁移登录页仍可打开

