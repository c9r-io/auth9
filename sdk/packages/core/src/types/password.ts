export interface PasswordPolicy {
  minLength: number;
  requireUppercase: boolean;
  requireLowercase: boolean;
  requireNumbers: boolean;
  requireSymbols: boolean;
  maxAgeDays: number;
  historyCount: number;
  lockoutThreshold: number;
  lockoutDurationMins: number;
}

export interface ForgotPasswordInput {
  email: string;
}

export interface ResetPasswordInput {
  token: string;
  newPassword: string;
}

export interface ChangePasswordInput {
  currentPassword: string;
  newPassword: string;
}

export interface AdminSetPasswordInput {
  password: string;
  temporary?: boolean;
}

export type UpdatePasswordPolicyInput = Partial<PasswordPolicy>;
