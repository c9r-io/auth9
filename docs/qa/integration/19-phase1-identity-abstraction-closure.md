# 集成测试 - Phase 1 身份抽象层 Closure

**模块**: 集成测试
**测试范围**: `keycloak` 默认 backend、`auth9_oidc` stub backend、adapter 注入链、`identity_subject` / `provider_session_id` / `provider_alias` 中性字段主路径
**场景数**: 4
**优先级**: 高

---

## 背景说明

本用例用于对 Keycloak Phase 1 的身份抽象层改造做最终 closure 验收，确认以下目标已同时成立：

- 默认 `keycloak` backend 仍可完成注入与 contract 回归
- `auth9_oidc` stub backend 可完成最小 wiring，不因抽象层改造而 panic
- 业务层与 adapter 注入链保持在 `IdentityEngine` 抽象边界内
- QA 主断言字段已经切到 `identity_subject`、`provider_session_id`、`provider_alias`

本文件是总验收文档，不替代分项回归文档 `integration/15`、`integration/16`、`integration/17`、`integration/18`。

---

## 场景 1：默认 `keycloak` backend 启动与注入链正常

### 初始状态
- 本地依赖服务已启动
- `auth9-core` 可执行 `cargo run`

### 目的
验证默认 `IDENTITY_BACKEND=keycloak` 下，`session_store`、`federation_broker`、`identity_engine` 注入链正常。

### 测试操作流程
1. 检查容器环境中的 backend 配置：
   ```bash
   docker inspect auth9-core --format '{{range .Config.Env}}{{println .}}{{end}}' | rg '^IDENTITY_BACKEND=' || echo 'IDENTITY_BACKEND not set (defaults to keycloak)'
   ```
2. 检查健康探针：
   ```bash
   curl -sf http://localhost:8080/health
   ```

### 预期结果
- 输出 `IDENTITY_BACKEND not set (defaults to keycloak)`，或显式出现 `IDENTITY_BACKEND=keycloak`
- `curl /health` 返回 `200`
- 当前运行实例在默认 `keycloak` backend 下健康响应

---

## 场景 2：切换到 `auth9_oidc` stub backend 后最小 wiring 仍成立

### 步骤 0：验证 backend 配置

```bash
echo "${IDENTITY_BACKEND:-keycloak}"
# 预期: 如需测试本场景，应显式启动为 auth9_oidc；若输出不是 auth9_oidc，请按步骤 1 重启服务
```

### 初始状态
- Rust 依赖已安装
- 本地可执行 `cargo test`

### 目的
验证 `auth9_oidc` backend 仍具备最小 wiring 与 smoke test，不因 Phase 1 抽象化回归而失效。

### 测试操作流程
1. 执行 smoke test：
   ```bash
   cd auth9-core && cargo test --test backend_switch_smoke_test -- --nocapture
   ```
2. 如需手动启动验证，使用：
   ```bash
   IDENTITY_BACKEND=auth9_oidc cargo run --manifest-path auth9-core/Cargo.toml -- serve 2>&1 | tee /tmp/auth9-phase1-auth9-oidc.log
   ```

### 预期结果
- `backend_switch_smoke_test` 通过
- `IdentityBackend::Auth9Oidc` 分支可完成 `session_store`、`federation_broker`、`identity_engine` 注入
- 未实现操作返回显式错误，不出现 wiring panic

---

## 场景 3：adapter contract 与业务抽象边界保持稳定

### 初始状态
- Rust 依赖已安装
- 本地可执行 `cargo test` 与 `rg`

### 目的
验证 adapter contract 持续成立，且目标业务层文件不回退为直接依赖 `KeycloakClient`。

### 测试操作流程
1. 执行 contract 回归：
   ```bash
   cd auth9-core && cargo test identity_engine -- --nocapture
   cd auth9-core && cargo test --test keycloak_adapter_contract_test -- --nocapture
   ```
2. 扫描目标服务文件的非测试区域：
   ```bash
   for f in \
     auth9-core/src/domains/tenant_access/service/user.rs \
     auth9-core/src/domains/identity/service/password.rs \
     auth9-core/src/domains/identity/service/webauthn.rs \
     auth9-core/src/domains/provisioning/service/scim.rs \
     auth9-core/src/domains/tenant_access/service/saml_application.rs; do
     echo "FILE:$f"
     sed '/#\[cfg(test)\]/,$d' "$f" | rg -n "KeycloakClient|Arc<KeycloakClient>" || true
   done
   ```

### 预期结果
- `identity_engine` 与 `keycloak_adapter_contract_test` 通过
- 目标服务实现文件的非测试区域无 `KeycloakClient` 命中
- `keycloak` backend 仍通过 adapter 暴露中性 contract

---

## 场景 4：中性字段成为 QA 主断言路径

### 初始状态
- QA 文档已同步到本 FR 实现版本
- 本地可执行 `rg`

### 目的
验证文档与集成回归主路径已切换到 `identity_subject`、`provider_session_id`、`provider_alias`。

### 测试操作流程
1. 检查中性字段引用：
   ```bash
   rg -n "identity_subject|provider_session_id|provider_alias" docs/qa
   ```
2. 检查旧字段是否仅保留在 migration / Keycloak 兼容说明中：
   ```bash
   rg -n "keycloak_id|keycloak_session_id|keycloak_alias" docs/qa docs/security docs/uiux
   ```

### 预期结果
- `docs/qa` 中存在对 `identity_subject`、`provider_session_id`、`provider_alias` 的主路径断言
- `docs/uiux` 无旧字段引用
- 旧字段仅出现在 migration period、底层 Keycloak 兼容、或专门验证 Keycloak 集成行为的文档中

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | 默认 `keycloak` backend 启动与注入链正常 | ☑ | 2026-03-17 | Codex | `docker inspect` 未发现 `IDENTITY_BACKEND` 覆盖，默认回落 `keycloak`；`/health` 返回 healthy |
| 2 | 切换到 `auth9_oidc` stub backend 后最小 wiring 仍成立 | ☑ | 2026-03-17 | Codex | `cargo test --test backend_switch_smoke_test -- --nocapture` 2/2 通过 |
| 3 | adapter contract 与业务抽象边界保持稳定 | ☑ | 2026-03-17 | Codex | `identity_engine` 与 `--test keycloak_adapter_contract_test` 通过；非测试区域扫描无 `KeycloakClient` 命中 |
| 4 | 中性字段成为 QA 主断言路径 | ☑ | 2026-03-17 | Codex | `rg` 确认 `identity_subject` / `provider_session_id` / `provider_alias` 已成为 QA 主断言字段 |
