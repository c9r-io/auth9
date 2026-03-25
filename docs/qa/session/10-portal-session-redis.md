# 会话与安全 - Portal Session Redis 后端迁移

**模块**: 会话与安全
**测试范围**: Portal BFF Session 从 Cookie 存储迁移至 Redis 服务端存储（FR-007）
**场景数**: 5
**优先级**: 高

---

## 背景说明

Portal BFF 的 Session 架构从 Cookie-based 迁移到 Redis-backed 服务端 Session：

- Cookie 仅存储签名的 opaque session ID，不再包含 accessToken / refreshToken
- 所有 Session 数据存储在 Redis，key 前缀为 `portal:session:`
- Session TTL = 8 小时
- 删除 Redis key 可立即撤销 Session
- Redis 不可用时优雅降级至登录页（不返回 500）

---

## 步骤 0：Gate Check（所有场景前置）

确认环境就绪：

```bash
# 1. 确认 Portal 正在运行
curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/login
# 预期: 200

# 2. 确认 Auth9 Core 正在运行
curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health
# 预期: 200

# 3. 确认 Redis 可用
docker exec auth9-redis redis-cli PING
# 预期: PONG
```

---

## 场景 1：正常登录后 Cookie 仅含 session ID

### 初始状态
- Portal 与 Auth9 Core 正常运行
- 测试用户 `admin@auth9.local` 已存在

### 目的
验证登录后浏览器 Cookie 仅包含 opaque session ID，不包含 accessToken、refreshToken 等敏感信息

### 测试操作流程
1. 打开浏览器，访问 `http://localhost:3000/login`
2. 使用 `admin@auth9.local` 登录
3. 登录成功后，打开浏览器 DevTools → Application → Cookies → `localhost`
4. 找到 Session Cookie（名称通常为 `__session` 或 `session`）
5. 复制 Cookie 值，检查内容

### 预期结果
- Cookie 值为不可读的 opaque 字符串（签名后的 session ID），**不是** base64 编码的 JSON
- 尝试 base64 decode Cookie 值，不应得到包含 `accessToken` / `refreshToken` 的 JSON：
  ```bash
  # 将浏览器中的 cookie 值赋给变量
  COOKIE_VALUE="<从浏览器复制的 cookie 值>"
  echo "$COOKIE_VALUE" | base64 -d 2>/dev/null
  # 预期: 解码失败或输出中不包含 accessToken / refreshToken 字样
  ```
- Cookie 属性满足安全要求：
  - `HttpOnly`: true
  - `Secure`: true（生产环境）或可为 false（localhost 开发环境）
  - `SameSite`: Lax 或 Strict

---

## 场景 2：Redis 中存储完整 Session 数据

### 初始状态
- 场景 1 已完成，用户已登录

### 目的
验证 Redis 中以 `portal:session:` 为前缀存储了完整的 Session 数据（accessToken、refreshToken 等）

### 测试操作流程
1. 在 Redis 中查找当前用户的 Session key：
   ```bash
   docker exec auth9-redis redis-cli KEYS "portal:session:*"
   # 预期: 至少返回 1 个 key
   ```

2. 获取 Session 数据内容：
   ```bash
   # 用上一步返回的 key 替换
   docker exec auth9-redis redis-cli GET "portal:session:<session_id>"
   ```

3. 检查 Session key 的 TTL：
   ```bash
   docker exec auth9-redis redis-cli TTL "portal:session:<session_id>"
   ```

### 预期结果
- Redis key 存在且前缀为 `portal:session:`
- Session 数据为 JSON 结构，包含以下字段：
  - `identityAccessToken` — 非空字符串（identity token）
  - `refreshToken` — 非空字符串（仅存在于 Redis，不在 Cookie 中）
  - 注意：`accessToken` 和 `expiresAt` 是冗余别名，已在存储时剥离（读取时由 `normalizeSession` 重建）
- TTL 值在合理范围内（登录后应接近 28800 秒 = 8 小时）

### 预期数据状态
```bash
# 验证 key 存在
docker exec auth9-redis redis-cli EXISTS "portal:session:<session_id>"
# 预期: 1

# 验证 TTL 在 0~28800 范围内
docker exec auth9-redis redis-cli TTL "portal:session:<session_id>"
# 预期: 返回值 > 0 且 <= 28800

# 验证数据包含 identityAccessToken
docker exec auth9-redis redis-cli GET "portal:session:<session_id>" | grep -c "identityAccessToken"
# 预期: 1
```

---

## 场景 3：删除 Redis key 立即撤销 Session

### 初始状态
- 用户已登录，浏览器持有有效 Session Cookie
- Redis 中存在对应的 `portal:session:<session_id>` key

### 目的
验证直接删除 Redis 中的 Session key 后，用户的下一次请求被重定向到登录页，实现即时 Session 撤销

