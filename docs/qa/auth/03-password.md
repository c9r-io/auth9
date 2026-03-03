# 认证流程 - 密码管理测试

**模块**: 认证流程
**测试范围**: 密码重置、修改、强度验证
**场景数**: 5

---

## 架构说明

Auth9 采用 Headless Keycloak 架构，密码相关的用户交互页面均由 Keycloak 托管：

1. **忘记密码 / 重置密码页面** → 由 Keycloak 托管，使用 auth9-keycloak-theme 自定义外观（基于 Keycloakify），用户看到的是 Auth9 品牌风格而非 Keycloak 原生 UI
2. **密码强度验证** → 由 Keycloak 根据 realm 密码策略执行，Auth9 通过 KeycloakSyncService 将租户密码策略同步到 Keycloak
3. **修改密码（已登录用户）** → 在 Auth9 Portal 的设置页面发起，后端通过 Keycloak Admin API 完成密码更新

**进入密码相关页面的路径**：
- Portal `/login` → 点击「**Sign in with password**」→ 跳转到 Keycloak 品牌化登录页
- 在 Keycloak 登录页上可以看到「忘记密码」链接，点击后进入密码重置流程
- 已登录用户的「设置 → 安全 → 修改密码」→ Auth9 Portal 页面

> **注意**：「忘记密码」链接位于 Keycloak 品牌化登录页上（非 Portal `/login` 页面）。QA 需要先点击「Sign in with password」进入 Keycloak 页面才能看到该链接。

**测试原则**：
- 默认从 Auth9 Portal 登录入口点击「**Sign in with password**」触发密码相关流程
- 不要求必须手工直接访问 Keycloak 登录页面 URL
- 如需排障可直接访问 Keycloak 托管页面进行补充验证

---

## 场景 1：忘记密码 - 发送重置邮件

### 初始状态
- 用户已注册但忘记密码
- 邮件服务已配置

### 目的
验证忘记密码功能

### 测试操作流程
1. 在 Portal `/login` 页面点击「**Sign in with password**」进入 Keycloak 品牌化登录页
2. 点击「忘记密码」链接
2. 输入注册邮箱：`user@example.com`
3. 点击「发送重置链接」

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
- Mailpit 已配置为 Keycloak 的 SMTP 服务（开发环境自动配置）

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
2. 在 Keycloak 重置页面输入新密码：`NewSecurePass123!`
3. 确认新密码
4. 提交

### 预期结果
- 显示密码重置成功
- 可以使用新密码登录
- 重置令牌失效

### 预期数据状态
```sql
-- Keycloak 中密码已更新（通过尝试登录验证）

-- 重置令牌应被标记为已使用
SELECT used_at FROM password_reset_tokens WHERE id = '{token_id}';
-- 预期: used_at 有值
```

---

## 场景 3：使用过期重置令牌

### 初始状态
- 重置令牌已过期

### 目的
验证过期令牌处理

### 测试操作流程
1. 从场景 2 获取重置链接，但**不立即使用**
2. 等待令牌过期（Keycloak 默认 5 分钟），或通过 Keycloak Admin API 缩短过期时间
3. 使用过期的重置链接

> **提示**：可通过修改链接中的 URL 参数或等待足够时间来测试过期场景。

### 预期结果
- 显示错误：「链接已过期，请重新申请」

---

## 场景 4：修改密码（已登录用户）

### 初始状态
- 用户已登录
- 用户知道当前密码

### 目的
验证修改密码功能

### 测试操作流程
1. 进入「设置」→「安全」
2. 点击「修改密码」
3. 输入当前密码
4. 输入新密码
5. 确认新密码
6. 提交

### 预期结果
- 显示密码修改成功
- 可以使用新密码登录

### 预期数据状态
```sql
-- Keycloak 中密码已更新
```

---

## 场景 5：密码强度验证

### 初始状态
- **Keycloak 密码策略已由 Seeder 配置**（auth9-core 启动时自动执行）
- 策略要求：最少 12 字符、至少 1 个大写字母、1 个小写字母、1 个数字、1 个特殊字符
- 验证策略是否生效（**必须使用 Bearer Token，`-u admin:admin` 基础认证可能返回不完整数据**）：
  ```bash
  KC_TOKEN=$(curl -s -X POST "http://localhost:8081/realms/master/protocol/openid-connect/token" \
    -d "client_id=admin-cli" -d "username=admin" -d "password=admin" -d "grant_type=password" \
    | python3 -c "import sys,json; print(json.load(sys.stdin)['access_token'])")
  curl -s "http://localhost:8081/admin/realms/auth9" -H "Authorization: Bearer $KC_TOKEN" \
    | python3 -c "import sys,json; print(json.load(sys.stdin).get('passwordPolicy','NULL'))"
  # 预期: 非 null，包含 "length(12) and upperCase(1) and ..."
  ```

> **注意**：如果 `passwordPolicy` 为 `null`，说明 auth9-core 的 Seeder 尚未完成初始化。请确保 auth9-core 已成功启动并完成数据库迁移和 Keycloak 配置同步。可检查日志：`docker logs auth9-init 2>&1 | grep -i "password\|policy\|seeder"`。注意：seeder 运行在 `auth9-init` 容器中，不是 `auth9-core`。

### 目的
验证密码强度验证

### 测试操作流程
通过 Keycloak 登录页的「忘记密码」流程重置密码时测试以下弱密码：
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
| 注册页面显示 "Registration not allowed" | Auth9 禁用了 Keycloak 直接注册（设计如此） | 通过「忘记密码」流程或 Portal 修改密码页面测试 |
| 弱密码被接受 | 密码策略未同步到 Keycloak realm | 重启 auth9-core 或手动运行 `./scripts/reset-docker.sh` |

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | 忘记密码 | ☐ | | | |
| 2 | 重置密码 | ☐ | | | |
| 3 | 过期重置令牌 | ☐ | | | |
| 4 | 修改密码 | ☐ | | | |
| 5 | 密码强度验证 | ☐ | | | |
