# Phase 4 FR3: Enterprise SAML Broker

**类型**: 核心能力替换
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core (Backend), auth9-portal (Frontend), tests, docs/qa/auth, docs/security
**前置依赖**:
- `keycloak_phase2_fr1_hosted_login_routes_and_branding.md`
- `keycloak_phase3_fr5_token_issuance_and_auth_server_core.md`
**被依赖**:
- `keycloak_phase4_fr4_federated_identity_linking.md`
- `keycloak_phase4_fr5_federation_audit_and_security_events.md`

---

## 背景

Enterprise SAML broker 与 OIDC broker 不是同一类实现问题。它有独立的协议风险、签名校验、RelayState、NameID/Attribute mapping 复杂度，因此必须单独治理，不能和 enterprise OIDC 混成一个 FR。

---

## 期望行为

### R1: Auth9 自己提供 SAML broker 最小能力

至少覆盖：

- SP metadata 生成
- AuthnRequest 发起
- Response 校验
- NameID / Attribute mapping
- RelayState 防篡改

### R2: 支持最小企业 SAML 登录场景

要求：

- 允许先覆盖 MVP 场景
- 不要求一开始支持全部 SLO 变体
- 登录成功后能进入 Auth9 本地 session / token issuance 流程

### R3: SAML 安全校验由 Auth9 自己完成

至少覆盖：

- issuer 校验
- audience 校验
- assertion 签名校验
- 时间窗口校验

### R4: Portal 与 connector 管理对接

要求：

- 租户侧可配置企业 SAML connector
- Portal 登录页可驱动企业 SAML 登录入口

---

## 非目标

- 本 FR 不要求完整通用 SAML 平台能力
- 本 FR 不要求支持全部 SLO / artifact binding 变体
- 本 FR 不要求处理 linked identity 自动合并策略

---

## 验证方法

```bash
cd auth9-core && cargo test saml
cd auth9-oidc && cargo test saml
cd auth9-portal && npm run test
```

手动验证：

1. Enterprise SAML 登录不再依赖 Keycloak SAML broker
2. SP metadata 由 Auth9 生成
3. 非法 assertion、issuer、audience、RelayState 会被 Auth9 自己拒绝
