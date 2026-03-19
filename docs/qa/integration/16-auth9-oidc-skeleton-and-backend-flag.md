# 集成测试 - auth9-oidc 服务骨架与 backend flag

**模块**: 集成测试
**测试范围**: `auth9-oidc` 独立服务骨架、`IDENTITY_BACKEND` 配置开关、`auth9-core` 注入链 smoke test
**场景数**: 4
**优先级**: 高

---

## 背景说明

> **迁移已完成**: `IDENTITY_BACKEND` 开关已移除，auth9-oidc 现在是唯一后端。以下为历史迁移验证记录。

本用例用于验证 Phase 1 FR4 完成后的关键回归点：

- 仓库新增独立 `auth9-oidc` 服务目录
- `auth9-core` 已固定使用 `auth9_oidc` 后端（`IDENTITY_BACKEND` 开关已移除）
- State wiring、Session/Federation 注入链完整

该阶段验证 auth9-oidc 基础骨架。

---

## 场景 1：默认 `keycloak` backend 启动成功

### 初始状态
- `auth9-core`、`auth9-oidc`、`auth9-redis`、`auth9-tidb` 已启动
- `IDENTITY_BACKEND` 开关已移除（auth9_oidc 为唯一后端）

### 目的
验证默认配置仍走 `keycloak`，且不影响现有服务启动。

### 测试操作流程
1. 查看 `auth9-core` 最近日志：
   ```bash
   docker logs auth9-core --tail 200
   ```
2. 调用健康检查：
   ```bash
   curl -sS http://localhost:8080/health
   ```

### 预期结果
- 日志中出现 `Current identity backend: keycloak`
- `/health` 返回 `200 OK`
- 不出现 `invalid IDENTITY_BACKEND`、`missing identity engine`、`panic`

---

## 场景 2：切换到 `auth9_oidc` backend 后 `auth9-core` 仍可完成注入

#### 步骤 0：以 `auth9_oidc` backend 启动

```bash
IDENTITY_BACKEND=auth9_oidc cargo run --manifest-path auth9-core/Cargo.toml -- serve 2>&1 | tee /tmp/auth9-core.log
```

### 初始状态
- TiDB、Redis 已启动
- 已完成步骤 0

### 目的
验证 backend flag 切换后，`auth9-core` 仍能完成最小注入链构造。

### 测试操作流程
1. 查看启动日志：
   ```bash
   tail -n 200 /tmp/auth9-core.log
   ```
2. 调用健康检查：
   ```bash
   curl -sS http://localhost:8080/health
   ```

### 预期结果
- 日志中出现 `Current identity backend: auth9_oidc`
- `/health` 返回 `200 OK`
- 不出现 `missing federation broker`、`missing session store`、`service not found`

---

## 场景 3：`auth9-oidc` 独立服务 `/health` 返回成功

### 初始状态
- `auth9-oidc` 已启动
- TiDB 可访问

### 目的
验证独立服务骨架可启动、可连 DB、可对外提供 health probe。

### 测试操作流程
1. 启动服务：
   ```bash
   cargo run --manifest-path auth9-oidc/Cargo.toml 2>&1 | tee /tmp/auth9-oidc.log
   ```
2. 调用 health 端点：
   ```bash
   curl -sS http://localhost:8090/health
   ```

### 预期结果
- 返回 `200 OK`
- 响应 JSON 包含 `service = "auth9-oidc"`
- 响应 JSON 包含 `identity_backend = "auth9_oidc"`
- 响应 JSON 包含 `database = "up"`

---

## 场景 4：非法 backend 配置会快速失败

### 初始状态
- 本地 shell 可直接运行 `auth9-core`

### 目的
验证 `IDENTITY_BACKEND` 具有显式校验，避免静默落到错误分支。

### 测试操作流程
1. 使用非法 backend 启动：
   ```bash
   IDENTITY_BACKEND=unknown cargo run --manifest-path auth9-core/Cargo.toml -- serve
   ```

### 预期结果
- 进程启动失败
- 错误输出包含 `Invalid IDENTITY_BACKEND`
- 不进入监听状态

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | 默认 `keycloak` backend 启动成功 | ☐ | | | |
| 2 | 切换到 `auth9_oidc` backend 后 `auth9-core` 仍可完成注入 | ☐ | | | |
| 3 | `auth9-oidc` 独立服务 `/health` 返回成功 | ☐ | | | |
| 4 | 非法 backend 配置会快速失败 | ☐ | | | |
