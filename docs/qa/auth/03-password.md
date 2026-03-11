# 认证流程 - 密码管理测试

**模块**: 认证流程
**测试范围**: 密码重置、修改、强度验证
**场景数**: 5

---

## 架构说明

Auth9 采用 Headless Keycloak 架构，密码能力应优先通过 Auth9 自身页面和 API 代理：

1. **忘记密码页面** → 优先使用 Auth9 Portal `/forgot-password`
2. **重置密码页面** → 优先使用 Auth9 Portal `/reset-password?token=...`
3. **密码强度验证** → 由 Auth9 后端执行并同步到底层认证引擎
4. **修改密码（已登录用户）** → 在 Auth9 Portal `Account -> Security` 页面发起，后端通过 Auth9 API 完成

> **实现说明**：当前托管认证页（由 `auth9-keycloak-theme` 承载）仍可能承载部分历史密码流程入口，但 QA 主路径应以 Auth9 代理页为准，不应把 Keycloak UI 当作标准验收入口。

**推荐测试入口**：
- 未登录用户：从 `/login` 点击 `Forgot password?`，并验证跳转到 `/forgot-password`
- 收到邮件后：直接访问 `/reset-password?token={token}`
- 已登录用户：进入 `Account -> Security` 修改密码

**测试原则**：
- 默认从 Auth9 Portal 或直接访问 Auth9 代理页触发密码相关流程
- 不要求必须手工直接访问底层登录页面 URL
- 如需排障，可额外核对托管认证页或后台同步状态，但不作为主验收路径

---

## 场景 1：忘记密码 - 发送重置邮件

### 初始状态
- 用户已注册但忘记密码
- 邮件服务已配置

### 目的
验证忘记密码功能

### 测试操作流程
1. 访问 Auth9 Portal `/login`
2. 点击 `Forgot password?`
3. 确认跳转到 `/forgot-password`
4. 输入注册邮箱：`user@example.com`
5. 点击「Send reset link」

### 预期结果
- 显示「如果该邮箱存在，我们已发送重置链接」
- 用户收到重置邮件

### 预期数据状态
```sql
-- 验证重置令牌已创建（如果有 password_reset_tokens 表）
SELECT id, user_id, expires_at FROM password_reset_tokens
WHERE user_id = (SELECT id FROM users WHERE email = 'user@example.com')
ORDER BY created_at DESC LIMIT 1;
-- 预期: 存在记录，expires_at 为未来时间
```

---

## 场景 2：重置密码

### 初始状态
- 用户有有效的密码重置令牌
- Mailpit 已配置为底层认证邮件服务（开发环境自动配置）

### 目的
验证密码重置流程

### 测试操作流程
1. 从 Mailpit 获取重置链接：
   - **方法 A（Web UI）**：打开 `http://localhost:8025`，找到最新的重置邮件，点击邮件中的链接
   - **方法 B（API）**：
     ```bash
     # 获取最新邮件中的重置链接
     curl -s http://localhost:8025/api/v1/messages | \
       python3 -c "import sys,json; msgs=json.load(sys.stdin)['messages']; print(msgs[0]['ID'])" | \
       xargs -I{} curl -s http://localhost:8025/api/v1/message/{} | \
       python3 -c "import sys,json,re; msg=json.load(sys.stdin); links=re.findall(r'http[s]?://[^\s\"<>]+action-token[^\s\"<>]+', msg.get('HTML','')); print(links[0] if links else 'No reset link found')"
     ```
2. 在 Auth9 `/reset-password` 页面输入新密码：`NewSecurePass123!`
3. 确认新密码
4. 提交

### 预期结果
- 显示密码重置成功
- 可以使用新密码登录
- 重置令牌失效

### 预期数据状态
```sql
-- Auth9 API 已受理密码重置，且新密码可用于后续登录验证

-- 重置令牌应被标记为已使用
SELECT used_at FROM password_reset_tokens WHERE id = '{token_id}';
-- 预期: used_at 有值
```

---

## 场景 3：使用过期重置令牌

### 初始状态
- 重置令牌已过期（`expires_at` 早于 `NOW()`）

### 目的
验证过期令牌处理

### 步骤 0：验证令牌已真正过期

**必须在测试前确认令牌已过期，否则测试无效：**

```sql
-- 确认令牌已过期（expires_at 在过去）
SELECT id, expires_at, used_at,
       CASE WHEN expires_at < NOW() THEN 'EXPIRED' ELSE 'VALID' END AS status
FROM password_reset_tokens
WHERE user_id = (SELECT id FROM users WHERE email = 'admin@auth9.local')
ORDER BY created_at DESC LIMIT 1;
-- 必须: status = 'EXPIRED'
-- 若 used_at IS NOT NULL，令牌已使用（测试会因无法找到未使用令牌而失败）
```

