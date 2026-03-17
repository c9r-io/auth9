# Phase 2 FR1: Hosted Login 路由与 Branding 托管

**类型**: 性能优化 + 架构演进
**严重程度**: High
**影响范围**: auth9-portal (Frontend), auth9-keycloak-theme, docs/qa
**前置依赖**:
- `keycloak_phase1_fr1_identity_engine_interfaces_and_state_injection.md`
- `keycloak_phase1_fr2_keycloak_adapter_layer.md`
**被依赖**:
- `keycloak_phase2_fr2_hosted_login_api.md`
- `keycloak_phase2_fr3_browser_session_and_refresh_chain.md`

---

## 背景

Phase 2 最直接的用户感知目标是消除“浏览器跳到 Keycloak 页面就慢”的体验问题。为此，第一步应先把登录相关页面的入口和渲染权收回到 Auth9 自己控制的前端。

这一步不要求此时已经完成完整的本地 session 或 refresh 主链路，但要求浏览器主路径上的 UI、branding 和页面路由由 Auth9 托管。

---

## 期望行为

### R1: Portal 托管认证相关页面

新增或补齐以下自有路由与 UI：

- `/login`
- `/register`
- `/forgot-password`
- `/reset-password`
- `/mfa/verify`

要求：

- 浏览器主链路不再默认重定向到 Keycloak hosted page
- 页面渲染由 Auth9 Portal 自己负责
- 页面需适配移动端和桌面端

**涉及文件**:
- `auth9-portal/app/routes/...`
- `auth9-portal/app/components/...`

### R2: Branding 直接由 Auth9 渲染

要求：

- 登录相关页面的 branding 直接由 Portal/Auth9 渲染
- 不再通过 Keycloak theme 二次拉取 Auth9 branding API
- 认证页的 logo、文案、配色、按钮状态与 Portal 其他页面保持一致的品牌来源

### R3: Keycloak theme 退为 fallback

要求：

- `auth9-keycloak-theme/` 仅保留兼容模式
- Keycloak 登录页不再是主入口
- 仍允许在回滚模式下继续使用 Keycloak theme

### R4: 前端入口可区分认证方式

要求：

- `/login` 页可承载 password / enterprise SSO / 其他认证方式的入口选择
- 认证方式入口布局需为后续 Phase 3/4 留扩展位
- 本 FR 不要求所有入口都已完成真实后端逻辑，但 UI 架构不能阻断后续扩展

---

## 非目标

- 本 FR 不要求实现 Hosted Login API
- 本 FR 不要求引入 Auth9 browser session cookie
- 本 FR 不要求本地存储密码、MFA 或完整 refresh 主链路
- 本 FR 不要求移除 Keycloak theme fallback

---

## 验证方法

```bash
cd auth9-portal && npm run typecheck
cd auth9-portal && npm run test
cd auth9-portal && npm run build
```

手动验证：

1. 访问 `/login`、`/register`、`/forgot-password`、`/reset-password`、`/mfa/verify` 时，地址栏始终停留在 Auth9 域名
2. Branding 由 Portal 直接渲染，而不是通过 Keycloak theme 拉取
3. Keycloak theme 仍可作为 fallback 保留，但不是默认入口
