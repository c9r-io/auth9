# Phase 4 FR5: 联邦审计与安全事件

**类型**: 安全与可观测性
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core (Backend), docs/qa, docs/security, analytics, observability
**前置依赖**:
- `keycloak_phase4_fr1_social_login_broker.md`
- `keycloak_phase4_fr2_enterprise_oidc_connector.md`
- `keycloak_phase4_fr3_enterprise_saml_broker.md`
- `keycloak_phase4_fr4_federated_identity_linking.md`
**被依赖**:
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

当前 social/federation 事件仍部分依赖 Keycloak event SPI webhook 或 Keycloak 事件语义。如果 Phase 4 完成后主 broker 已切到 Auth9，但事件面仍靠 Keycloak，观测、审计和安全检测会出现断层。

---

## 期望行为

### R1: Auth9 自己产出联邦事件

至少覆盖：

- login success / failure
- idp timeout
- invalid issuer / audience
- invalid assertion
- account link / unlink

### R2: 不再依赖 Keycloak event SPI 作为主事件源

要求：

- Keycloak event SPI 可以在迁移期保留兼容
- 但 Auth9 自己产出的事件应成为主事件源

### R3: 事件可用于审计与检测

要求：

- 安全观测、analytics、审计日志可消费新的联邦事件
- 失败原因和 provider 维度可被检索与统计

### R4: QA 与安全文档同步

要求：

- 更新联邦登录、账号绑定、异常 IdP 响应等场景的 QA 文档
- 更新安全测试文档，覆盖 invalid issuer / audience / assertion / timeout 路径

---

## 非目标

- 本 FR 不要求新增新的 broker 能力
- 本 FR 不要求立即删除全部 Keycloak webhook 兼容代码
- 本 FR 不要求完成全部监控面板重构

---

## 验证方法

```bash
cd auth9-core && cargo test analytics
cd auth9-core && cargo test federation
cd auth9-oidc && cargo test federation_events
```

手动验证：

1. social / enterprise 登录成功失败事件由 Auth9 自己产出
2. IdP 超时、非法 assertion、非法 issuer / audience 会进入 Auth9 审计与检测链路
3. 不再依赖 Keycloak event SPI webhook 才能得到主联邦事件
