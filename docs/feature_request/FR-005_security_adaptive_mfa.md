# 自适应 MFA（风险驱动多因素认证）

**类型**: 功能增强
**严重程度**: High
**影响范围**: auth9-core (Backend), auth9-portal (Frontend)
**前置依赖**: `FR-004_security_anomaly_detection_enhanced.md`（风险评分引擎）

---

## 背景

Auth9 已具备完善的 MFA 能力（TOTP、WebAuthn/Passkey、Email OTP、Recovery Codes），但 MFA 触发是静态的 — 要么全局启用，要么全局禁用。Auth0 提供风险驱动的自适应 MFA：低风险登录跳过 MFA，高风险登录强制 MFA，在安全与用户体验之间取得平衡。

| 维度 | Auth0 | Auth9 现状 |
|------|-------|-----------|
| MFA 触发 | 风险评分动态决定 | 全局开/关 |
| 触发条件 | 新设备、新地点、暴力破解、密码泄漏 | 无条件触发 |
| 记住设备 | "Trust this device" 30 天跳过 | 不支持 |
| Step-up Auth | 高敏感操作临时要求 MFA | 不支持 |
| 降级策略 | 风险引擎故障时回退策略 | 不适用 |

### 现有基础（可复用）

- TOTP 服务: `auth9-core/src/domains/identity/service/totp.rs`
- WebAuthn 服务: `auth9-core/src/domains/identity/service/webauthn.rs`
- Email OTP 服务: `auth9-core/src/domains/identity/service/otp/`
- MFA 状态查询: `/api/v1/mfa/status`
- MFA 验证端点: `/api/v1/mfa/challenge/verify`
- 安全检测服务: `security_detection.rs`（告警生成）

---

## 期望行为

### R1: MFA 策略引擎

新增 `AdaptiveMfaService`，在认证流程中根据风险评分决定是否要求 MFA：

```rust
pub struct AdaptiveMfaPolicy {
    pub tenant_id: String,
    pub mode: AdaptiveMfaMode,
    pub risk_threshold: u8,                // 风险分超过此值要求 MFA，默认 40
    pub always_require_for_admins: bool,   // 管理员始终要求 MFA，默认 true
    pub trust_device_days: u16,            // 信任设备天数，默认 30
    pub step_up_operations: Vec<String>,   // 需要 step-up 的操作列表
}

pub enum AdaptiveMfaMode {
    Disabled,       // 不要求 MFA
    Always,         // 始终要求 MFA（现有行为）
    Adaptive,       // 风险驱动
    OptionalEnroll, // 鼓励但不强制
}

pub enum MfaDecision {
    Skip,                           // 低风险，跳过 MFA
    Required { methods: Vec<MfaMethod>, reason: String },  // 需要 MFA
    StepUp { operation: String },   // Step-up 认证
}
```

决策逻辑：

1. `Disabled` → 始终跳过
2. `Always` → 始终要求（向后兼容）
3. `Adaptive` →
   - 检查设备是否已信任（R2）→ 已信任且低风险 → 跳过
   - 计算风险评分（依赖 `FR-004_security_anomaly_detection_enhanced.md` 的风险引擎）
   - 风险分 > `risk_threshold` → 要求 MFA
   - 管理员角色 + `always_require_for_admins` → 要求 MFA
   - 其他 → 跳过

**涉及文件**:
- `auth9-core/src/domains/identity/service/adaptive_mfa.rs` — 新增自适应 MFA 服务
- `auth9-core/src/models/system_settings.rs` — MFA 策略配置

### R2: 设备信任机制

允许用户在 MFA 验证后标记设备为"已信任"，在信任期内跳过 MFA：

```sql
CREATE TABLE trusted_devices (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    tenant_id VARCHAR(36) NOT NULL,
    device_fingerprint VARCHAR(255) NOT NULL,   -- user_agent + IP 段 组合哈希
    device_name VARCHAR(255),                    -- 人类可读的设备描述
    trusted_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    last_used_at TIMESTAMP NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    INDEX idx_trusted_devices_user (user_id),
    INDEX idx_trusted_devices_lookup (user_id, device_fingerprint),
    INDEX idx_trusted_devices_expires (expires_at)
);
```

- 设备指纹 = SHA-256(user_agent + IP /24 子网)，允许同一网段内 IP 变化
- 信任期默认 30 天，管理员可配置
- 用户可在个人设置中查看和撤销已信任设备
- 管理员可批量撤销某用户所有设备信任

**涉及文件**:
- `auth9-core/migrations/` — 新增 `trusted_devices` 表
- `auth9-core/src/repository/trusted_device.rs` — Repository 层
- `auth9-core/src/domains/identity/service/trusted_device.rs` — 服务层

### R3: 登录流程集成

修改登录流程，在密码/Social/SSO 验证成功后、签发 Token 之前插入 MFA 决策点：

