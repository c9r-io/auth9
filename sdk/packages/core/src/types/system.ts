export interface SmtpConfig {
  type: "smtp";
  host: string;
  port: number;
  username: string;
  password: string;
  useTls: boolean;
  fromEmail: string;
  fromName?: string;
}

export interface SesConfig {
  type: "ses";
  region: string;
  accessKeyId: string;
  secretAccessKey: string;
  fromEmail: string;
  fromName?: string;
}

export interface OracleConfig {
  type: "oracle";
  smtpEndpoint: string;
  port: number;
  username: string;
  password: string;
  fromEmail: string;
  fromName?: string;
}

export interface NoneConfig {
  type: "none";
}

export type EmailProviderConfig =
  | SmtpConfig
  | SesConfig
  | OracleConfig
  | NoneConfig;

export interface EmailSettings {
  config: EmailProviderConfig;
}

export interface UpdateEmailSettingsInput {
  config: EmailProviderConfig;
}

export interface TestEmailResponse {
  success: boolean;
  message: string;
  messageId?: string;
}

export interface SendTestEmailInput {
  toEmail: string;
}
