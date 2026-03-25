# 增强异常检测：风险评分引擎与自动响应

**类型**: 功能增强
**严重程度**: High
**影响范围**: auth9-core (Backend), auth9-portal (Frontend)
**前置依赖**: 无（在现有 security_detection 基础上增量扩展）

---

## 背景

Auth9 当前已具备规则驱动的安全检测能力（暴力破解、密码喷射、不可能旅行、新设备检测、IP 黑名单），但与 Auth0 的 ML 风险驱动异常检测相比，存在以下差距：

| 维度 | Auth0 | Auth9 现状 |
|------|-------|-----------|
| 风险评分 | 每次登录实时计算 0-100 风险分 | 无，仅二元告警 |
| 用户画像 | 自动建立用户行为基线 | 无历史行为建模 |
| 自动响应 | 高风险自动阻断/要求 MFA | 仅生成告警，无自动动作 |
| 地理定位 | IP → 经纬度精确计算 | location 为字符串，无真实距离计算 |
| 决策透明度 | 风险因子可追溯 | 告警 details 为 JSON，无结构化因子 |

### 现有基础（可复用）

- `security_alerts` 表：完整的告警生命周期
- `SecurityDetectionService`：多向量攻击检测
- `login_events` 表：IP、user_agent、device_type、location
- `malicious_ip_blacklist` + `tenant_malicious_ip_blacklist`：IP 黑名单
- Webhook 告警推送

---

## 期望行为

### R1: IP 地理定位集成

引入 MaxMind GeoLite2 离线数据库，将 IP 地址解析为结构化地理信息：

```rust
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub country_code: String,       // ISO 3166-1 alpha-2
    pub country_name: String,
    pub city: Option<String>,
    pub accuracy_radius_km: u16,
}
```

- 使用 `maxminddb` crate 解析 `.mmdb` 文件
- 不可能旅行检测改用 Haversine 公式计算真实距离
- `login_events` 表新增 `latitude`、`longitude`、`country_code` 字段
- GeoLite2 数据库通过配置路径加载，支持定期更新

**涉及文件**:
- `auth9-core/src/domains/security_observability/service/geo.rs` — 新增地理定位服务
- `auth9-core/src/domains/security_observability/service/security_detection.rs` — 替换字符串比较为距离计算
- `auth9-core/migrations/` — login_events 表新增地理字段

### R2: 用户行为基线

为每个用户维护登录行为画像，存储于 Redis（热数据）+ 数据库（冷备份）：

```rust
pub struct UserLoginProfile {
    pub user_id: String,
    pub known_ips: Vec<String>,           // 最近 90 天使用过的 IP
    pub known_devices: Vec<String>,        // 已知设备指纹
    pub known_countries: Vec<String>,      // 常用国家
    pub typical_login_hours: Vec<u8>,      // 常用登录时段 (0-23)
    pub avg_login_frequency: f64,          // 日均登录次数
    pub last_updated: DateTime<Utc>,
}
```

- 每次成功登录后异步更新画像（不阻塞登录流程）
- 画像数据保留 90 天滑动窗口
- 新用户在前 10 次登录期间处于"学习期"，不产生行为异常告警

**涉及文件**:
- `auth9-core/src/domains/security_observability/service/user_profile.rs` — 新增用户画像服务
- `auth9-core/migrations/` — 新增 `user_login_profiles` 表

### R3: 风险评分引擎

每次登录尝试计算 0-100 的风险分，基于多因子加权：

| 因子 | 权重 | 计算方式 |
|------|------|---------|
| IP 信誉 | 20 | 黑名单命中 = 100，新 IP = 40，已知 IP = 0 |
| 地理异常 | 20 | 新国家 = 60，不可能旅行 = 100，已知地点 = 0 |
| 设备异常 | 15 | 新设备 = 50，已知设备 = 0 |
| 时间异常 | 10 | 非典型时段 = 40，典型时段 = 0 |
| 失败历史 | 20 | 近期失败次数线性映射 |
| 账户状态 | 15 | 近期密码重置 = 30，MFA 变更 = 20 |

```rust
pub struct RiskAssessment {
    pub score: u8,                          // 0-100
    pub level: RiskLevel,                   // Low(0-25), Medium(26-50), High(51-75), Critical(76-100)
    pub factors: Vec<RiskFactor>,           // 可追溯的因子列表
    pub recommended_action: RiskAction,     // Allow, StepUpMFA, Block, Alert
    pub assessed_at: DateTime<Utc>,
}

pub struct RiskFactor {
    pub name: String,
    pub score: u8,
    pub weight: f64,
    pub detail: String,                     // 人类可读的说明
}
```

- 风险评分在认证流程中同步计算（< 5ms 目标延迟）
- 评分结果写入 `login_events` 表的 `risk_score` 字段
- 评分历史可通过 API 查询

