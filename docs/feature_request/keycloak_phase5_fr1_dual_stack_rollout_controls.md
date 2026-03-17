# Phase 5 FR1: 双栈运行与灰度切流控制

**类型**: 上线工程
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core, deploy, observability, docs
**前置依赖**:
- `keycloak_phase1_fr4_auth9_oidc_skeleton_and_backend_flag.md`
- `keycloak_phase3_fr5_token_issuance_and_auth_server_core.md`
- `keycloak_phase4_fr5_federation_audit_and_security_events.md`
**被依赖**:
- `keycloak_phase5_fr2_migration_and_diff_tooling.md`
- `keycloak_phase5_fr3_shadow_compare_and_cutover_readiness.md`
- `keycloak_phase5_fr4_rollback_drills_and_operational_playbooks.md`
- `keycloak_phase5_fr5_keycloak_retirement_and_cleanup.md`

---

## 背景

在真正迁移和切流前，生产环境必须先支持双栈运行。否则后续 shadow compare、灰度验证和回滚都没有稳定落点。

---

## 期望行为

### R1: 生产支持两套身份后端并行

至少支持：

- `backend=keycloak`
- `backend=auth9_oidc`

### R2: 灰度维度可配置

至少覆盖：

- 环境级
- tenant 级
- client 级
- 百分比级

### R3: 路由决策可观测

要求：

- 每次认证请求都能记录实际命中的 backend
- 路由命中结果能进入日志和指标

### R4: 配置方式适合生产运维

要求：

- 灰度开关可在 deploy / runtime 配置中显式控制
- 不依赖手工改代码

---

## 非目标

- 本 FR 不要求做数据迁移
- 本 FR 不要求做 shadow compare
- 本 FR 不要求删除 Keycloak 任何部署资源

---

## 验证方法

```bash
rg -n "backend=keycloak|backend=auth9_oidc|rollout|tenant rollout|client rollout|percentage rollout" auth9-core deploy docs
```

手动验证：

1. 可按环境切到 `auth9_oidc`
2. 可按 tenant / client / 百分比路由到不同 backend
3. 日志中可看到请求实际命中的 backend
