# Phase 3 FR1: 本地 Credential Store

**类型**: 核心能力替换
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core (Backend), migrations, tests
**前置依赖**:
- `keycloak_phase1_fr1_identity_engine_interfaces_and_state_injection.md`
- `keycloak_phase1_fr2_keycloak_adapter_layer.md`
- `keycloak_phase1_fr4_auth9_oidc_skeleton_and_backend_flag.md`
**被依赖**:
- `keycloak_phase3_fr2_password_and_reset_flow.md`
- `keycloak_phase3_fr3_email_verification_and_required_actions.md`
- `keycloak_phase3_fr4_local_mfa.md`
- `keycloak_phase3_fr5_token_issuance_and_auth_server_core.md`

---

## 背景

只要第一方账号的 credential 仍由 Keycloak 存储，Auth9 就没有真正拿回认证内核。Phase 3 的第一步必须先建立中性的本地 credential store，为后续 password、MFA、required actions 和 token issuance 提供统一数据基础。

---

## 期望行为

### R1: 新增中性凭据模型

至少覆盖以下能力：

- password hash
- TOTP secret
- recovery codes
- WebAuthn credentials
- email verification status
- required actions / pending actions

要求：

- 数据模型不出现 `keycloak_*` 语义
- 设计能兼容后续新增认证因子

### R2: `auth9-oidc` 落地持久化存储

要求：

- 在 `auth9-oidc/migrations/` 中新增对应表结构
- 在 `auth9-oidc` 中实现 credential repository / domain 层骨架
- `auth9-core` 通过对接层消费，而不是直接操作 Keycloak credentials API

### R3: 迁移期可并存

要求：

- 允许迁移期内 Keycloak credential 与本地 credential 并存
- 新增本地模型不得破坏现有运行路径
- 为后续双写、回填或灰度提供兼容空间

### R4: Credential contract tests

要求：

- 覆盖 create/read/update/delete 与状态切换
- 覆盖 pending actions、email verification status 和 recovery codes 的基本约束

---

## 非目标

- 本 FR 不要求实现密码验证逻辑
- 本 FR 不要求实现 MFA challenge 流程
- 本 FR 不要求实现 token issuance
- 本 FR 不要求完成生产迁移或清理 Keycloak 中的 credential 数据

---

## 验证方法

```bash
cd auth9-oidc && cargo test credential
cd auth9-oidc && cargo build
rg -n "password_hash|recovery_codes|pending_actions|email_verification" auth9-oidc
```

手动验证：

1. `auth9-oidc` migration 后存在中性 credential 表
2. `auth9-core` 不需要直接调用 Keycloak credentials API 即可访问本地 credential store
