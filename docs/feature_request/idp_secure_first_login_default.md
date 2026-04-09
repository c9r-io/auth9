# IdP 首次登录默认策略安全加固

**类型**: 安全加固
**严重程度**: High
**影响范围**: auth9-core (Backend), auth9-portal (Frontend - IdP 配置 UI)
**前置依赖**: 无
**被依赖**: 无

---

## 背景

Auth9 的社交/企业 IdP 登录流程（`auth9-core/src/domains/identity/api/social_broker.rs`）支持三种首次登录策略（`FirstLoginPolicy`）：

1. `auto_merge` — 当 IdP 返回的邮箱匹配到已有用户时，**自动**创建 `linked_identity` 并登录为该用户。
2. `prompt_confirm` — 发现邮箱匹配时，进入确认页面，由已认证用户显式确认合并。
3. `create_new` — 永不基于邮箱合并，始终创建新账号。

**当前默认值为 `auto_merge`**，分布在多个位置：

- `auth9-core/src/models/identity_provider.rs:147` (`default_first_login_policy` 函数)
- `auth9-core/src/models/identity_provider.rs:84` (`From<backend>` 实现)
- `auth9-core/src/models/linked_identity.rs:51-52` (`FirstLoginPolicy::default`)
- `auth9-core/src/identity_engine/types.rs:80` (Federation types)
- `auth9-core/src/models/social_provider.rs:40,156`
- `auth9-core/src/identity_engine/adapters/auth9_oidc/federation_broker.rs:159`
- `auth9-core/src/repository/social_provider.rs:275` (测试数据)

### 风险场景

攻击者控制 IdP（如 GitHub）上与受害者 Auth9 账号同邮箱的账号：
1. 受害者以 `victim@example.com` 注册 Auth9 本地账号。
2. 攻击者在 GitHub 设置邮箱为 `victim@example.com`（GitHub 允许未验证邮箱加入账号）。
3. 攻击者通过 Auth9 的 GitHub IdP 登录。
4. `social_broker.rs:862-900` 命中邮箱匹配分支，默认策略为 `AutoMerge`，于是创建 `linked_identity` 并登录为受害者。

对应 QA 场景：`docs/security/authentication/05-idp-security.md` 场景 1（OAuth 账户关联劫持）。

OWASP ASVS 5.0 参考: V10.5（账号联邦与合并）、V10.6（身份保证）。

---

## 期望行为

### R1: 代码层默认改为 `create_new`

将上述所有默认值统一改为 `create_new`：

- `default_first_login_policy()` 返回 `"create_new"`
- `FirstLoginPolicy::default()` → `CreateNew`（需要移除 `#[default]` 标注并改为 `CreateNew`）
- 所有 `From`/`Default`/测试 fixture 中的 `"auto_merge"` 默认初值改为 `"create_new"`

**注意事项**: 现有数据库记录（`identity_providers.first_login_policy`、`social_providers.first_login_policy` 列）不会被自动修改。需要迁移脚本或管理员手动更新。

### R2: 迁移与向后兼容

提供一次性迁移：

```sql
-- 可选：对现有未显式设置的 IdP 切换到更安全的默认。
-- 不强制 — 管理员可能依赖 auto_merge 作为业务功能。
-- 建议仅在升级说明中提醒。
```

升级文档需明确列出此行为变更，并在 Portal IdP 配置页展示策略说明。

### R3: Portal UI 警告

在 `auth9-portal` 的 IdP 配置表单中：

- 默认下拉选中 `create_new`。
- 选择 `auto_merge` 时显示红色警告："此策略允许 IdP 邮箱自动合并到已有账号，如果 IdP 的邮箱未经验证可能导致账户劫持。仅在 IdP 完全可信时启用。"
- `trust_email=true` 选项同步给出类似警告（当前代码中 `trust_email=true` 会强制走 `AutoMerge` 分支）。

### R4: 遥测

在 `social_broker.rs` 进入 `AutoMerge` 分支时记录结构化日志/指标，便于监控异常账号合并行为：

```rust
tracing::warn!(
    provider_alias = %provider.alias,
    matched_email = %email,
    existing_user_id = %existing_user.id,
    "First-login policy=auto_merge: linked external identity to existing user by email"
);
```

---

## 涉及文件

- `auth9-core/src/models/identity_provider.rs`
- `auth9-core/src/models/linked_identity.rs`
- `auth9-core/src/models/social_provider.rs`
- `auth9-core/src/identity_engine/types.rs`
- `auth9-core/src/identity_engine/adapters/auth9_oidc/federation_broker.rs`
- `auth9-core/src/repository/social_provider.rs`
- `auth9-core/src/domains/identity/api/social_broker.rs`
- `auth9-core/src/domains/identity/api/enterprise_common.rs`
- `auth9-portal/app/routes/dashboard/identity-providers/*`（UI 侧表单、警告文案）
- 单元测试需同步更新：`social_broker.rs` tests 当前断言默认 `AutoMerge`，需改为断言 `CreateNew` 并新增 opt-in `AutoMerge` 测试。

---

## 验证方法

1. 单元测试：`cargo test -p auth9-core first_login_policy` 全部通过，默认返回 `CreateNew`。
2. 手动端到端：创建新 IdP 不显式设置 `first_login_policy`，通过 IdP 登录同邮箱账号应创建新用户而非合并。
3. QA 文档 `docs/security/authentication/05-idp-security.md` 场景 1 重新执行应通过。
4. Portal E2E：IdP 表单默认选中 `create_new`；切换至 `auto_merge` 显示警告。
5. 迁移向后兼容：升级现有环境后，已有 IdP 的 `first_login_policy` 列保持不变。
