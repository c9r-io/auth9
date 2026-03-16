export interface SendEmailOtpInput {
  email: string;
}

export interface VerifyEmailOtpInput {
  email: string;
  code: string;
}
