# Phase 1: 身份引擎抽象层（总控 FR）

**类型**: 架构演进  
**严重程度**: High  
**影响范围**: auth9-core (Backend), `auth9-oidc`, migrations, tests, docs/qa  
**前置依赖**: `keycloak_replacement_program.md`

**子 FR**:
- FR1: Identity Engine 能力面与 State 清理（closed）
- FR2: 业务层 Keycloak 解耦（closed）
- FR3: 身份抽象层 QA 与文档对齐（closed）

**被依赖**:
- `keycloak_phase2_hosted_login_and_session_frontend.md`
- `keycloak_phase3_local_credentials_and_mfa.md`
- `keycloak_phase4_external_identity_broker.md`
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

当前 `auth9-core` 已经引入了 `IdentityEngine` 抽象、`Keycloak` adapter、`auth9-oidc` skeleton、中性字段迁移和 backend flag，但总目标尚未完全关闭：

- 仍有部分业务层直接持有或调用 `KeycloakClient`
- `state` 仍显式暴露 `KeycloakClient`
- 一些 handler / service / QA 文档仍保留 Keycloak 语义作为主路径

因此原本单体的 Phase 1 FR 已不适合继续作为一个执行单元，需要拆成可独立实施和关闭的子 FR。

---

## 拆分原则

### 子 FR1：抽象能力面与 State 清理

目标：
- 补齐 `IdentityUserStore` / `IdentityClientStore` / `IdentityCredentialStore` 的最小能力面
- 保证 `KeycloakClient` 只在 adapter 组装层可见
- `state` 不再向业务层暴露 `KeycloakClient`

### 子 FR2：业务层去 `KeycloakClient` 依赖

目标：
- `UserService` / `PasswordService` / `WebAuthnService` / `ScimService` / `SamlApplicationService`
- `tenant_sso` / `user` / `invitation` 等 handler
- 不再直接依赖 `KeycloakClient` 或 `Keycloak*` DTO 作为业务主接口

### 子 FR3：QA 与文档收口

目标：
- 清理 QA 文档中仍把 `keycloak_*` 字段作为主断言的内容
- 新增本阶段 closure QA 文档
- 在子 FR 全部完成后关闭本总控 FR

---

## 总体验收标准

以下条件全部满足时，才可以关闭本总控 FR：

1. 子 FR1、FR2、FR3 全部关闭
2. `auth9-core` 业务层源码中不再直接依赖 `KeycloakClient`
3. `state` 不再通过 trait 暴露 `KeycloakClient`
4. `auth9-oidc` skeleton、backend flag、中性字段 migration 保持可用
5. QA 文档与 README 已完成对齐，且 QA 测试通过

---

## 非目标

- 本阶段不要求去掉 Keycloak 页面
- 本阶段不要求自研 token issuance
- 本阶段不要求迁移密码、MFA、社交登录完整闭环

---

## 关闭规则

- **不要直接在本总控 FR 上实现代码**
- 代码改动应落到对应子 FR
- 当子 FR 全部完成后：
  - 在本文件补一段完成记录，或
  - 直接删除本文件

---

## Closure Record

- **Date**: 2026-03-17
- **Status**: Closed
- **Fulfilled by**:
  - `docs/qa/integration/15-neutral-identity-schema-migration.md`
  - `docs/qa/integration/16-auth9-oidc-skeleton-and-backend-flag.md`
  - `docs/qa/integration/17-identity-engine-capabilities-state-cleanup.md`
  - `docs/qa/integration/18-business-layer-keycloak-decoupling.md`
  - `docs/qa/integration/19-phase1-identity-abstraction-closure.md`
- **Cross-doc alignment**:
  - `docs/qa/session/03-alerts.md`
  - `docs/qa/user/02-advanced.md`
  - `docs/qa/user/04-account-profile.md`
  - `docs/qa/identity-provider/02-toggle-validation.md`
  - `docs/security/input-validation/04-parameter-tampering.md`
  - `docs/security/data-security/01-sensitive-data.md`
- **Verification**:
  - `cargo test identity_engine -- --nocapture`
  - `cargo test --test keycloak_adapter_contract_test -- --nocapture`
  - `cargo test --test backend_switch_smoke_test -- --nocapture`
  - `./scripts/qa-doc-lint.sh`
- **Notes**:
  - Phase 1 QA 主语义已切换到 `identity_subject`、`provider_session_id`、`provider_alias`
  - 旧 `keycloak_*` 字段仅保留在 migration period、底层 Keycloak 兼容、或 Keycloak 专项验证文档中
