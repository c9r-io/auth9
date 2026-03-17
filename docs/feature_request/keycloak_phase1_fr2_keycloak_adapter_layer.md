# Phase 1 FR2: Keycloak Adapter 层

**类型**: 架构演进
**严重程度**: High
**影响范围**: auth9-core (Backend), tests
**前置依赖**: `keycloak_phase1_fr1_identity_engine_interfaces_and_state_injection.md`
**被依赖**:
- `keycloak_phase1_fr4_auth9_oidc_skeleton_and_backend_flag.md`

---

## 背景

完成抽象接口后，仍然需要一个现役实现来承接当前生产能力。Phase 1 不应直接删除 `KeycloakClient`，而应把它降级为“adapter 底座”。

这样可以同时满足两点：

1. 现有 Keycloak 功能继续可用
2. 业务层不再感知 Keycloak 专有实现

---

## 期望行为

### R1: Keycloak 迁移为抽象实现

保留现有 `KeycloakClient`，但新增 adapter 层实现统一接口：

- `KeycloakUserStoreAdapter`
- `KeycloakClientStoreAdapter`
- `KeycloakSessionStoreAdapter`
- `KeycloakCredentialStoreAdapter`
- `KeycloakFederationBrokerAdapter`
- 必要时提供聚合 `KeycloakIdentityEngineAdapter`

要求：

- adapter 文件位于 `auth9-core/src/identity_engine/adapters/keycloak/`
- adapter 对外暴露中性 trait 能力，而不是 Keycloak 语义方法

### R2: `KeycloakClient` 只作为底层 HTTP client

要求：

- 原 `KeycloakClient` 继续存在
- `KeycloakClient` 不再是业务层构造函数的直接依赖
- 若业务层需要 Keycloak 能力，只能通过 adapter 注入

**涉及文件**:
- `auth9-core/src/keycloak/`
- `auth9-core/src/identity_engine/adapters/keycloak/`

### R3: 类型映射收敛在 adapter 层

要求：

- Keycloak 特有的请求/响应结构与中性领域结构之间的转换集中放在 adapter
- 不把 `KeycloakIdentityProvider`、`KeycloakSession` 之类类型再扩散进新的业务层依赖

### R4: 补齐 adapter contract tests

要求：

- 至少覆盖 session revoke、identity provider CRUD、federated identity 读取/删除等已使用路径
- 测试应验证 adapter 的行为契约，而不是只验证 `KeycloakClient` 本身

---

## 非目标

- 本 FR 不要求实现 `auth9-oidc` backend
- 本 FR 不要求删掉所有遗留 API handler 中的 Keycloak 引用
- 本 FR 不要求把 `auth9-core/src/keycloak/` 拆成独立 crate

---

## 验证方法

```bash
rg -n "Keycloak(UserStore|ClientStore|SessionStore|CredentialStore|FederationBroker)Adapter" auth9-core/src
rg -n "KeycloakClient" auth9-core/src/domains/identity/service auth9-core/src/domains/platform/service
cd auth9-core && cargo test keycloak_adapter
cd auth9-core && cargo test identity_engine
```

手动验证：

1. 以默认 Keycloak backend 启动 `auth9-core`
2. 现有依赖 Keycloak 的服务路径仍可正常执行
3. 启动日志或调试日志中可以确认实际注入的是 Keycloak adapter
