# FR-001 延期项：泄漏密码检测增强

**类型**: 功能增强（FR-001 已关闭的延期子项）
**严重程度**: Low
**影响范围**: auth9-core (Backend), auth9-portal (Frontend)
**前置依赖**: FR-001 已完成（BreachedPasswordService 已实现）

---

## 已完成（FR-001 核心）

- ✅ HIBP k-Anonymity API 客户端（Block 模式）
- ✅ 注册流程集成
- ✅ 密码修改/重置集成
- ✅ Fail-open 降级
- ✅ 全局配置（`HIBP_ENABLED` env var）
- ✅ Portal i18n 错误映射
- ✅ 8 个单元测试 + QA 文档

---

## 延期项

### D1: 登录时异步检查（原 R4）

用户成功登录后，异步检查其密码（此时明文可用）是否已泄漏：

- 在 OIDC 登录成功回调中异步调用 `BreachedPasswordService::check_password()`
- 发现泄漏时在 `login_events` 中标记 `breached_password_detected`
- 可选：强制用户修改密码后才能继续（`force_reset_on_breach` 配置）

**涉及文件**:
- `auth9-core/src/domains/identity/api/auth/oidc_flow.rs` — 登录成功后异步检查
- `auth9-core/migrations/` — users 表新增 `password_breached` 标记（可选）

### D2: 租户级配置（原 R5）

将 HIBP 配置从全局 env var 迁移到租户级系统设置：

```rust
pub struct BreachedPasswordConfig {
    pub enabled: bool,                  // 默认 true
    pub mode: BreachCheckMode,          // Block / Warn / Disabled
    pub check_on_login: bool,           // 默认 true
    pub force_reset_on_breach: bool,    // pragma: allowlist secret    // 默认 false
    pub min_breach_count: u64,          // 默认 1
}
```

**涉及文件**:
- `auth9-core/src/models/system_settings.rs` — 配置模型
- `auth9-core/src/domains/identity/service/breached_password.rs` — 读取租户配置
- `auth9-portal/app/routes/dashboard.settings.security.tsx` — Portal 配置 UI

### D3: Warn 模式

Block 模式之外增加 Warn 模式（允许使用泄漏密码但警告用户）：

- 注册/改密响应中附加 `password_warning: "breached"` 字段 <!-- pragma: allowlist secret -->
- Portal 显示黄色警告而非阻止提交
- 需要修改注册和密码修改端点的响应类型

**涉及文件**:
- `auth9-core/src/domains/identity/service/password.rs` — Warn 分支
- `auth9-core/src/domains/tenant_access/api/user.rs` — 响应类型扩展
- `auth9-portal/app/routes/register.tsx` — 警告 UI

---

## 优先级建议

D1（登录检查）> D2（租户配置）> D3（Warn 模式）
