# Bot 防护（CAPTCHA 集成）

**类型**: 新功能
**严重程度**: High
**影响范围**: auth9-core (Backend), auth9-portal (Frontend)
**前置依赖**: 无

---

## 背景

Auth9 当前依赖速率限制和 IP 黑名单进行 Bot 防护，但缺少 CAPTCHA 验证这一业界标准的人机识别手段。Auth0 在登录、注册等关键端点集成了 CAPTCHA 防护，有效阻止自动化攻击。

| 维度 | Auth0 | Auth9 现状 |
|------|-------|-----------|
| CAPTCHA | reCAPTCHA/hCaptcha | ❌ 无 |
| 触发方式 | 始终/可疑时 | 不适用 |
| 自定义 | 可配置阈值和外观 | 不适用 |
| Bot 评分 | reCAPTCHA v3 分数 | 不适用 |
| 防护范围 | 登录、注册、密码重置 | 仅速率限制 |

### CAPTCHA 提供商对比

| 提供商 | 优势 | 劣势 |
|--------|------|------|
| Cloudflare Turnstile | 免费、隐私友好、无视觉挑战 | 市场份额较小 |
| Google reCAPTCHA v3 | 广泛使用、invisible 模式 | 隐私顾虑、需 Google 账户 |
| hCaptcha | 隐私导向、GDPR 友好 | 视觉挑战影响体验 |

本 FR 采用**提供商抽象**设计，优先支持 Cloudflare Turnstile（免费 + 隐私友好），同时保留 reCAPTCHA 和 hCaptcha 扩展接口。

---

## 期望行为

### R1: CAPTCHA 提供商抽象层

设计可扩展的提供商接口：

```rust
#[async_trait]
pub trait CaptchaProvider: Send + Sync {
    /// 验证 CAPTCHA token
    async fn verify(&self, token: &str, remote_ip: Option<&str>) -> Result<CaptchaVerification>;

    /// 提供商名称（用于日志和配置）
    fn provider_name(&self) -> &str;
}

pub struct CaptchaVerification {
    pub success: bool,
    pub score: Option<f64>,             // 0.0-1.0，部分提供商支持
    pub challenge_ts: Option<String>,
    pub hostname: Option<String>,
    pub error_codes: Vec<String>,
}

pub enum CaptchaProviderType {
    Turnstile,
    RecaptchaV3,
    HCaptcha,
}
```

**Turnstile 实现**:

```rust
pub struct TurnstileProvider {
    secret_key: String,
    http_client: reqwest::Client,
    verify_url: String,  // https://challenges.cloudflare.com/turnstile/v0/siteverify
}
```

验证流程：POST `secret` + `response` (token) + `remoteip` → 解析 JSON 响应。

**涉及文件**:
- `auth9-core/src/domains/security_observability/service/captcha/mod.rs` — trait 定义
- `auth9-core/src/domains/security_observability/service/captcha/turnstile.rs` — Turnstile 实现
- `auth9-core/src/domains/security_observability/service/captcha/recaptcha.rs` — reCAPTCHA 预留
- `auth9-core/src/config/` — CAPTCHA 配置项

### R2: CAPTCHA 验证中间件

新增中间件，在指定端点验证 CAPTCHA token：

```rust
pub struct CaptchaConfig {
    pub enabled: bool,
    pub provider: CaptchaProviderType,
    pub site_key: String,               // 前端使用
    pub secret_key: String,             // 后端使用
    pub mode: CaptchaMode,
    pub score_threshold: f64,           // 默认 0.5，低于此分数拒绝
    pub protected_endpoints: Vec<ProtectedEndpoint>,
}

pub enum CaptchaMode {
    Always,         // 所有请求都需要 CAPTCHA
    Adaptive,       // 基于风险/速率动态决定
    Disabled,       // 关闭
}

pub struct ProtectedEndpoint {
    pub path: String,
    pub methods: Vec<String>,
    pub mode_override: Option<CaptchaMode>,  // 可覆盖全局模式
}
```

默认保护的端点：
- `POST /api/v1/auth/login` — 登录
- `POST /api/v1/auth/register` — 注册
- `POST /api/v1/auth/forgot-password` — 忘记密码
- `POST /api/v1/auth/email-otp/send` — Email OTP 发送
- `POST /api/v1/auth/sms-otp/send` — SMS OTP 发送（如已实现）

CAPTCHA token 通过请求头 `X-Captcha-Token` 或请求体 `captcha_token` 字段传递。

**涉及文件**:
- `auth9-core/src/middleware/captcha.rs` — CAPTCHA 验证中间件
- `auth9-core/src/server/mod.rs` — 中间件注册

### R3: Adaptive CAPTCHA 触发

在 `Adaptive` 模式下，仅在可疑请求时要求 CAPTCHA：

触发条件（满足任一即要求 CAPTCHA）：

| 条件 | 说明 |
|------|------|
| 同一 IP 连续 3 次登录失败 | 短期暴力破解嫌疑 |
| 同一 IP 10 分钟内 > 5 次请求 | 自动化脚本嫌疑 |
| IP 在黑名单中 | 已知恶意 IP |
| 风险评分 > 40 | 依赖风险引擎（如已实现） |
| 请求缺少合理 User-Agent | Bot 特征 |