**涉及文件**:
- `auth9-core/src/domains/security_observability/service/risk_engine.rs` — 新增风险评分引擎
- `auth9-core/src/domains/security_observability/service/security_detection.rs` — 集成风险引擎
- `auth9-core/migrations/` — login_events 表新增 `risk_score` 字段

### R4: 自动响应机制

根据风险评分自动执行响应动作：

| 风险等级 | 分数范围 | 自动动作 |
|---------|---------|---------|
| Low | 0-25 | 允许登录 |
| Medium | 26-50 | 允许登录，记录告警 |
| High | 51-75 | 要求 MFA 验证（step-up） |
| Critical | 76-100 | 阻断登录，通知管理员 |

自动响应可在租户级配置：

```rust
pub struct TenantRiskPolicy {
    pub tenant_id: String,
    pub mfa_threshold: u8,          // 默认 51，超过此分数要求 MFA
    pub block_threshold: u8,        // 默认 76，超过此分数阻断
    pub notify_admin: bool,         // 高风险时通知管理员
    pub auto_lock_account: bool,    // Critical 时自动锁定账户
}
```

**涉及文件**:
- `auth9-core/src/domains/security_observability/service/risk_response.rs` — 新增自动响应服务
- `auth9-core/src/domains/identity/api/auth/` — 登录流程集成风险检查
- `auth9-core/src/models/system_settings.rs` — 添加租户风险策略配置

### R5: Portal 安全仪表盘增强

在管理 Portal 中增强安全可视化：

1. **风险趋势图**：7/30 天风险分布趋势
2. **地理热力图**：登录来源地理分布（基于 R1 的经纬度数据）
3. **用户风险排行**：高风险用户 Top 10 列表
4. **实时告警面板**：未解决告警流式更新
5. **风险策略配置页**：管理员可调整阈值和响应动作

**涉及文件**:
- `auth9-portal/app/routes/dashboard.security.tsx` — 安全仪表盘页面
- `auth9-core/src/domains/security_observability/api/` — 新增统计 API 端点

### R6: 单元测试覆盖

- GeoLocation 解析：各类 IP 格式和 fallback 处理
- Haversine 距离计算：已知坐标对精度验证
- 用户画像更新：滑动窗口淘汰、学习期逻辑
- 风险评分引擎：各因子边界值、权重计算正确性
- 自动响应：阈值触发验证、租户级配置覆盖
- 使用 MockRepository，不依赖外部服务

---

## 安全考量

1. **GeoIP 数据准确性**: MaxMind GeoLite2 在城市级别约 50-80% 准确度，VPN/代理用户可能产生误判
2. **用户画像隐私**: 行为画像数据属于个人信息，需遵守 GDPR/CCPA 数据最小化原则，提供清除 API
3. **自动阻断误伤**: Critical 级别自动阻断可能影响合法用户，建议提供自助解锁通道（如邮箱验证）
4. **风险评分可解释性**: 每次评估记录完整因子链，便于管理员审计和用户申诉

---

## 验证方法

### 代码验证

```bash
# 搜索风险评分相关实现
grep -r "RiskEngine\|risk_score\|RiskAssessment" auth9-core/src/

# 搜索地理定位相关实现
grep -r "GeoLocation\|haversine\|maxminddb" auth9-core/src/

# 运行测试
cd auth9-core && cargo test risk
cd auth9-core && cargo test geo
```

### 手动验证

1. 使用已知坐标的 IP 登录，验证地理定位解析正确
2. 从新设备/新地点登录，验证风险分上升
3. 模拟暴力破解场景，验证 Critical 级别自动阻断
4. 在 Portal 安全仪表盘查看风险趋势和告警面板
5. 调整租户风险策略阈值，验证响应动作变化

---

## 实现顺序

建议按以下顺序实施：

1. **R1: IP 地理定位** — 基础能力，其他模块依赖
2. **R2: 用户行为基线** — 风险评分所需的历史数据
3. **R3: 风险评分引擎** — 核心能力
4. **R4: 自动响应机制** — 评分落地
5. **R5: Portal 仪表盘** — 可视化
6. **R6: 测试覆盖** — 贯穿各阶段

---

## 参考

- 现有安全检测服务: `auth9-core/src/domains/security_observability/service/security_detection.rs`
- 现有告警模型: `auth9-core/migrations/20260202000007_create_security_alerts.sql`
- 现有登录事件: `auth9-core/migrations/20260202000005_create_login_events.sql`
- MaxMind GeoLite2: https://dev.maxmind.com/geoip/geolite2-free-geolocation-data
- OWASP Threat Modeling: https://owasp.org/www-community/Threat_Modeling
