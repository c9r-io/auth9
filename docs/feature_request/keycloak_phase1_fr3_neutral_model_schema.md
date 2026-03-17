# Phase 1 FR3: 用户与会话模型去 Keycloak 语义化

**类型**: 架构演进
**严重程度**: High
**影响范围**: auth9-core (Backend), migrations, tests
**前置依赖**: `keycloak_phase1_fr1_identity_engine_interfaces_and_state_injection.md`
**被依赖**:
- `keycloak_phase2_hosted_login_and_session_frontend.md`
- `keycloak_phase3_local_credentials_and_mfa.md`
- `keycloak_phase4_external_identity_broker.md`
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

即使业务层已经抽象化，只要数据库和模型还用 `keycloak_*` 命名，后续引入 `auth9_oidc` 或其他实现时，代码语义仍然被旧实现绑死。

因此需要在 Phase 1 内完成第一批中性命名迁移，但必须采用兼容方式，避免破坏现网和测试数据。

---

## 期望行为

### R1: 为关键表新增中性字段

建议至少覆盖以下三类：

- `users.keycloak_id` → `identity_subject`
- `sessions.keycloak_session_id` → `provider_session_id`
- `enterprise_sso_connectors.keycloak_alias` → `provider_alias`

要求：

- 第一阶段允许保留旧列
- migration 必须包含旧列到新列的回填
- 新列应具备必要索引

**涉及文件**:
- `auth9-core/migrations/`

### R2: Rust 模型优先切中性字段

要求：

- `User`、`Session`、`EnterpriseSsoConnector` 等模型优先使用中性字段名
- 代码层尽量避免新增 `keycloak_*` 字段引用
- 若为了兼容旧 API/测试必须保留过渡映射，应明确标注为 migration period

**涉及文件**:
- `auth9-core/src/models/user.rs`
- `auth9-core/src/models/session.rs`
- `auth9-core/src/models/enterprise_sso.rs`

### R3: Repository 查询优先读写新列

要求：

- create / find / update / search 查询优先使用中性字段
- 查找历史数据时应兼容已有旧列值
- 过渡期内保持不破坏现有功能

**涉及文件**:
- `auth9-core/src/repository/user/`
- `auth9-core/src/repository/session/`
- `auth9-core/src/domains/tenant_access/api/tenant_sso.rs`
- 其他直接查询上述字段的 SQL

### R4: 测试数据与断言同步更新

要求：

- repository tests
- HTTP regression tests
- mock state / fixture builders

都应切换到中性命名或兼容双字段断言。

---

## 非目标

- 本 FR 不要求一次性处理所有 `keycloak_*` 命名
- 本 FR 不要求删除旧列
- 本 FR 不要求完成生产数据回收或列清理

---

## 验证方法

```bash
rg -n "identity_subject|provider_session_id|provider_alias" auth9-core/src auth9-core/tests auth9-core/migrations
cd auth9-core && cargo test repository::user
cd auth9-core && cargo test repository::session
cd auth9-core && cargo test tenant_sso
```

手动验证：

1. 跑 migration 后，新列存在且旧数据被回填
2. 用户查询、session 查询、企业 SSO connector 查询均仍返回正确数据
3. 代码主路径优先读取中性字段
