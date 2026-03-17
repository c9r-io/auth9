# Phase 5 FR3: Shadow Compare 与切换就绪性验证

**类型**: 上线工程 + 质量门禁
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core, observability, scripts, docs/qa, docs/security
**前置依赖**:
- `keycloak_phase5_fr1_dual_stack_rollout_controls.md`
- `keycloak_phase5_fr2_migration_and_diff_tooling.md`
**被依赖**:
- `keycloak_phase5_fr4_rollback_drills_and_operational_playbooks.md`
- `keycloak_phase5_fr5_keycloak_retirement_and_cleanup.md`

---

## 背景

有了双栈和迁移工具，还不能直接切流。必须先在正式切换前对关键认证链路做只读回放和结果比对，确认新旧后端在请求级别上没有不可接受的偏差。

---

## 期望行为

### R1: 对关键链路做 shadow compare

至少覆盖：

- password login
- refresh
- userinfo
- social login callback
- enterprise OIDC callback

### R2: 差异可追踪到请求级

要求：

- 新旧后端结果差异可关联到请求级日志
- 能定位到输入、输出和失败原因

### R3: 建立切换门禁

要求：

- 明确什么差异可接受、什么差异阻断切流
- 形成切换 readiness checklist

### R4: 结果面向运维和安全可读

要求：

- shadow compare 结果可用于运维决策
- 安全关键差异需单独标记

---

## 非目标

- 本 FR 不要求正式把流量切到 100%
- 本 FR 不要求删除 Keycloak fallback
- 本 FR 不要求直接修改生产数据

---

## 验证方法

```bash
rg -n "shadow compare|compare|replay|readiness|cutover checklist" scripts docs auth9-core
```

手动验证：

1. 关键认证链路可以在不影响主流量的前提下做只读比对
2. 差异项能追溯到请求级日志
3. 可形成明确的“允许切流 / 暂停切流”结论
