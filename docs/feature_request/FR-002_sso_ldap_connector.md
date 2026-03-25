# LDAP/Active Directory 企业 SSO 连接器

**类型**: 新功能
**严重程度**: Medium
**影响范围**: auth9-core (Backend), auth9-portal (Frontend)
**前置依赖**: 无

---

## 背景

Auth9 当前支持 SAML 2.0 和 OIDC 作为企业 SSO 协议，覆盖了大部分现代 IdP 集成场景。但许多企业（特别是传统企业和内部系统）仍依赖 LDAP/Active Directory 作为主要身份源。Auth0 原生支持 LDAP/AD 连接器，Auth9 完全缺失此能力。

| 维度 | Auth0 | Auth9 现状 |
|------|-------|-----------|
| LDAP 绑定认证 | ✅ | ❌ |
| AD 集成 | ✅ 原生 | ❌ |
| 目录搜索 | ✅ | ❌ |
| 用户属性映射 | ✅ 灵活映射 | ❌ |
| 连接模式 | 云端 + Agent | ❌ |

### 技术背景

LDAP (Lightweight Directory Access Protocol) 是企业目录服务的标准协议。Active Directory 是微软基于 LDAP 的目录实现。关键集成方式：

1. **直连模式**: Auth9 直接连接企业 LDAP 服务器（需网络可达）
2. **Agent 模式**: 企业内网部署轻量代理，代理与 Auth9 云端通信（解决网络隔离问题）

本 FR 聚焦直连模式，Agent 模式可作为后续扩展。

---

## 期望行为

### R1: LDAP 连接器配置模型

新增 LDAP 连接器数据模型和存储：

```sql
CREATE TABLE ldap_connectors (
    id VARCHAR(36) PRIMARY KEY,
    tenant_id VARCHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    alias VARCHAR(100) NOT NULL,                -- URL 友好标识
    enabled BOOLEAN NOT NULL DEFAULT TRUE,

    -- 连接配置
    server_url VARCHAR(512) NOT NULL,           -- ldap://host:389 或 ldaps://host:636
    use_tls BOOLEAN NOT NULL DEFAULT TRUE,      -- STARTTLS (ldap://) 或隐式 TLS (ldaps://)
    tls_skip_verify BOOLEAN NOT NULL DEFAULT FALSE,
    tls_ca_cert TEXT,                           -- 自签名 CA 证书 (PEM)
    connection_timeout_secs INT NOT NULL DEFAULT 10,

    -- 绑定凭证（用于搜索）
    bind_dn VARCHAR(512) NOT NULL,              -- 如 cn=auth9,ou=services,dc=company,dc=com
    bind_password_encrypted TEXT NOT NULL,       -- pragma: allowlist secret       -- 加密存储

    -- 搜索配置
    base_dn VARCHAR(512) NOT NULL,              -- 搜索起点，如 ou=users,dc=company,dc=com
    user_search_filter VARCHAR(512) NOT NULL DEFAULT '(uid={username})',
    user_search_scope VARCHAR(10) NOT NULL DEFAULT 'sub',  -- sub, one, base
    group_search_base VARCHAR(512),
    group_search_filter VARCHAR(512) DEFAULT '(member={dn})',

    -- 属性映射
    attr_username VARCHAR(100) NOT NULL DEFAULT 'uid',           -- AD 用 sAMAccountName
    attr_email VARCHAR(100) NOT NULL DEFAULT 'mail',
    attr_first_name VARCHAR(100) NOT NULL DEFAULT 'givenName',
    attr_last_name VARCHAR(100) NOT NULL DEFAULT 'sn',
    attr_display_name VARCHAR(100) DEFAULT 'displayName',
    attr_phone VARCHAR(100) DEFAULT 'telephoneNumber',
    attr_groups VARCHAR(100) DEFAULT 'memberOf',

    -- Active Directory 特有
    is_active_directory BOOLEAN NOT NULL DEFAULT FALSE,
    ad_domain VARCHAR(255),                     -- 如 company.com（用于 UPN 登录）

    -- 首次登录策略
    first_login_policy VARCHAR(20) NOT NULL DEFAULT 'auto_create',  -- auto_create, manual
    default_role_ids JSON,                      -- 首次登录自动分配的角色

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    UNIQUE INDEX idx_ldap_tenant_alias (tenant_id, alias),
    INDEX idx_ldap_tenant (tenant_id)
);
```

**涉及文件**:
- `auth9-core/migrations/` — 新增迁移文件
- `auth9-core/src/models/ldap_connector.rs` — 模型定义
- `auth9-core/src/repository/ldap_connector.rs` — CRUD Repository

