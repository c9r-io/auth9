# Phase 4 FR2: Enterprise OIDC Connector

**类型**: 核心能力替换
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core (Backend), auth9-portal (Frontend), tenant access, provisioning, tests
**前置依赖**:
- `keycloak_phase1_fr3_neutral_model_schema.md`
- `keycloak_phase2_fr1_hosted_login_routes_and_branding.md`
- `keycloak_phase3_fr5_token_issuance_and_auth_server_core.md`
**被依赖**:
- `keycloak_phase4_fr4_federated_identity_linking.md`
- `keycloak_phase4_fr5_federation_audit_and_security_events.md`

---

## 背景

当前 enterprise OIDC connector 的真实承载仍是 Keycloak identity provider，域名发现后最终也是把请求导向 Keycloak。要让 Auth9 成为真正的 broker，Enterprise OIDC 需要独立成为一条完整的配置与运行时链路。

---

## 期望行为

### R1: 重建企业 OIDC connector 模型

至少覆盖：

- issuer
- client_id / secret
- authorize endpoint
- token endpoint
- userinfo endpoint
- jwks endpoint
- claim mapping

### R2: Domain discovery 直接返回 Auth9 broker redirect

要求：

- 域名发现后直接进入 Auth9 broker redirect
- 不再依赖 `kc_idp_hint` 或 Keycloak alias 透传
- `enterprise_sso_connectors` 中的 `keycloak_alias` 彻底退场

### R3: Runtime 能处理 enterprise OIDC 细节

至少覆盖：

- login hint / domain hint / organization hint
- callback 处理
- token / userinfo 拉取
- claim mapping 到 Auth9 用户模型

### R4: Tenant access 与运维能力保持可用

要求：

- 租户侧 connector CRUD、test connector、SCIM 等配套能力不因 broker 重构而丢失
- Portal 管理界面继续可用

---

## 非目标

- 本 FR 不要求处理 enterprise SAML
- 本 FR 不要求实现 linked identity 冲突合并策略
- 本 FR 不要求支持所有 OIDC provider 厂商特有扩展

---

## 验证方法

```bash
cd auth9-core && cargo test enterprise_sso
cd auth9-oidc && cargo test oidc_connector
cd auth9-portal && npm run test
```

手动验证：

1. 域名发现后跳转到 Auth9 broker 流程，而不是 Keycloak broker
2. 新建 enterprise OIDC connector 时不再写入 Keycloak alias 语义
3. 登录成功后 claim mapping 由 Auth9 自己执行