### 测试操作流程
1. 确认当前 Session 有效（访问受保护页面正常）：
   ```bash
   curl -s -o /dev/null -w "%{http_code}" \
     -b "session=<cookie_value>" \
     http://localhost:3000/dashboard
   # 预期: 200
   ```

2. 删除 Redis 中的 Session key：
   ```bash
   docker exec auth9-redis redis-cli DEL "portal:session:<session_id>"
   # 预期: 1 (成功删除 1 个 key)
   ```

3. 再次使用相同 Cookie 访问受保护页面：
   ```bash
   curl -s -o /dev/null -w "%{http_code}" \
     -b "session=<cookie_value>" \
     -L --max-redirs 0 \
     http://localhost:3000/dashboard
   # 预期: 302 (重定向到登录页)
   ```

### 预期结果
- 步骤 1：正常返回 200
- 步骤 2：Redis DEL 返回 1
- 步骤 3：返回 302，`Location` header 指向 `/login`
- 浏览器中刷新页面，跳转到登录页

### 预期数据状态
```bash
# 确认 key 已被删除
docker exec auth9-redis redis-cli EXISTS "portal:session:<session_id>"
# 预期: 0
```

---

## 场景 4：正常登出清除 Redis Session

### 初始状态
- 用户已登录，Redis 中存在对应 Session key
- 记录当前 Session 的 Redis key 名称

### 目的
验证用户正常登出后，Redis 中的 Session key 被删除且浏览器 Cookie 被清除

### 测试操作流程
1. 登录并记录 Session key：
   ```bash
   docker exec auth9-redis redis-cli KEYS "portal:session:*"
   # 记录返回的 key，例如 portal:session:abc123
   ```

2. 执行登出操作：
   - 浏览器方式：点击 Portal 右上角用户菜单 → 登出
   - 或 curl 方式：
     ```bash
     curl -s -o /dev/null -w "%{http_code}" \
       -b "session=<cookie_value>" \
       -X POST \
       http://localhost:3000/logout
     # 预期: 302 (重定向到登录页)
     ```

3. 检查 Redis 中 Session key 是否已删除：
   ```bash
   docker exec auth9-redis redis-cli EXISTS "portal:session:<session_id>"
   ```

4. 检查浏览器 Cookie 是否已清除：
   - 打开 DevTools → Application → Cookies
   - 查看 Session Cookie 是否已被删除或设为过期

### 预期结果
- 登出后返回 302 重定向到登录页
- Redis 中对应 Session key 已不存在
- 浏览器中 Session Cookie 已被清除（`Set-Cookie` header 中 `Max-Age=0` 或 `Expires` 为过去时间）

### 预期数据状态
```bash
# 验证 Redis key 已删除
docker exec auth9-redis redis-cli EXISTS "portal:session:<session_id>"
# 预期: 0

# 验证无残留 key（如果测试环境中只有一个用户登录）
docker exec auth9-redis redis-cli KEYS "portal:session:*"
# 预期: (empty array) 或不包含已登出用户的 key
```

---

## 场景 5：Redis 不可用时优雅降级

### 初始状态
- 用户已登录，浏览器持有有效 Session Cookie
- Redis 服务正常运行

### 目的
验证 Redis 不可用时，Portal 返回登录页重定向（而非 500 错误页）

### 测试操作流程
1. 确认当前 Session 有效：
   ```bash
   curl -s -o /dev/null -w "%{http_code}" \
     -b "session=<cookie_value>" \
     http://localhost:3000/dashboard
   # 预期: 200
   ```

2. 停止 Redis 服务：
   ```bash
   docker stop auth9-redis
   ```

3. 使用相同 Cookie 访问受保护页面：
   ```bash
   curl -s -o /dev/null -w "%{http_code}" \
     -b "session=<cookie_value>" \
     -L --max-redirs 0 \
     http://localhost:3000/dashboard
   ```

4. 尝试访问登录页（确认 Portal 本身未崩溃）：
   ```bash
   curl -s -o /dev/null -w "%{http_code}" \
     http://localhost:3000/login
   ```

5. 恢复 Redis 服务：
   ```bash
   docker start auth9-redis
   ```

6. 等待几秒后再次验证正常访问：
   ```bash
   sleep 3
   curl -s -o /dev/null -w "%{http_code}" \
     -b "session=<cookie_value_new>" \
     http://localhost:3000/dashboard
   ```

### 预期结果
- 步骤 1：正常返回 200
- 步骤 3：返回 302 重定向到 `/login`，**不是** 500 / 502 / 503
- 步骤 4：登录页正常返回 200（Portal 进程未崩溃）
- 步骤 5：Redis 恢复后，重新登录可正常使用
- Portal 日志中应有 Redis 连接失败的 warning/error 日志，但无 uncaught exception
