# Phase 1 FR1: 身份引擎接口与 State 注入

**类型**: 架构演进
**严重程度**: High
**影响范围**: auth9-core (Backend), tests
**前置依赖**: `keycloak_replacement_program.md`
**被依赖**:
- `keycloak_phase1_fr2_keycloak_adapter_layer.md`
- `keycloak_phase1_fr4_auth9_oidc_skeleton_and_backend_flag.md`

---

## 背景

当前 `auth9-core` 的业务逻辑和状态注入链直接暴露 `KeycloakClient`。这使 Keycloak 不是一个可替换实现，而是默认且唯一的身份后端。

在真正替换实现之前，必须先完成两件事：

1. 定义中性的身份引擎能力接口
2. 让业务层通过 State 注入抽象，而不是直接拿 `KeycloakClient`

FR1 的目标是完成这层“编排边界”搭建，但不要求此时已经具备完整的替换实现。

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
- 每个接口按能力面拆分，禁止做成一个巨型 super trait
- 抽象层需要提供后续扩展到 `auth9_oidc` 的空间

**涉及文件**:
- `auth9-core/src/identity_engine/`

### R2: AppState 注入抽象实例

`AppState` / test state 需要注入抽象身份后端，而不是只有 `KeycloakClient`：

- State 可持有 `Arc<dyn IdentityEngine>` 或按能力拆分后的 typed store
- `auth9-core/src/state.rs` 应能暴露抽象访问入口
- 测试态 `TestAppState` 也必须能注入同一套抽象

**涉及文件**:
- `auth9-core/src/state.rs`
- `auth9-core/src/server/mod.rs`
- `auth9-core/tests/support/http.rs`

### R3: 域服务切到抽象依赖

第一批需要脱离 `KeycloakClient` 的 domain service 至少包括：

- Session 管理
- Linked identity / identity provider 管理
- 任何仅需要“更新 realm 配置”之类有限能力的同步服务

要求：

- 域服务构造函数改为接收 trait object / 泛型抽象
- 域服务代码中不再直接 `use crate::keycloak::KeycloakClient`

**涉及文件**:
- `auth9-core/src/domains/identity/service/session.rs`
- `auth9-core/src/domains/identity/service/identity_provider.rs`
- `auth9-core/src/domains/platform/service/keycloak_sync.rs`

### R4: 不扩大到全量 API handler 重写

本 FR 只要求业务层和状态注入脱离 Keycloak 语义，不要求一次性改完所有 API handler 中的 Keycloak 调用。

要求：

- API handler 里遗留的 Keycloak 调用可以暂时保留
- 但不得阻断后续 FR2/FR4 通过 adapter 或新 backend 接管

---

## 非目标

- 本 FR 不要求实现 Keycloak adapter
- 本 FR 不要求新增 `auth9-oidc` 服务
- 本 FR 不要求修改数据库字段命名
- 本 FR 不要求替换当前登录、token、userinfo 流程

---

## 验证方法

```bash
rg -n "trait IdentityEngine|trait FederationBroker|trait IdentitySessionStore" auth9-core/src/identity_engine
rg -n "KeycloakClient" auth9-core/src/domains/identity/service auth9-core/src/domains/platform/service
cd auth9-core && cargo test identity_engine
cd auth9-core && cargo test session_service
cd auth9-core && cargo test identity_provider_service
```

手动验证：

1. `auth9-core` 正常启动，State 构造链可完成
2. 现有依赖 SessionService / IdentityProviderService 的 HTTP 路径不因注入方式变更而崩溃