```
用户提交凭证
    ↓
验证凭证（密码/Social/SSO）
    ↓
计算风险评分 (来自 risk_engine)
    ↓
AdaptiveMfaService.evaluate(user, risk_score, device)
    ↓
├── MfaDecision::Skip → 签发 Token
├── MfaDecision::Required → 返回 MFA Challenge
│   ↓
│   用户完成 MFA 验证
│   ↓
│   (可选) "Trust this device?" → 创建设备信任
│   ↓
│   签发 Token
└── MfaDecision::StepUp → (见 R4)
```

- MFA Challenge 返回用户已注册的 MFA 方法列表
- 支持多方法选择（如 TOTP 或 WebAuthn）
- MFA 中间状态通过 Redis 存储（TTL 5 分钟）

**涉及文件**:
- `auth9-core/src/domains/identity/api/auth/login.rs` — 登录流程插入决策点
- `auth9-core/src/domains/identity/api/auth/social_callback.rs` — Social 登录集成
- `auth9-core/src/domains/identity/api/enterprise_saml_broker.rs` — SAML SSO 集成

### R4: Step-up Authentication

对高敏感操作（如修改密码、管理 API Key、修改 RBAC）要求临时 MFA 验证：

```rust
// Step-up 保护的操作
const STEP_UP_OPERATIONS: &[&str] = &[
    "change_password",
    "manage_api_keys",
    "modify_rbac",
    "delete_tenant",
    "modify_security_settings",
    "export_user_data",
];
```

- Step-up Token 有效期 15 分钟
- Step-up 结果缓存在 session 中，同一会话内不重复要求
- 管理员可自定义需要 step-up 的操作列表

**涉及文件**:
- `auth9-core/src/middleware/` — 新增 `step_up_auth` 中间件
- `auth9-core/src/domains/identity/api/` — 敏感操作端点添加 step-up 检查

### R5: Portal UI 集成

1. **登录流程**:
   - MFA Challenge 页面已存在，新增"Trust this device for 30 days"复选框
   - 低风险登录无感跳过 MFA

2. **用户设置 — 已信任设备管理**:
   - 展示已信任设备列表（设备名、信任时间、最后使用时间）
   - 支持单个撤销和全部撤销

3. **管理面板 — MFA 策略配置**:
   - 模式选择（Disabled / Always / Adaptive / Optional）
   - 风险阈值滑块
   - 信任设备天数配置
   - Step-up 操作列表配置

4. **Step-up 弹窗**:
   - 执行敏感操作时弹出 MFA 验证弹窗
   - 支持 TOTP 或 WebAuthn 验证

**涉及文件**:
- `auth9-portal/app/routes/login.tsx` — 信任设备复选框
- `auth9-portal/app/routes/dashboard.settings.security.tsx` — MFA 策略配置
- `auth9-portal/app/routes/dashboard.profile.tsx` — 已信任设备管理
- `auth9-portal/app/components/` — Step-up 验证弹窗组件

### R6: 单元测试覆盖

- MFA 决策引擎：各模式 × 各风险等级 × 设备信任状态的组合
- 设备信任：创建、查找、过期、撤销
- 登录流程：MFA 跳过 / MFA 要求 / Step-up 三种路径
- Step-up 中间件：保护/非保护操作、Token 过期
- 向后兼容：`Always` 模式与现有行为一致

---

## 安全考量

1. **设备信任安全**: 设备指纹基于 user_agent + IP 子网，不够强壮。建议后续引入浏览器指纹库增强
2. **风险引擎故障降级**: 风险引擎不可用时，建议降级为 `Always` 模式（宁可多要求 MFA，不可跳过）
3. **Step-up Token 保护**: Step-up Token 绑定 session ID，防止跨会话重放
4. **管理员特权**: `always_require_for_admins` 默认启用，防止管理员账户被低风险评分绕过

---

## 验证方法

### 代码验证

```bash
grep -r "AdaptiveMfa\|MfaDecision\|trusted_device\|step_up" auth9-core/src/
cd auth9-core && cargo test adaptive_mfa
cd auth9-core && cargo test trusted_device
```

### 手动验证

1. 设置 Adaptive 模式，从已知设备登录 → 验证跳过 MFA
2. 从新设备/新 IP 登录 → 验证要求 MFA
3. 完成 MFA 后勾选"Trust this device" → 再次登录验证跳过
4. 在用户设置中撤销设备信任 → 再次登录验证要求 MFA
5. 尝试修改密码 → 验证 Step-up 弹窗触发
6. 管理员登录 → 验证始终要求 MFA

---

## 实现顺序

1. **R1: MFA 策略引擎** — 核心决策逻辑
2. **R2: 设备信任机制** — 决策引擎依赖
3. **R3: 登录流程集成** — 串联所有组件
4. **R4: Step-up Authentication** — 独立模块，可并行
5. **R5: Portal UI** — 最后集成
6. **R6: 测试覆盖** — 贯穿各阶段

---

## 参考

- 风险评分引擎: `docs/feature_request/security_anomaly_detection_enhanced.md`
- 现有 MFA 服务: `auth9-core/src/domains/identity/service/totp.rs`, `webauthn.rs`
- 现有 MFA API: `auth9-core/src/domains/identity/api/mfa.rs`
- NIST 800-63B: Authenticator Assurance Levels (AAL1/AAL2/AAL3)
