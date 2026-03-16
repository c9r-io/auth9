# 集成测试 - OTP 通用服务层基础设施

**模块**: 集成测试
**测试范围**: OtpManager 验证码生成/存储/验证、OtpChannel 通道抽象、速率限制、CacheOperations OTP 扩展
**场景数**: 5
**优先级**: 高

---

## 背景说明

OTP 通用服务层是 Email OTP 和 SMS OTP 功能的共享基础设施，位于 `auth9-core/src/domains/identity/service/otp/`。

核心组件：
- **OtpManager**: 验证码生成（6 位密码学安全随机数）、Redis 存储（带 TTL）、一次性验证消费
- **OtpChannel trait**: 发送通道抽象（Email / SMS），支持 mockall 自动 mock
- **EmailOtpChannel**: 封装 EmailService + EmailMfa 模板的邮件发送实现
- **SmsOtpChannel**: 占位实现（返回 "not yet implemented" 错误）
- **OtpRateLimitConfig**: 冷却期、日发送上限、失败锁定的通道级配置
- **CacheOperations 扩展**: 6 个新方法 (`store_otp`, `get_otp`, `remove_otp`, `increment_counter`, `get_counter`, `set_flag`)

Redis Key 设计：

| Key 模式 | 用途 | TTL |
|----------|------|-----|
| `auth9:otp:{channel}:{destination}` | 存储验证码 | Email 10min, SMS 5min |
| `auth9:otp_cooldown:{channel}:{destination}` | 发送冷却期 | Email 60s, SMS 120s |
| `auth9:otp_daily:{channel}:{destination}` | 24 小时发送计数 | 24h |
| `auth9:otp_fail:{channel}:{destination}` | 连续失败计数 | Email 15min, SMS 30min |

> **注意**: 本 FR 为纯后端基础设施层，无 REST API 端点和 UI。验证通过单元测试、代码结构检查和编译检查完成。

---

## 场景 1：单元测试全部通过

### 初始状态
- auth9-core 代码已编译
- OTP 模块文件存在于 `src/domains/identity/service/otp/`

### 目的
验证 OTP 服务层的 14 个单元测试全部通过，覆盖生成、存储、验证、速率限制

### 测试操作流程
1. 运行 OTP 相关单元测试：
   ```bash
   cd auth9-core && cargo test otp -- --nocapture 2>&1
   ```
2. 检查测试结果输出

### 预期结果
- 所有测试通过，输出包含 `test result: ok`
- 测试列表包含以下 14 个测试：
  - `otp::channel::tests::test_channel_type_display`
  - `otp::channel::tests::test_channel_type_equality`
  - `otp::manager::tests::test_generate_code_six_digits`
  - `otp::manager::tests::test_generate_code_unique`
  - `otp::manager::tests::test_store_and_verify_success`
  - `otp::manager::tests::test_verify_wrong_code`
  - `otp::manager::tests::test_verify_consumed_code`
  - `otp::manager::tests::test_verify_no_code_stored`
  - `otp::manager::tests::test_rate_limit_cooldown`
  - `otp::manager::tests::test_rate_limit_daily_max`
  - `otp::manager::tests::test_rate_limit_failure_lockout`
  - `otp::manager::tests::test_different_channels_isolated`
  - `otp::rate_limit::tests::test_email_defaults`
  - `otp::rate_limit::tests::test_sms_defaults`
- 无编译警告或错误

---

## 场景 2：OTP 模块代码结构完整性

### 初始状态
- auth9-core 源码可访问

### 目的
验证 OTP 模块文件结构符合 FR 规范，所有组件正确导出

### 测试操作流程
1. 验证模块文件结构：
   ```bash
   ls -la auth9-core/src/domains/identity/service/otp/
   ```
2. 验证模块导出：
   ```bash
   grep -n "pub use\|pub mod" auth9-core/src/domains/identity/service/otp/mod.rs
   ```
3. 验证 identity service 层注册了 OTP 模块：
   ```bash
   grep -n "otp" auth9-core/src/domains/identity/service/mod.rs
   ```

