# MFA: 种子 MFA 测试用户验证

> **模块**: auth (MFA)
> **前置条件**: `./scripts/reset-docker.sh` 已执行，MFA 测试用户已种入
> **涉及端点**:
> - `POST /api/v1/hosted-login/password`
> - `POST /api/v1/mfa/challenge/totp`
> - `POST /api/v1/mfa/challenge/recovery-code`
> - `GET /api/v1/mfa/recovery-codes/remaining`

---

## 场景 1: MFA 用户登录触发 TOTP 挑战

### 步骤 0: Gate Check

- 确认 MFA 用户存在且 mfa_enabled=TRUE:
```sql
SELECT id, email, mfa_enabled FROM users WHERE email='mfa-user@auth9.local';
-- 预期: mfa_enabled = 1
```
- 确认 TOTP credential 存在:
```sql
SELECT COUNT(*) FROM credentials
WHERE user_id = (SELECT id FROM users WHERE email='mfa-user@auth9.local')
  AND credential_type = 'totp';
-- 预期: 1
```

### 步骤 1: 密码认证

```bash
curl -s -X POST http://localhost:8080/api/v1/hosted-login/password \
  -H "Content-Type: application/json" \
  -d '{"email":"mfa-user@auth9.local","password":"SecurePass123!"}' | jq .
```

**预期**:
- HTTP 200
- `mfa_required: true`
- `mfa_methods` 包含 `"totp"`
- 返回 `mfa_session_token`

### 步骤 2: 使用确定性 TOTP 码完成验证

生成 TOTP 码（使用种子用户的固定 secret）:
```bash
TOTP_CODE=$(node -e "
const crypto = require('crypto');
const alpha = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
const b32 = 'JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP';
let bits = '';
for (const c of b32) bits += alpha.indexOf(c).toString(2).padStart(5, '0');
const bytes = [];
for (let i = 0; i + 8 <= bits.length; i += 8) bytes.push(parseInt(bits.substr(i, 8), 2));
const secret = Buffer.from(bytes);
const time = Math.floor(Date.now() / 1000 / 30);
const buf = Buffer.alloc(8);
buf.writeBigUInt64BE(BigInt(time));
const hmac = crypto.createHmac('sha1', secret).update(buf).digest();
const offset = hmac[hmac.length - 1] & 0x0f;
const code = ((hmac.readUInt32BE(offset) & 0x7fffffff) % 1000000).toString().padStart(6, '0');
console.log(code);
")
echo "Generated TOTP code: $TOTP_CODE"
```

提交 TOTP 码:
```bash
curl -s -X POST http://localhost:8080/api/v1/mfa/challenge/totp \
  -H "Content-Type: application/json" \
  -d "{\"mfa_session_token\":\"<MFA_SESSION_TOKEN>\",\"code\":\"$TOTP_CODE\"}" | jq .
```

**预期**:
- HTTP 200
- 返回 `access_token`（JWT）
- `token_type: "Bearer"`

---

## 场景 2: 使用恢复码完成 MFA 挑战

### 步骤 0: Gate Check

- 确认恢复码存在且未使用:
```sql
SELECT COUNT(*) FROM credentials
WHERE user_id = (SELECT id FROM users WHERE email='mfa-user@auth9.local')
  AND credential_type = 'recovery_code'
  AND JSON_EXTRACT(credential_data, '$.used') = false;
-- 预期: 8（全部未使用）
```

### 步骤 1: 密码认证（获取 MFA session）

```bash
curl -s -X POST http://localhost:8080/api/v1/hosted-login/password \
  -H "Content-Type: application/json" \
  -d '{"email":"mfa-user@auth9.local","password":"SecurePass123!"}' | jq .
```

### 步骤 2: 使用恢复码 `rc-test-0001` 完成验证

```bash
curl -s -X POST http://localhost:8080/api/v1/mfa/challenge/recovery-code \
  -H "Content-Type: application/json" \
  -d '{"mfa_session_token":"<MFA_SESSION_TOKEN>","code":"rc-test-0001"}' | jq .
```

**预期**:
- HTTP 200
- 返回 `access_token`

### 步骤 3: 确认恢复码已被消耗

```sql
SELECT JSON_EXTRACT(credential_data, '$.used') as used
FROM credentials
WHERE user_id = (SELECT id FROM users WHERE email='mfa-user@auth9.local')
  AND credential_type = 'recovery_code'
  AND JSON_EXTRACT(credential_data, '$.code_hash') = SHA2('rc-test-0001', 256);
-- 预期: used = true
```

### 预期数据状态

```sql
SELECT COUNT(*) FROM credentials
WHERE user_id = (SELECT id FROM users WHERE email='mfa-user@auth9.local')
  AND credential_type = 'recovery_code'
  AND JSON_EXTRACT(credential_data, '$.used') = false;
-- 预期: 7（一个已消耗）
```

---

## 场景 3: 种子数据幂等性

### 步骤 1: 执行两次 reset-docker.sh

```bash
./scripts/reset-docker.sh
./scripts/reset-docker.sh
```

### 步骤 2: 验证 MFA 用户仅一份

```sql
SELECT COUNT(*) FROM users WHERE email='mfa-user@auth9.local';
-- 预期: 1

SELECT COUNT(*) FROM credentials
WHERE user_id = (SELECT id FROM users WHERE email='mfa-user@auth9.local');
-- 预期: 10 (1 password + 1 totp + 8 recovery codes)
```

**预期**: 无重复记录，凭证数量固定为 10。
