# OIDC Logout - 登出流程

- **模块**: Logout
- **端点**: `GET/POST /api/v1/auth/logout`
- **最后更新**: 2026-03-27

## 前置条件

1. 执行环境重置脚本初始化测试数据：
   ```bash
   ./scripts/reset-docker.sh --conformance
   ```
2. 确认 Auth9 Core 服务运行在 `http://localhost:8080`
3. 准备测试用 OAuth Client（`{client_id}`, `{client_secret}`, `{redirect_uri}`）
4. 用户已通过 Authorization Code Flow 完成登录并持有有效 session

---

## 场景 1: GET /api/v1/auth/logout 带有效参数正确重定向

> **需浏览器**: 此场景需要浏览器环境以验证 session 状态和重定向行为。

**目的**: 验证 GET 方式的 logout 请求携带有效参数时，正确清除 session 并重定向到指定 URI。

**步骤**:

1. 在浏览器中完成用户登录，建立有效 session
2. 发起 GET logout 请求

```bash
# GET logout 请求（浏览器中访问此 URL）
curl -s -o /dev/null -w "%{http_code}\n%{redirect_url}" \
  -X GET "http://localhost:8080/api/v1/auth/logout?\
client_id={client_id}&\
post_logout_redirect_uri={redirect_uri}"
```

**预期结果**:
- HTTP 状态码 `302`（重定向）
- `Location` header 指向 `{redirect_uri}`
- 用户 session 已被清除
- 再次访问受保护资源需重新登录

---

## 场景 2: POST /api/v1/auth/logout 携带 Bearer token 撤销会话

> **需浏览器**: 此场景需要浏览器环境获取有效 session token。

**目的**: 验证 POST 方式的 logout 请求携带 Bearer token 时，正确撤销对应会话。

**步骤**:

1. 在浏览器中完成用户登录，获取有效 access_token
2. 发起 POST logout 请求

```bash
# POST logout 请求（携带 Bearer token）
curl -s -o /dev/null -w "%{http_code}" \
  -X POST http://localhost:8080/api/v1/auth/logout \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id={client_id}&\
post_logout_redirect_uri={redirect_uri}"
```

**预期结果**:
- HTTP 状态码 `302`（重定向）或 `200`
- 用户 session 已被撤销
- 使用同一 access_token 再次请求 UserInfo 端点返回 `401`

**验证 session 已撤销**:
```bash
# 使用已撤销的 token 请求 UserInfo，应返回 401
curl -s -o /dev/null -w "%{http_code}" \
  -X GET http://localhost:8080/api/v1/auth/userinfo \
  -H "Authorization: Bearer ${ACCESS_TOKEN}"
```

---

## 场景 3: 无效的 post_logout_redirect_uri 返回错误

**目的**: 验证 logout 请求携带未注册的 redirect URI 时被正确拒绝。

**步骤**:

```bash
# 使用未注册的 redirect URI
curl -s -o /dev/null -w "%{http_code}" \
  -X GET "http://localhost:8080/api/v1/auth/logout?\
client_id={client_id}&\
post_logout_redirect_uri=https://malicious-site.example.com/callback"

# 查看完整响应
curl -s -X GET "http://localhost:8080/api/v1/auth/logout?\
client_id={client_id}&\
post_logout_redirect_uri=https://malicious-site.example.com/callback" \
  | jq .
```

**预期结果**:
- HTTP 状态码 `400` 或其他错误码
- 响应包含错误信息，指示 redirect URI 无效或未注册
- 不会重定向到未注册的 URI

---

## 场景 4: 参数缺失时的 logout 行为（OIDC RP-Initiated Logout 1.0）

**目的**: 验证不同参数组合下 logout 端点的行为符合 OIDC 规范。

> **注意**: 根据 [OIDC RP-Initiated Logout 1.0](https://openid.net/specs/openid-connect-rpinitiated-1_0.html) 规范第 2 节，所有 logout 参数均为 **OPTIONAL**。无参数请求时 OP 应正常响应（不重定向）。

**步骤**:

```bash
# 4a. 完全不携带参数 → 200 OK（规范允许，OP 本地结束会话）
curl -s -o /dev/null -w "%{http_code}" \
  -X GET "http://localhost:8080/api/v1/auth/logout"

# 4b. 仅携带 client_id → 200 OK（无 redirect_uri 则不重定向）
curl -s -o /dev/null -w "%{http_code}" \
  -X GET "http://localhost:8080/api/v1/auth/logout?\
client_id={client_id}"

# 4c. 仅携带 redirect_uri，缺少 client_id → 400 Bad Request
curl -s -o /dev/null -w "%{http_code}" \
  -X GET "http://localhost:8080/api/v1/auth/logout?\
post_logout_redirect_uri={redirect_uri}"
```

**预期结果**:

| 测试 | 预期状态码 | 说明 |
|------|-----------|------|
| 4a. 无参数 | `200` | 所有参数均为 OPTIONAL，OP 本地处理 |
| 4b. 仅 client_id | `200` | 无 redirect_uri 则不重定向，正常响应 |
| 4c. 仅 redirect_uri | `400` | `client_id` is required when `post_logout_redirect_uri` is specified |

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | GET logout 带有效参数正确重定向 | | | | 需浏览器 |
| 2 | POST logout 携带 Bearer token 撤销会话 | | | | 需浏览器 |
| 3 | 无效的 post_logout_redirect_uri 返回错误 | | | | |
| 4 | 参数缺失时的 logout 行为（规范合规） | | | | 200/200/400 |
