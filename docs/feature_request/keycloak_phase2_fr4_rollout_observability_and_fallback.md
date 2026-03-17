# Phase 2 FR4: 灰度回滚、性能观测与兼容 fallback

**类型**: 上线治理 + 可观测性
**严重程度**: High
**影响范围**: auth9-core (Backend), auth9-portal (Frontend), deploy, docs/qa
**前置依赖**:
- `keycloak_phase2_fr1_hosted_login_routes_and_branding.md`
- `keycloak_phase2_fr2_hosted_login_api.md`
- `keycloak_phase2_fr3_browser_session_and_refresh_chain.md`
**被依赖**:
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

Phase 2 涉及认证入口、session 和 refresh 主链路，属于高风险变更。即便功能做完，也不能直接“一刀切”替换默认登录模式，必须具备可观测、可灰度、可快速回滚的运行方案。

---

## 期望行为

### R1: 增加 Hosted Login 性能指标

至少新增两组观测：

1. 登录首屏性能
2. 登录完成端到端耗时

对比维度至少包括：

- `mode=keycloak_hosted`
- `mode=auth9_hosted`

要求：

- 能在上线后证明用户感知延迟下降
- 指标能按环境和时间窗口分析

### R2: 支持灰度开关

要求：

- 支持按环境切换
- 支持按 tenant 切换
- 支持按百分比切换

至少要能控制：

- 是否默认进入 Auth9 Hosted Login
- 是否回退到 Keycloak hosted login

### R3: 回滚不依赖数据迁移

要求：

- 回滚到 Keycloak hosted login 时，不需要额外数据迁移
- 回滚过程不要求清理新 session 数据或前端路由数据
- 出现严重故障时可快速恢复旧主路径

### R4: QA 与运行文档同步

要求：

- 补齐 QA 文档中的 hosted login / fallback / rollout 场景
- 更新运维或上线说明，明确开关位置、观测指标和回滚步骤

---

## 非目标

- 本 FR 不要求新增新的认证能力
- 本 FR 不要求移除 Keycloak fallback
- 本 FR 不要求完成最终 Keycloak retirement

---

## 验证方法

```bash
rg -n "auth9_hosted|keycloak_hosted|hosted login|fallback|rollout|tenant rollout|percentage rollout" auth9-core auth9-portal deploy docs
cd auth9-portal && npm run build
cd auth9-core && cargo build
```

手动验证：

1. 可按配置切到 Auth9 Hosted Login
2. 可按配置快速回退到 Keycloak hosted login
3. 指标面板或日志中可区分 `auth9_hosted` 与 `keycloak_hosted`
4. 回滚后无需做数据库迁移即可恢复旧路径
