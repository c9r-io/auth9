# SDK Phase 3: 认证流程与凭证管理 API 客户端

**类型**: 功能增强
**严重程度**: Medium
**影响范围**: sdk/packages/core (@auth9/core), sdk/packages/node (@auth9/node)
**前置依赖**: `sdk_phase1_core_management.md`

---

## 背景

Phase 3 覆盖认证相关的 API：密码管理（重置/策略）、WebAuthn/Passkey、Email OTP、OAuth 流程辅助、Organization 端点。这些 API 通常由前端应用或 BFF 层调用，SDK 封装可简化集成。

### 注意事项

部分端点（authorize、callback）是浏览器重定向流程，SDK 仅提供 URL 构建辅助而非直接调用。

---

## 期望行为

### R1: Password 子客户端

封装密码管理端点：

```typescript
// 未认证端点
client.password.forgotPassword(input: ForgotPasswordInput): Promise<void>
client.password.resetPassword(input: ResetPasswordInput): Promise<void>

// 已认证端点
client.password.changeMyPassword(input: ChangePasswordInput): Promise<void>
client.password.adminSetPassword(userId: string, input: AdminSetPasswordInput): Promise<void>
client.password.getPolicy(tenantId: string): Promise<PasswordPolicy>
client.password.updatePolicy(tenantId: string, input: UpdatePasswordPolicyInput): Promise<PasswordPolicy>
```

新增类型：

```typescript
interface PasswordPolicy {
  minLength: number;
  requireUppercase: boolean;
  requireLowercase: boolean;
  requireNumbers: boolean;
  requireSpecialChars: boolean;
  maxAge?: number; // days
  historyCount?: number;
}

interface ForgotPasswordInput { email: string; }
interface ResetPasswordInput { token: string; newPassword: string; }
interface ChangePasswordInput { currentPassword: string; newPassword: string; }
interface AdminSetPasswordInput { password: string; temporary?: boolean; }
```

**涉及文件**:
- `sdk/packages/core/src/clients/password.ts` — 新增
- `sdk/packages/core/src/types/password.ts` — 新增

### R2: Passkeys (WebAuthn) 子客户端

封装 WebAuthn 端点：

```typescript
// 已认证 — 管理 passkeys
client.passkeys.list(): Promise<Passkey[]>
client.passkeys.delete(id: string): Promise<void>
client.passkeys.startRegistration(): Promise<PasskeyRegistrationOptions>
client.passkeys.completeRegistration(input: PasskeyRegistrationResult): Promise<Passkey>

// 未认证 — passkey 登录
client.passkeys.startAuthentication(input?: PasskeyAuthStartInput): Promise<PasskeyAuthenticationOptions>
client.passkeys.completeAuthentication(input: PasskeyAuthenticationResult): Promise<AuthTokenResponse>
```

新增类型：

```typescript
interface Passkey {
  id: string;
  name?: string;
  credentialId: string;
  createdAt: string;
  lastUsedAt?: string;
}

// WebAuthn options/result 类型遵循 W3C WebAuthn spec
interface PasskeyRegistrationOptions {
  publicKey: PublicKeyCredentialCreationOptions;
}

interface PasskeyAuthenticationOptions {
  publicKey: PublicKeyCredentialRequestOptions;
}
```

**涉及文件**:
- `sdk/packages/core/src/clients/passkeys.ts` — 新增
- `sdk/packages/core/src/types/passkey.ts` — 新增

### R3: Email OTP 子客户端

封装无密码邮箱登录端点：

```typescript
client.emailOtp.send(input: SendEmailOtpInput): Promise<void>
client.emailOtp.verify(input: VerifyEmailOtpInput): Promise<AuthTokenResponse>
```

```typescript
interface SendEmailOtpInput { email: string; }
interface VerifyEmailOtpInput { email: string; code: string; }
interface AuthTokenResponse {
  accessToken: string;
  tokenType: string;
  expiresIn: number;
  refreshToken?: string;
}
```

**涉及文件**:
- `sdk/packages/core/src/clients/email-otp.ts` — 新增
- `sdk/packages/core/src/types/email-otp.ts` — 新增

### R4: Auth 流程辅助

提供 OAuth/OIDC 流程的 URL 构建和 token 管理方法：

```typescript
// URL 构建（不发 HTTP 请求）
client.auth.getAuthorizeUrl(options: AuthorizeOptions): string
client.auth.getLogoutUrl(options?: LogoutOptions): string

// Token 端点
client.auth.exchangeTenantToken(input: TenantTokenInput): Promise<AuthTokenResponse>
client.auth.getUserInfo(): Promise<UserInfo>

// Enterprise SSO discovery
client.auth.discoverEnterpriseSso(input: SsoDiscoveryInput): Promise<SsoDiscoveryResult>
```

```typescript
interface AuthorizeOptions {
  redirectUri: string;
  scope?: string;
  state?: string;
  responseType?: string;
  tenantId?: string;
}
```

**涉及文件**:
- `sdk/packages/core/src/clients/auth.ts` — 新增
- `sdk/packages/core/src/types/auth.ts` — 新增

### R5: Organizations 子客户端

封装 Organization 端点：

```typescript
client.organizations.create(input: CreateOrganizationInput): Promise<Organization>
client.organizations.getMyTenants(): Promise<Tenant[]>
```

**涉及文件**:
- `sdk/packages/core/src/clients/organizations.ts` — 新增

---

## 验收标准

- [ ] 5 个子客户端全部实现
- [ ] WebAuthn 类型与 W3C spec 对齐
- [ ] Auth URL 构建方法不发送 HTTP 请求，仅拼接 URL
- [ ] 单元测试覆盖所有方法
- [ ] `npm run build` 通过
