# Phase 5 FR5: Keycloak 退役与清理收尾

**类型**: 架构收尾 + 运维清理
**严重程度**: High
**影响范围**: deploy, docker-compose, `auth9-keycloak-theme`, `auth9-keycloak-events`, docs, scripts, observability
**前置依赖**:
- `keycloak_phase5_fr1_dual_stack_rollout_controls.md`
- `keycloak_phase5_fr2_migration_and_diff_tooling.md`
- `keycloak_phase5_fr3_shadow_compare_and_cutover_readiness.md`
- `keycloak_phase5_fr4_rollback_drills_and_operational_playbooks.md`
**被依赖**: 无

---

## 背景

只有在运行时流量、回滚演练、监控告警和文档都准备完毕后，才能进入最终 Keycloak 退役。退役不是简单删 Deployment，而是整个工程面和文档面的收尾动作。

---

## 期望行为

### R1: 满足退役前置条件

至少包括：

- 运行时流量 100% 不再依赖 Keycloak
- 最近一个完整发布周期无 Keycloak fallback
- 所有监控、报警、运维脚本已切换
- 所有 QA/security/uiux 文档已更新

### R2: 删除 Keycloak 运行时依赖

至少覆盖：

- 删除 Keycloak Deployment / StatefulSet / DB
- 删除 `deploy` 中 Keycloak 相关配置项
- 删除 docker-compose 中 Keycloak 运行时依赖

### R3: 删除 Keycloak 辅助构建链路

至少覆盖：

- 删除 Keycloak theme 构建链路
- 删除 Keycloak event SPI 依赖

### R4: 文档与监控收尾

要求：

- 更新架构图
- 更新部署文档
- 更新 onboarding 文档
- 为 `auth9-oidc` 建立独立监控面板和告警

---

## 非目标

- 本 FR 不要求长期保留 Keycloak 数据结构兼容
- 本 FR 不要求保留 Keycloak 相关开发链路

---

## 验证方法

```bash
rg -n "keycloak" deploy docker-compose.yml scripts docs auth9-core auth9-keycloak-theme auth9-keycloak-events
```

手动验证：

1. 关闭 Keycloak 后，认证主路径仍全部可用
2. 部署与运维脚本中不再依赖 Keycloak
3. QA/security/uiux/架构文档已同步到 `auth9-oidc` 新架构
