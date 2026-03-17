# Phase 1 FR4: `auth9-oidc` 服务骨架与身份后端开关

**类型**: 架构演进
**严重程度**: High
**影响范围**: auth9-core (Backend), 新增 `auth9-oidc` 服务骨架, docker-compose, deploy, tests
**前置依赖**:
- `keycloak_phase1_fr1_identity_engine_interfaces_and_state_injection.md`
- `keycloak_phase1_fr2_keycloak_adapter_layer.md`
**被依赖**:
- `keycloak_phase2_hosted_login_and_session_frontend.md`
- `keycloak_phase3_local_credentials_and_mfa.md`
- `keycloak_phase4_external_identity_broker.md`
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

有了抽象接口和 Keycloak adapter 之后，还需要一个真正的“第二实现入口”，否则整体架构仍然只有单后端。

Phase 1 不要求 `auth9-oidc` 已实现完整认证闭环，但必须有一个最小可启动、可注入、可被 feature flag 选择的服务骨架，作为后续替换承载体。

---

## 期望行为

### R1: 新增独立 `auth9-oidc` 服务目录

新增 `auth9-oidc/`，至少具备：

- 独立 `Cargo.toml`
- 独立配置结构
- 可启动的最小 HTTP 服务
- `GET /health`
- 最小数据库连接

要求：

- 这是未来替换 Keycloak 的承载体
- 不把全部逻辑继续塞回 `auth9-core`

### R2: 提供最小 `IdentityEngine` stub 实现

要求：

- `auth9-oidc` 中至少存在一套最小 `IdentityEngine` 实现骨架
- 允许返回 `not implemented`、空结果或 stub 数据
- 但构造链必须完整，后续可以持续填充真实逻辑

### R3: 新增身份后端选择开关

新增：

```yaml
IDENTITY_BACKEND=keycloak|auth9_oidc
```

要求：

- 默认仍为 `keycloak`
- dev/test 可切到 `auth9_oidc`
- 启动日志必须打印当前 backend
- `auth9-core` 能基于该开关注入不同 backend

**涉及文件**:
- `auth9-core/src/config/mod.rs`
- `auth9-core/src/server/mod.rs`

### R4: 本地与部署配置补齐占位

要求：

- `docker-compose.yml` 中新增 `auth9-oidc` 组件占位
- `deploy/k8s/` 中新增最小 deployment / service 占位
- 不要求本阶段完成完整生产参数或灰度策略

### R5: 双 backend smoke test

要求：

- 默认 `keycloak` backend 启动成功
- `auth9_oidc` backend 注入成功
- 至少覆盖 health / config / state wiring 的 smoke test

---

## 非目标

- 本 FR 不要求完整的 token issuance
- 本 FR 不要求 Hosted Login 页面
- 本 FR 不要求本地密码、MFA、federation 全量实现
- 本 FR 不要求 `auth9-core` 通过网络 RPC 调用 `auth9-oidc`

---

## 验证方法

```bash
rg -n "IDENTITY_BACKEND|auth9_oidc|Current identity backend" auth9-core auth9-oidc docker-compose.yml deploy/k8s
cd auth9-oidc && cargo build
cd auth9-oidc && cargo test
cd auth9-core && cargo test backend_switch
docker compose config >/tmp/auth9-compose-check.yaml
```

手动验证：

1. 使用默认配置启动，日志显示 `keycloak`
2. 使用 `IDENTITY_BACKEND=auth9_oidc` 启动，`auth9-core` 构造链仍可完成
3. `auth9-oidc` 的 `/health` 返回成功
