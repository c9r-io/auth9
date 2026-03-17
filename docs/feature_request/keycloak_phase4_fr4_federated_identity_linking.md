# Phase 4 FR4: Federated Identity Linking

**类型**: 核心能力替换 + 安全加固
**严重程度**: High
**影响范围**: `auth9-oidc`, auth9-core (Backend), auth9-portal (Frontend), linked identities, tests, docs/security
**前置依赖**:
- `keycloak_phase4_fr1_social_login_broker.md`
- `keycloak_phase4_fr2_enterprise_oidc_connector.md`
- `keycloak_phase4_fr3_enterprise_saml_broker.md`
**被依赖**:
- `keycloak_phase4_fr5_federation_audit_and_security_events.md`
- `keycloak_phase5_cutover_and_keycloak_retirement.md`

---

## 背景

外部身份 broker 接管后，账号绑定关系如果仍然由 Keycloak federated identity API 维护，Auth9 仍然无法完全控制账号合并、冲突判定和 takeover 防护。

因此 `linked_identities`、link/unlink/relink、首次登录合并策略必须单独本地化。

---

## 期望行为

### R1: Auth9 自己维护 linked identities

要求：

- `linked_identities` 由 Auth9 自己作为主数据维护
- 不再调用 Keycloak federated identity API

### R2: 支持 link / unlink / relink

要求：

- 用户可主动 unlink / relink 外部身份
- 账号设置页和 API 均支持新的 linking 流程

### R3: 首次登录自动合并 / 手动合并策略

要求：

- 可定义首次登录时的自动合并或手动确认策略
- 冲突时不能静默 takeover

### R4: 历史绑定关系可迁移

要求：

- 历史 federated identity 绑定关系可迁移到本地 `linked_identities`
- 迁移期内保证读写一致或可验证

### R5: 安全测试

要求：

- linked identity conflict
- account takeover 防护
- unlink / relink 权限检查

均需覆盖。

---

## 非目标

- 本 FR 不要求定义所有 broker 的事件规范
- 本 FR 不要求替换所有历史导入工具
- 本 FR 不要求处理非第一方账号的复杂主账号治理策略

---

## 验证方法

```bash
cd auth9-core && cargo test linked_identity
cd auth9-oidc && cargo test federation_linking
cd auth9-portal && npm run test
```

手动验证：

1. social / enterprise 登录成功后，`linked_identities` 由 Auth9 自己维护
2. unlink / relink 不再调用 Keycloak federated identity API
3. 冲突场景下不会发生静默账号接管