### R2: LDAP 认证服务

新增 `LdapAuthService`，实现 LDAP bind 认证流程：

```rust
pub struct LdapAuthService {
    // 连接池管理
}

impl LdapAuthService {
    /// LDAP 认证流程:
    /// 1. 使用 bind_dn/bind_password 连接 LDAP（服务账号）
    /// 2. 搜索用户 DN：base_dn + user_search_filter（替换 {username}）
    /// 3. 用户 DN + 用户密码再次 bind（验证密码）
    /// 4. 成功后读取用户属性
    /// 5. 返回标准化的用户信息
    pub async fn authenticate(
        &self,
        connector: &LdapConnector,
        username: &str,
        password: &str,
    ) -> Result<LdapUserInfo>;

    /// 测试 LDAP 连接（管理员配置时使用）
    pub async fn test_connection(
        &self,
        connector: &LdapConnector,
    ) -> Result<LdapTestResult>;

    /// 搜索 LDAP 用户（用于用户导入预览）
    pub async fn search_users(
        &self,
        connector: &LdapConnector,
        query: &str,
        limit: u32,
    ) -> Result<Vec<LdapUserInfo>>;
}

pub struct LdapUserInfo {
    pub dn: String,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub groups: Vec<String>,
    pub raw_attributes: HashMap<String, Vec<String>>,
}
```

使用 `ldap3` crate（Rust 原生异步 LDAP 客户端）。

**涉及文件**:
- `auth9-core/src/domains/identity/service/ldap_auth.rs` — LDAP 认证服务
- `auth9-core/Cargo.toml` — 新增 `ldap3` 依赖

### R3: LDAP 登录端点

新增公开端点 `POST /api/v1/auth/ldap/login`：

```json
// 请求
{
  "connector_alias": "company-ad",
  "username": "john.doe",        // 或 john.doe@company.com (UPN)
  "password": "user_password"
}
<!-- pragma: allowlist secret -->

// 成功响应
{
  "identity_token": "eyJ...",
  "user": {
    "id": "...",
    "email": "john.doe@company.com",
    "display_name": "John Doe"
  },
  "is_first_login": false
}
```

流程：
1. 根据 `connector_alias` + 请求来源租户查找 LDAP 连接器
2. 调用 `LdapAuthService::authenticate`
3. 首次登录 → 自动创建 Auth9 用户（根据 `first_login_policy`）
4. 非首次登录 → 更新用户属性（如 LDAP 侧修改了姓名/邮箱）
5. 签发 Identity Token
6. 记录 login_event（event_type: `federation`, provider_type: `ldap`）

**涉及文件**:
- `auth9-core/src/domains/identity/api/auth/ldap.rs` — LDAP 登录 handler
- `auth9-core/src/server/mod.rs` — 路由注册

### R4: LDAP 管理 API

受保护的管理端点：

| 方法 | 端点 | 说明 |
|------|------|------|
| POST | `/api/v1/enterprise-sso/ldap` | 创建 LDAP 连接器 |
| GET | `/api/v1/enterprise-sso/ldap` | 列出租户下所有 LDAP 连接器 |
| GET | `/api/v1/enterprise-sso/ldap/{id}` | 获取单个连接器详情 |
| PUT | `/api/v1/enterprise-sso/ldap/{id}` | 更新连接器配置 |
| DELETE | `/api/v1/enterprise-sso/ldap/{id}` | 删除连接器 |
| POST | `/api/v1/enterprise-sso/ldap/{id}/test` | 测试连接 |
| POST | `/api/v1/enterprise-sso/ldap/{id}/search-users` | 搜索 LDAP 用户 |

**涉及文件**:
- `auth9-core/src/domains/identity/api/ldap_management.rs` — 管理端点
- `auth9-core/src/api/access_control.rs` — 新增 LDAP 管理权限

### R5: LDAP 组到 Auth9 角色映射

支持将 LDAP 组自动映射为 Auth9 角色：

```sql
CREATE TABLE ldap_group_role_mappings (
    id VARCHAR(36) PRIMARY KEY,
    ldap_connector_id VARCHAR(36) NOT NULL,
    ldap_group_dn VARCHAR(512) NOT NULL,        -- LDAP 组 DN
    role_id VARCHAR(36) NOT NULL,                -- Auth9 角色 ID
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE INDEX idx_ldap_group_role (ldap_connector_id, ldap_group_dn, role_id),
    INDEX idx_ldap_connector (ldap_connector_id)
);
```

- 用户登录时自动同步组→角色映射
- 支持多对多映射（一个 LDAP 组 → 多个 Auth9 角色）
- 组变更时下次登录自动同步（不做实时同步）