### 预期结果
- OTP 目录包含 6 个文件：`mod.rs`, `channel.rs`, `email_channel.rs`, `sms_channel.rs`, `manager.rs`, `rate_limit.rs`
- `mod.rs` 导出：`OtpChannel`, `OtpChannelType`, `OtpManager`, `OtpRateLimitConfig`, `EmailOtpChannel`, `SmsOtpChannel`
- identity service `mod.rs` 包含 `pub mod otp` 和对应 `pub use` 语句

---

## 场景 3：CacheOperations 扩展完整性

### 初始状态
- auth9-core 源码可访问

### 目的
验证 CacheOperations trait 正确扩展了 6 个 OTP 方法，Redis 和 NoOp 实现均已同步

### 测试操作流程
1. 验证 trait 定义包含 OTP 方法：
   ```bash
   grep -c "async fn.*otp\|async fn.*counter\|async fn.*flag" auth9-core/src/cache/mod.rs
   ```
2. 验证 Redis 实现（CacheManager）：
   ```bash
   grep -c "store_otp\|get_otp\|remove_otp\|increment_counter\|get_counter\|set_flag" auth9-core/src/cache/manager.rs
   ```
3. 验证 NoOp 实现（NoOpCacheManager）：
   ```bash
   grep -c "store_otp\|get_otp\|remove_otp\|increment_counter\|get_counter\|set_flag" auth9-core/src/cache/noop.rs
   ```
4. 验证 Key 常量已定义：
   ```bash
   grep "OTP" auth9-core/src/cache/mod.rs
   ```
5. 运行 cache 模块测试确认无回归：
   ```bash
   cd auth9-core && cargo test cache 2>&1
   ```

### 预期结果
- trait 中有 6 个 OTP 相关方法签名
- CacheManager 和 NoOpCacheManager 各有 6 个方法实现
- manager_ops.rs 和 noop_ops.rs 各有 6 个委托方法
- Key 常量包含：`OTP`, `OTP_COOLDOWN`, `OTP_DAILY`, `OTP_FAIL`
- cache 测试全部通过，无回归

---

## 场景 4：速率限制默认配置正确性

### 初始状态
- auth9-core 源码可访问

### 目的
验证 Email 和 SMS 的速率限制默认值符合 FR 规范

### 测试操作流程
1. 检查 Email 默认配置：
   ```bash
   grep -A 8 "fn email_defaults" auth9-core/src/domains/identity/service/otp/rate_limit.rs
   ```
2. 检查 SMS 默认配置：
   ```bash
   grep -A 8 "fn sms_defaults" auth9-core/src/domains/identity/service/otp/rate_limit.rs
   ```
3. 运行速率限制测试：
   ```bash
   cd auth9-core && cargo test rate_limit 2>&1
   ```

### 预期结果
- Email 默认值：`cooldown_secs=60`, `daily_max=10`, `max_failures=5`, `lockout_secs=900`
- SMS 默认值：`cooldown_secs=120`, `daily_max=5`, `max_failures=3`, `lockout_secs=1800`
- 速率限制单元测试通过

---

## 场景 5：编译和 Lint 无错误

### 初始状态
- auth9-core 代码未修改

### 目的
验证 OTP 模块集成后项目编译和 Lint 无任何错误或警告

### 测试操作流程
1. 执行编译检查：
   ```bash
   cd auth9-core && cargo check 2>&1
   ```
2. 执行 Clippy Lint：
   ```bash
   cd auth9-core && cargo clippy 2>&1
   ```
3. 执行格式检查：
   ```bash
   cd auth9-core && cargo fmt --check 2>&1
   ```

### 预期结果
- `cargo check` 输出 `Finished` 无错误
- `cargo clippy` 输出 `Finished` 无警告
- `cargo fmt --check` 无格式差异（退出码 0）

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | 单元测试全部通过 | ✅ PASS | 2026-03-16 | QA Test | 14 个 OTP 测试全部通过 |
| 2 | OTP 模块代码结构完整性 | ✅ PASS | 2026-03-16 | QA Test | 6 个文件齐全 |
| 3 | CacheOperations 扩展完整性 | ✅ PASS | 2026-03-16 | QA Test | 6 个新方法，无回归 |
| 4 | 速率限制默认配置正确性 | ✅ PASS | 2026-03-16 | QA Test | Email/SMS 默认值符合规范 |
| 5 | 编译和 Lint 无错误 | ✅ PASS | 2026-03-16 | QA Test | check/clippy/fmt 全部通过 |
