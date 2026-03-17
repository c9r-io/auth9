# Phase 4: 外部身份代理层（Social / Enterprise OIDC / Enterprise SAML）

**类型**: 核心能力替换
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core (Backend), auth9-portal (Frontend), provisioning, tenant access
**前置依赖**:
- `keycloak_phase1_identity_engine_abstraction.md`
- `keycloak_phase2_hosted_login_and_session_frontend.md`
- `keycloak_phase3_local_credentials_and_mfa.md`
**被依赖**:
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

当前 Auth9 的 social login 和 enterprise SSO connector 真实落地仍是 Keycloak identity provider / broker。只要这一层没接管，Keycloak 就仍然是系统的联邦核心。

本阶段目标是让 Auth9 自己成为 broker。

---

## 期望行为

### R1: Social Login Broker

接管以下社交身份源：

- Google
- GitHub
- Microsoft
- 通用 OIDC Social Provider

要求：

- Provider 配置存储在 Auth9
- 授权跳转、callback、token exchange、profile mapping 由 Auth9 执行
- 不再依赖 Keycloak identity provider 实例

### R2: Enterprise OIDC Connector

重建企业 OIDC connector 模型与运行时：

- issuer
- client_id / secret
- authorize/token/userinfo/jwks endpoint
- domain discovery
- claim mapping
- login hint / domain hint / organization hint

要求：

- `enterprise_sso_connectors` 中的 `keycloak_alias` 彻底退场
- domain-based discovery 直接返回 Auth9 broker redirect

### R3: Enterprise SAML Connector

新增 Auth9 自己的 SAML broker 能力：

- SP metadata 生成
- AuthnRequest 发起
- Response 校验
- NameID / Attribute mapping
- RelayState 防篡改

要求：

- 允许先覆盖最小企业 SAML 登录场景
- 不要求一开始支持全部 SLO 变体

### R4: Federated Identity Linking

Auth9 自己维护：

- `linked_identities`
- 主账号与外部身份绑定
- unlink / relink
- 首次登录自动合并 / 手动合并策略

要求：

- 不再调用 Keycloak federated identity API
- 历史绑定关系需可迁移

### R5: 审计与安全事件

Auth9 自己产出联邦事件：

- login success / failure
- idp timeout
- invalid assertion
- invalid issuer / audience
- account link / unlink

要求：

- 不再依赖 Keycloak event SPI webhook 作为主事件源

### R6: 测试覆盖

- Social provider mock
- OIDC enterprise discovery / callback / mapping
- SAML response verification
- linked identity conflict / takeover 防护
- broker 超时 / 重试 / cancel path

---

## 非目标

- 本阶段不要求立刻支持所有冷门身份源
- 本阶段不要求做完整通用 SAML 平台

---

## 验证方法

```bash
cd auth9-core && cargo test identity_provider
cd auth9-oidc && cargo test federation
```

手动验证：

1. 新建 social / enterprise connector 时不再在 Keycloak 中创建实例
2. 域名发现后跳转到 Auth9 broker 流程
3. 登录成功后 `linked_identities` 由 Auth9 自己维护

