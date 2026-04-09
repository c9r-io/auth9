# Bot 防护 (CAPTCHA) QA 测试

**模块**: settings / security
**功能**: CAPTCHA 验证（Cloudflare Turnstile 优先）
**FR**: FR-003

---

## 场景 1: CAPTCHA 配置端点返回公开配置

**前提**: auth9-core 已启动

### 步骤

1. 请求公开 CAPTCHA 配置端点：

```bash
curl -sf http://localhost:8080/api/v1/public/captcha-config | jq .
```

### 预期结果

- 返回 200 OK
- 响应包含 `enabled`、`provider`、`site_key`、`mode` 字段
- **不包含** `secret_key` 字段
- 默认配置：`enabled: false`，`mode: "disabled"`

---

## 场景 2: CAPTCHA 禁用时请求正常通过

**前提**: CAPTCHA_ENABLED=false（默认）

### 步骤

1. 正常登录请求，不附带 CAPTCHA token：

```bash
curl -sf -X POST http://localhost:8080/api/v1/hosted-login/password \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@auth9.local", "password": "admin123"}' \ # pragma: allowlist secret
  -w "\n%{http_code}"
```
<!-- pragma: allowlist secret -->

### 预期结果

- 不因缺少 CAPTCHA token 被拒绝（可能返回 200 或其他业务逻辑错误，但不是 403 CAPTCHA_REQUIRED）
- 响应中不包含 `X-Captcha-Required` 头

---

## 场景 3: Always 模式下缺少 token 返回 403 `[DEFERRED - needs CAPTCHA env setup]`

> **⚠️ 环境要求**: 默认的 `docker-compose.yml` **不**设置 CAPTCHA_* 环境变量，
> `auth9-core` 启动时 `enabled=false`。执行本场景前必须：
>
> 1. 创建 `docker-compose.captcha.yml` 覆盖文件，加入：
>    ```yaml
>    services:
>      auth9-core:
>        environment:
>          CAPTCHA_ENABLED: "true"
>          CAPTCHA_MODE: "always"
>          CAPTCHA_PROVIDER: "turnstile"
>          # Cloudflare 官方测试站点 key：始终通过
>          CAPTCHA_SITE_KEY: "1x00000000000000000000AA"
>          CAPTCHA_SECRET_KEY: "1x0000000000000000000000000000000AA"
>    ```
> 2. 运行 `docker compose -f docker-compose.yml -f docker-compose.dev.yml -f docker-compose.captcha.yml up -d auth9-core`
> 3. 确认 `curl http://localhost:8080/api/v1/public/captcha-config` 返回 `"enabled": true`。
>
> 如果以上步骤未完成，本场景及场景 4、5 会因 CAPTCHA 未启用而不可测。

**前提**: CAPTCHA_ENABLED=true, CAPTCHA_MODE=always, CAPTCHA_SITE_KEY=test-site-key

### 步骤

1. 发送登录请求，不附带 CAPTCHA token：

```bash
curl -sf -X POST http://localhost:8080/api/v1/hosted-login/password \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "password": "password123"}' \ # pragma: allowlist secret
  -w "\n%{http_code}" \
  -D -
# pragma: allowlist secret
```

### 预期结果

- HTTP 状态码：403
- 响应体包含 `"code": "CAPTCHA_REQUIRED"`
- 响应头包含 `X-Captcha-Required: true`
- 响应头包含 `X-Captcha-Site-Key: test-site-key`

---

## 场景 4: Always 模式下有效 token 通过

**前提**: CAPTCHA_ENABLED=true, CAPTCHA_MODE=always, CAPTCHA_SECRET_KEY 配置正确

### 步骤

1. 获取有效的 Turnstile 测试 token（使用 Cloudflare 测试 keys）
2. 发送登录请求附带 CAPTCHA token：

```bash
curl -sf -X POST http://localhost:8080/api/v1/hosted-login/password \
  -H "Content-Type: application/json" \
  -H "X-Captcha-Token: VALID_TURNSTILE_TOKEN" \
  -d '{"email": "admin@auth9.local", "password": "admin123"}' \ # pragma: allowlist secret
  -w "\n%{http_code}"
```

### 预期结果

- 请求正常处理（不因 CAPTCHA 被拒绝）
- 返回正常的登录响应（200 + token 或业务错误）

---

## 场景 5: 前端页面在 Always 模式下渲染 CAPTCHA 组件

**前提**: CAPTCHA_ENABLED=true, CAPTCHA_MODE=always, CAPTCHA_SITE_KEY 已配置

### 步骤

1. 访问登录页面 http://localhost:3000/login
2. 检查页面是否加载了 Turnstile 脚本
3. 检查密码登录表单中是否显示 CAPTCHA 组件
4. 访问注册页面 http://localhost:3000/register
5. 检查注册表单中是否显示 CAPTCHA 组件
6. 访问忘记密码页面 http://localhost:3000/forgot-password
7. 检查表单中是否显示 CAPTCHA 组件

### 预期结果

- 登录页、注册页、忘记密码页均渲染 CAPTCHA 组件
- 组件通过动态加载 Turnstile 脚本呈现
- 表单中包含隐藏的 `captchaToken` 字段

> **CSP 注意**: Content-Security-Policy 已更新，`script-src` 和 `frame-src` 均允许 `challenges.cloudflare.com`。如果 Turnstile 组件未渲染，请先检查浏览器控制台是否有 CSP 拦截错误。