当不需要 CAPTCHA 时，后端在响应头返回 `X-Captcha-Required: false`；需要时返回 `X-Captcha-Required: true` + `X-Captcha-Site-Key: {key}`。

前端根据此信号动态加载 CAPTCHA 组件。

**涉及文件**:
- `auth9-core/src/middleware/captcha.rs` — 自适应逻辑
- `auth9-core/src/domains/security_observability/service/security_detection.rs` — 复用检测逻辑

### R4: Portal 前端集成

1. **CAPTCHA 通用组件**:

```typescript
// app/components/captcha.tsx
interface CaptchaProps {
  provider: "turnstile" | "recaptcha" | "hcaptcha";
  siteKey: string;
  onVerify: (token: string) => void;
  onError?: () => void;
  theme?: "light" | "dark" | "auto";
  size?: "normal" | "compact" | "invisible";
}
```

- Turnstile: 动态加载 `challenges.cloudflare.com/turnstile/v0/api.js`
- reCAPTCHA: 动态加载 `google.com/recaptcha/api.js`
- 脚本延迟加载，不影响首屏性能

2. **登录页集成**:
   - `Always` 模式：页面加载时即显示 CAPTCHA 组件
   - `Adaptive` 模式：首次提交不带 CAPTCHA → 后端返回 `X-Captcha-Required: true` → 前端动态加载并要求完成 CAPTCHA 后重试
   - CAPTCHA token 附加到表单提交中

3. **注册页、忘记密码页**同理集成

**涉及文件**:
- `auth9-portal/app/components/captcha.tsx` — CAPTCHA 通用组件
- `auth9-portal/app/routes/login.tsx` — 登录页集成
- `auth9-portal/app/routes/register.tsx` — 注册页集成
- `auth9-portal/app/routes/forgot-password.tsx` — 忘记密码页集成

### R5: 租户级配置

管理员可在 Portal 中配置 CAPTCHA：

- **提供商选择**: Turnstile / reCAPTCHA v3 / hCaptcha
- **Site Key / Secret Key**: 租户自有的 CAPTCHA 凭证
- **模式**: Always / Adaptive / Disabled
- **分数阈值**: 0.0 - 1.0 滑块（仅 reCAPTCHA v3 和 Turnstile 适用）
- **受保护端点**: 可勾选启用/禁用各端点的 CAPTCHA 保护

配置存储在系统设置表中，支持租户级覆盖。

**涉及文件**:
- `auth9-core/src/models/system_settings.rs` — CAPTCHA 配置模型
- `auth9-portal/app/routes/dashboard.settings.security.tsx` — 配置页面

### R6: 单元测试覆盖

- Turnstile token 验证：成功、失败、过期、格式错误
- 中间件：有 token 通过、无 token 拒绝、Disabled 模式跳过
- Adaptive 触发逻辑：各触发条件的边界测试
- 提供商 fallback：API 不可用时的降级行为（fail-open，记录日志）
- 使用 HTTP mock（`wiremock`），不调用真实 CAPTCHA API
- 前端组件：渲染测试、onVerify 回调、错误处理

---

## 安全考量

1. **Secret Key 保护**: CAPTCHA Secret Key 仅存于后端，绝不暴露给前端
2. **Token 重放**: 每个 CAPTCHA token 只能验证一次，提供商会自动拒绝重复提交
3. **降级策略**: CAPTCHA API 不可用时 fail-open（允许通过），但速率限制仍然生效
4. **隐私合规**: Turnstile 不使用 Cookie 追踪用户，GDPR 友好；reCAPTCHA 需在隐私政策中声明
5. **绕过防护**: Bot 可能使用 CAPTCHA 解决服务，CAPTCHA 应作为多层防护之一，不是唯一手段

---

## 验证方法

### 代码验证

```bash
grep -r "CaptchaProvider\|Turnstile\|captcha_token" auth9-core/src/ auth9-portal/app/
cd auth9-core && cargo test captcha
cd auth9-portal && npm run test
```

### 手动验证

1. 注册 Cloudflare Turnstile 获取测试 Site Key / Secret Key
2. 在 Portal 配置 CAPTCHA 为 Always 模式
3. 访问登录页，验证 Turnstile 组件显示
4. 完成 CAPTCHA 后登录，验证后端收到并验证 token
5. 切换到 Adaptive 模式，正常登录验证无 CAPTCHA
6. 连续 3 次输错密码后，验证 CAPTCHA 组件出现
7. 禁用 CAPTCHA，验证所有端点正常工作

---

## 实现顺序

1. **R1: 提供商抽象层** — 基础架构
2. **R2: 验证中间件** — 核心能力（Always 模式）
3. **R4: 前端组件** — 前后端联调
4. **R3: Adaptive 触发** — 增强模式
5. **R5: 租户配置** — 管理界面
6. **R6: 测试覆盖** — 贯穿各阶段

---

## 参考

- Cloudflare Turnstile: https://developers.cloudflare.com/turnstile/
- Turnstile Server-side Validation: https://developers.cloudflare.com/turnstile/get-started/server-side-validation/
- Google reCAPTCHA v3: https://developers.google.com/recaptcha/docs/v3
- hCaptcha: https://docs.hcaptcha.com/
- 现有速率限制: `auth9-core/src/middleware/rate_limit.rs`
- 现有 IP 黑名单: `auth9-core/src/domains/security_observability/`
