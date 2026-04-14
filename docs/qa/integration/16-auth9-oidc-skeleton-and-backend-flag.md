# 集成测试 - auth9-oidc 骨架与 backend 开关（已归档）

**状态**: ✅ 已归档 — 2026-04-15
**模块**: 集成测试
**场景数**: 0（全部归档）

---

## 归档说明

本用例的两个场景在 OIDC 引擎收敛回 auth9-core 后均已失效，故整体归档：

- **场景 1（auth9-core 使用 auth9_oidc 后端启动成功）**: `IDENTITY_BACKEND` 环境变量与 `Identity backend: auth9_oidc` 启动日志已移除。OIDC 引擎现在固定内嵌于 auth9-core，无 backend 概念，无需断言。
- **场景 2（auth9-oidc 独立服务 /health 返回成功）**: auth9-oidc 独立 crate / Pod / 镜像已删除，无独立 health 端点可测。

### 历史背景

早期阶段为脱离 Keycloak、为将来可能的服务拆分预留边界，曾把 OIDC 相关数据模型与 health skeleton 放在独立的 `auth9-oidc` crate（仅暴露 `/health`，与 auth9-core 共享同一 TiDB 数据库）。skeleton 没有任何协议端点，所有 OIDC 协议代码始终在 auth9-core 内。

经评估「半拆分」状态成本大于收益（双部署物 + 双镜像 + 双监控，零隔离收益），将 models / repository 合回 `auth9-core/src/identity_engine/` 下，删除独立 crate 与 K8s manifest。`IdentityEngine` trait 抽象保留，便于未来真正需要替换实现时仍可插拔。

### 替代验证

- 协议端点功能 — 见 `docs/qa/auth/`（authorize / token / userinfo / discovery 等）
- 凭据 / pending action / 邮箱验证存储 — 见 `docs/qa/integration/20-local-credential-store.md`
- 身份引擎注入链路 — 见 `docs/qa/integration/13-identity-engine-state-injection.md`、`docs/qa/integration/17-identity-engine-capabilities-state-cleanup.md`
