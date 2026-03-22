-- Add per-user email OTP enabled flag
ALTER TABLE users ADD COLUMN email_otp_enabled BOOLEAN NOT NULL DEFAULT FALSE;