> **系统默认令牌有效期：1 小时**。建议通过直接修改数据库使令牌过期，而非等待：
> ```sql
> UPDATE password_reset_tokens
> SET expires_at = '2020-01-01 00:00:00'
> WHERE user_id = (SELECT id FROM users WHERE email = 'admin@auth9.local')
>   AND used_at IS NULL;
> ```

### 测试操作流程
1. 首先通过场景 2 步骤获取重置链接（`/reset-password?token=xxx`）
2. **执行步骤 0 验证令牌已过期**（通过等待或直接修改 DB）
3. 访问已过期的重置链接并提交新密码

### 预期结果
- 显示错误：「链接已过期，请重新申请」

> **故障排除**
>
> | 症状 | 原因 | 解决方案 |
> |------|------|---------|
> | 密码重置成功（而非报错） | 令牌实际未过期 | 执行步骤 0 确认 status=EXPIRED |
> | 页面直接报"无效令牌" | 令牌已被使用（used_at 非空） | 申请新令牌并确保未使用前手动使其过期 |

---

## 场景 4：修改密码（已登录用户）

### 初始状态
- 用户已登录
- 用户知道当前密码

### 目的
验证修改密码功能

### 测试操作流程
1. 进入「Account」→「Security」
2. 在「Change Password」表单中操作
3. 输入当前密码
4. 输入新密码
5. 确认新密码
6. 提交

### 预期结果
- 显示密码修改成功
- 可以使用新密码登录

### 预期数据状态
```sql
-- Auth9 API 已受理密码更新，且新密码可用于后续登录验证
```

---

## 场景 5：密码强度验证

### 初始状态
- **密码策略已由 Seeder 配置并同步到底层认证引擎**（auth9-core 启动时自动执行）
- 策略要求：最少 12 字符、至少 1 个大写字母、1 个小写字母、1 个数字、1 个特殊字符
- 如需做后台同步校验（可选），验证策略是否已同步到内部 realm（**必须使用 Bearer Token，`-u admin:admin` 基础认证可能返回不完整数据**）：
  ```bash
  KC_TOKEN=$(curl -s -X POST "http://localhost:8081/realms/master/protocol/openid-connect/token" \
    -d "client_id=admin-cli" -d "username=admin" -d "password=admin" -d "grant_type=password" \
    | python3 -c "import sys,json; print(json.load(sys.stdin)['access_token'])")
  curl -s "http://localhost:8081/admin/realms/auth9" -H "Authorization: Bearer $KC_TOKEN" \
    | python3 -c "import sys,json; print(json.load(sys.stdin).get('passwordPolicy','NULL'))"
  # 预期: 非 null，包含 "length(12) and upperCase(1) and ..."
  ```

> **注意**：如果 `passwordPolicy` 为 `null`，说明 auth9-core 的 Seeder 尚未完成初始化。请确保 auth9-core 已成功启动并完成数据库迁移和底层策略同步。可检查日志：`docker logs auth9-init 2>&1 | grep -i "password\|policy\|seeder"`。注意：seeder 运行在 `auth9-init` 容器中，不是 `auth9-core`。

### 目的
验证密码强度验证

### 测试操作流程
通过 Auth9 `/reset-password` 页面或 `Account -> Security` 页面测试以下弱密码：
1. 太短：`abc123`
2. 无大写：`password123!`
3. 无数字：`Password!`
4. 无特殊字符：`Password123`

### 预期结果
- 每种情况显示相应的密码强度错误
- 密码不被接受

### 常见误报

| 症状 | 原因 | 解决方法 |
|------|------|----------|
| `passwordPolicy` 为 `null` | Seeder 未完成或 auth9-core 未正常启动 | 检查 `docker logs auth9-core` 确认启动完成 |
| `/login` 页面未直接暴露找回密码入口 | 登录页入口整合尚未完成 | 直接访问 `/forgot-password` 或 `/reset-password` 进行验证 |
| 弱密码被接受 | 密码策略未同步到底层 realm | 重启 auth9-core 或手动运行 `./scripts/reset-docker.sh` |

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | 忘记密码 | ☐ | | | |
| 2 | 重置密码 | ☐ | | | |
| 3 | 过期重置令牌 | ☐ | | | |
| 4 | 修改密码 | ☐ | | | |
| 5 | 密码强度验证 | ☐ | | | |
