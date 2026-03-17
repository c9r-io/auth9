# Phase 5 FR4: 回滚演练与运维手册

**类型**: 上线工程 + 运维治理
**严重程度**: High
**影响范围**: deploy, scripts, docs, observability, oncall 手册
**前置依赖**:
- `keycloak_phase5_fr1_dual_stack_rollout_controls.md`
- `keycloak_phase5_fr2_migration_and_diff_tooling.md`
- `keycloak_phase5_fr3_shadow_compare_and_cutover_readiness.md`
**被依赖**:
- `keycloak_phase5_fr5_keycloak_retirement_and_cleanup.md`

---

## 背景

身份系统切流最危险的不是“不能切”，而是“切了回不去”。因此回滚能力必须在 Keycloak 退役前独立治理，并通过演练和文档化固化下来。

---

## 期望行为

### R1: 任一阶段切流后可快速切回 Keycloak

要求：

- 回滚不得依赖人工修改数据库
- 回滚开关必须明确且稳定

### R2: 回滚流程文档化

要求：

- 文档说明开关位置、执行顺序、观测项和回滚后的验证项
- oncall / 运维可以按手册执行

### R3: 至少完成一次真实演练

要求：

- 回滚路径经过演练
- 演练结果有记录

### R4: 故障处置手册同步

要求：

- 更新故障排查手册
- 更新发布/回滚 SOP

---

## 非目标

- 本 FR 不要求删除 Keycloak 资源
- 本 FR 不要求完成全部文档清理
- 本 FR 不要求切流已达到 100%

---

## 验证方法

```bash
rg -n "rollback|playbook|runbook|drill|cutover" deploy scripts docs
```

手动验证：

1. 从 `auth9_oidc` 灰度切回 Keycloak 不需要改数据库
2. 按文档执行可完成回滚
3. 回滚后关键认证主路径恢复正常