**涉及文件**:
- `auth9-core/migrations/` — 新增映射表
- `auth9-core/src/domains/identity/service/ldap_auth.rs` — 角色同步逻辑

### R6: Portal LDAP 管理 UI

1. **连接器管理页面** (`/dashboard/enterprise-sso/ldap`):
   - 连接器列表（名称、状态、服务器地址、用户数）
   - 创建/编辑/删除连接器
   - 连接测试按钮（实时反馈连接状态）

2. **连接器配置表单**:
   - 基本信息：名称、别名
   - 连接设置：服务器 URL、TLS、超时
   - 绑定凭证：Bind DN、密码（密码字段加密）
   - 搜索配置：Base DN、搜索过滤器、搜索范围
   - 属性映射：各字段映射（预填 LDAP 或 AD 默认值）
   - AD 特有：域名、UPN 支持开关
   - 高级：首次登录策略、默认角色

3. **组角色映射页面**:
   - 左侧显示 LDAP 组（通过搜索获取）
   - 右侧显示 Auth9 角色
   - 拖拽或选择建立映射关系

4. **登录页集成**:
   - 当租户配置了 LDAP 连接器时，登录页显示 "Sign in with Corporate Account" 入口
   - 点击后显示用户名/密码表单（带连接器选择，如有多个）

**涉及文件**:
- `auth9-portal/app/routes/dashboard.enterprise-sso.ldap.tsx` — LDAP 管理页面
- `auth9-portal/app/routes/dashboard.enterprise-sso.ldap.$id.tsx` — 连接器详情
- `auth9-portal/app/routes/login.tsx` — 添加 LDAP 登录入口
- `auth9-portal/app/services/api.ts` — LDAP API 方法

### R7: 单元测试覆盖

- LDAP 连接器 CRUD：创建、更新、删除、查询
- 认证流程：使用 mock LDAP 服务器（`ldap3` crate 提供测试工具）
- 用户搜索：过滤器替换、属性映射
- 组角色同步：映射创建、用户登录时同步
- AD 特有：UPN 登录、sAMAccountName 查找
- TLS 配置：自签名证书处理、STARTTLS
- 错误处理：连接超时、无效凭证、用户不存在、搜索无结果

---

## 安全考量

1. **绑定密码存储**: bind_password 必须加密存储（AES-256-GCM），不可明文
2. **TLS 强制**: 生产环境应强制 LDAPS 或 STARTTLS，UI 中对非 TLS 连接显示安全警告
3. **搜索注入**: username 代入搜索过滤器时必须转义 LDAP 特殊字符（`*`, `(`, `)`, `\`, `NUL`）
4. **连接池**: LDAP 连接应池化管理，避免每次认证新建连接
5. **密码不存储**: Auth9 不存储 LDAP 用户密码，每次认证都通过 LDAP bind 验证
6. **网络安全**: LDAP 连接器仅从 auth9-core 服务器发起，不暴露 LDAP 服务器地址给客户端

---

## 验证方法

### 代码验证

```bash
grep -r "LdapConnector\|LdapAuth\|ldap3" auth9-core/src/
cd auth9-core && cargo test ldap
```

### 手动验证

1. 启动 OpenLDAP 测试容器：`docker run -p 389:389 osixia/openldap`
2. 在 Portal 创建 LDAP 连接器，配置测试 LDAP 服务器
3. 点击"Test Connection"验证连接成功
4. 搜索 LDAP 用户，验证属性映射正确
5. 使用 LDAP 用户登录 Portal，验证自动创建 Auth9 用户
6. 配置组角色映射，验证登录后角色自动同步
7. 测试 Active Directory 配置（需 AD 测试环境或 Samba AD）

---

## 实现顺序

1. **R1: 数据模型** — 基础存储
2. **R2: LDAP 认证服务** — 核心能力
3. **R3: 登录端点** — 最小可用
4. **R4: 管理 API** — 配置能力
5. **R5: 组角色映射** — 增强功能
6. **R6: Portal UI** — 最后集成
7. **R7: 测试覆盖** — 贯穿各阶段

---

## 参考

- 现有 SAML 连接器: `auth9-core/src/domains/identity/api/enterprise_saml_broker.rs`
- 现有 OIDC 连接器: `auth9-core/src/domains/identity/api/enterprise_broker.rs`
- `ldap3` crate: https://docs.rs/ldap3/latest/ldap3/
- RFC 4511: LDAP Protocol
- Microsoft AD LDAP 参考: https://learn.microsoft.com/en-us/windows/win32/ad/active-directory-domain-services
