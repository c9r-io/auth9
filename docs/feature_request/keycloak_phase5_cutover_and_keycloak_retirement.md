# Phase 5: 灰度切换、数据迁移与 Keycloak 退役

**类型**: 上线工程 + 架构收尾
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core, deploy, observability, scripts, docs
**前置依赖**:
- `keycloak_phase1_identity_engine_abstraction.md`
- `keycloak_phase2_hosted_login_and_session_frontend.md`
- `keycloak_phase3_local_credentials_and_mfa.md`
- `keycloak_phase4_external_identity_broker.md`

---

## 背景

完成功能替换后，最危险的部分不是开发，而是生产切换。身份系统切换涉及 token、session、credential、federation mapping、client metadata、审计事件和回滚策略，必须单独作为一个 FR 管理。

---

## 期望行为

### R1: 双栈运行

生产环境支持两套身份后端并行：

- `backend=keycloak`
- `backend=auth9_oidc`

灰度维度至少支持：

- 环境级
- tenant 级
- client 级
- 百分比级

### R2: 数据迁移工具

提供迁移与比对工具，至少覆盖：

- users
- credentials
- sessions
- linked identities
- OIDC clients
- enterprise connectors

要求：

- 支持 dry-run
- 支持 idempotent 重跑
- 支持差异报告导出

### R3: 只读回放与结果比对

在正式切换前，对关键认证链路做 shadow compare：

- password login
- refresh
- userinfo
- social login callback
- enterprise OIDC callback

要求：

- 记录新旧后端结果差异
- 不一致项必须可追踪到请求级日志

### R4: 回滚策略

要求：

- 任一阶段切流后可快速切回 Keycloak
- 回滚不得依赖人工修改数据库
- 回滚开关必须文档化并经过演练

### R5: 退役 Keycloak

当以下条件全部满足时，允许退役：

- 运行时流量 100% 不再依赖 Keycloak
- 最近一个完整发布周期无 Keycloak fallback
- 所有监控、报警、运维脚本已切换
- 所有 QA/security/uiux 文档已更新

退役内容：

- 删除 Keycloak Deployment / StatefulSet / DB
- 删除 Keycloak theme 构建链路
- 删除 Keycloak event SPI 依赖
- 删除 `deploy` 中 Keycloak 相关配置项

### R6: 文档与运维收尾

要求：

- 更新架构图
- 更新部署文档
- 更新故障排查手册
- 更新 onboarding 文档
- 为 `auth9-oidc` 建立独立监控面板和告警

---

## 非目标

- 本阶段不要求保留对 Keycloak 数据结构的长期兼容
- 本阶段不要求迁移已过期 session / token 的历史存量

---

## 验证方法

```bash
rg -n "keycloak" deploy scripts docs auth9-core auth9-oidc
```

切换验证：

1. 灰度 1% 用户到 `auth9_oidc`
2. 观察 24h 无异常
3. 扩大至单 tenant / 单 client
4. 扩大至 100%
5. 关闭 Keycloak 运行时依赖后回归所有认证主路径

