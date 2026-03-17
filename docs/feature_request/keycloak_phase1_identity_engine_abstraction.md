# Phase 1: 身份引擎抽象层

**类型**: 架构演进
**严重程度**: High
**影响范围**: auth9-core (Backend), 新增 `auth9-oidc` 服务骨架, tests
**前置依赖**: `keycloak_replacement_program.md`
**被依赖**:
- `keycloak_phase2_hosted_login_and_session_frontend.md`
- `keycloak_phase3_local_credentials_and_mfa.md`
- `keycloak_phase4_external_identity_broker.md`
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

当前 `auth9-core` 在业务层直接持有 `KeycloakClient`，调用范围覆盖：

- 用户 CRUD
- OIDC client / SAML client 管理
- refresh / userinfo
- session 吊销
- identity provider 管理
- federated identity 管理
- realm 配置和密码策略

这使得 Keycloak 不是“可替换实现”，而是“编译期绑定的核心模块”。Phase 1 的目标不是替掉 Keycloak，而是把它从业务语义中剥离。

---

## 期望行为

### R1: 新增统一身份后端接口

引入一组中性 trait，至少覆盖当前已使用的能力面：

```rust
pub trait IdentityEngine;
pub trait IdentityUserStore;
pub trait IdentityClientStore;
pub trait IdentitySessionStore;
pub trait IdentityCredentialStore;
pub trait FederationBroker;
pub trait IdentityEventSource;
```

要求：

- trait 命名不得出现 `Keycloak`
- 业务层只依赖抽象，不依赖具体实现
- 每个接口按能力面拆分，避免一个巨型 super trait

**涉及文件**:
- `auth9-core/src/identity_engine/` — 新增模块
- `auth9-core/src/state/...` — AppState 注入抽象实例

### R2: Keycloak 适配器层

保留现有 Keycloak 代码，但迁移为适配器实现：

- `KeycloakUserStoreAdapter`
- `KeycloakClientStoreAdapter`
- `KeycloakSessionStoreAdapter`
- `KeycloakFederationBrokerAdapter`

要求：

- 原 `KeycloakClient` 继续存在，但只在 adapter 层可见
- `auth9-core` 域服务不再直接引用 `KeycloakClient`

**涉及文件**:
- `auth9-core/src/keycloak/`
- `auth9-core/src/identity_engine/adapters/keycloak/`

### R3: 新增 `auth9-oidc` 服务骨架

新增独立服务目录 `auth9-oidc/`，先不要求完整功能，只要求：

- 有独立配置结构
- 有 health endpoint
- 有最小数据库连接
- 有最小 `IdentityEngine` 实现骨架

要求：

- 这是未来替换 Keycloak 的承载体
- 不把全部逻辑继续塞回 `auth9-core`

**涉及文件**:
- `auth9-oidc/` — 新服务目录
- `docker-compose.yml`
- `deploy/k8s/` — 新组件占位

### R4: 用户和会话模型去 Keycloak 语义化

把内部持久化模型中的 `keycloak_*` 字段改造成中性命名，至少做到兼容扩展：

建议方案：

- `users.keycloak_id` → `identity_subject`
- `sessions.keycloak_session_id` → `provider_session_id`
- `enterprise_sso_connectors.keycloak_alias` → `provider_alias`

要求：

- 第一阶段允许保留旧列并增加新列，避免破坏式迁移
- 代码层优先切换到中性字段

**涉及文件**:
- `auth9-core/migrations/`
- `auth9-core/src/models/user.rs`
- `auth9-core/src/models/session.rs`
- `auth9-core/src/models/enterprise_sso.rs`
- Repository 实现和查询

### R5: Feature Flag

新增身份后端选择开关：

```yaml
IDENTITY_BACKEND=keycloak|auth9_oidc
```

要求：

- 默认仍为 `keycloak`
- dev/test 可切到 `auth9_oidc`
- 启动日志必须打印当前 backend

### R6: 测试覆盖

- trait contract tests
- Keycloak adapter 测试
- `auth9-oidc` stub backend 测试
- State 注入测试
- 双 backend 切换 smoke test

---

## 非目标

- 本阶段不要求去掉 Keycloak 页面
- 本阶段不要求自研 token issuance
- 本阶段不要求迁移密码、MFA、社交登录

---

## 验证方法

```bash
rg -n "KeycloakClient" auth9-core/src
rg -n "IdentityEngine|FederationBroker|IdentitySessionStore" auth9-core/src auth9-oidc/src
cd auth9-core && cargo test identity_engine
```

手动验证：

1. 使用 `IDENTITY_BACKEND=keycloak` 启动，现有登录流程不受影响
2. 使用 `IDENTITY_BACKEND=auth9_oidc` 启动，最小 health / stub 调用可用

