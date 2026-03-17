# Phase 5 FR2: 数据迁移与差异比对工具

**类型**: 上线工程 + 数据治理
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core, scripts, docs
**前置依赖**:
- `keycloak_phase5_fr1_dual_stack_rollout_controls.md`
- `keycloak_phase3_fr1_local_credential_store.md`
- `keycloak_phase4_fr4_federated_identity_linking.md`
**被依赖**:
- `keycloak_phase5_fr3_shadow_compare_and_cutover_readiness.md`
- `keycloak_phase5_fr4_rollback_drills_and_operational_playbooks.md`
- `keycloak_phase5_fr5_keycloak_retirement_and_cleanup.md`

---

## 背景

切换身份后端前，必须先有迁移和比对工具，否则无法证明 `auth9_oidc` 与现有 Keycloak 承载的数据一致，也无法安全重跑迁移。

---

## 期望行为

### R1: 提供迁移工具

至少覆盖：

- users
- credentials
- sessions
- linked identities
- OIDC clients
- enterprise connectors

### R2: 工具具备安全重跑特性

要求：

- 支持 dry-run
- 支持 idempotent 重跑
- 支持部分模块单独迁移

### R3: 差异报告可导出

要求：

- 支持导出差异报告
- 差异至少可按对象类型和对象 ID 检索

### R4: 迁移日志可审计

要求：

- 每次迁移执行有运行记录
- 差异修复过程可追踪

---

## 非目标

- 本 FR 不要求正式切流
- 本 FR 不要求删除历史 Keycloak 数据
- 本 FR 不要求迁移已过期 session / token 的全部历史存量

---

## 验证方法

```bash
rg -n "dry-run|idempotent|diff|backfill|migration report" scripts auth9-core docs
```

手动验证：

1. 同一批数据迁移可重复执行且结果稳定
2. dry-run 不写入数据但能产出差异报告
3. 可针对单个模块导出差异明细
