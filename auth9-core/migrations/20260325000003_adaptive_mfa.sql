-- FR-005: Adaptive MFA (risk-driven multi-factor authentication)
-- Adds trusted devices and per-tenant adaptive MFA policies.

-- 1. Trusted devices (allow users to skip MFA on known devices)
CREATE TABLE IF NOT EXISTS trusted_devices (
  id CHAR(36) PRIMARY KEY,
  user_id CHAR(36) NOT NULL,
  tenant_id CHAR(36),
  device_fingerprint VARCHAR(64) NOT NULL,
  device_name VARCHAR(255),
  trusted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP NOT NULL,
  last_used_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  revoked BOOLEAN NOT NULL DEFAULT FALSE,
  INDEX idx_trusted_devices_user (user_id),
  INDEX idx_trusted_devices_lookup (user_id, device_fingerprint),
  INDEX idx_trusted_devices_expires (expires_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- 2. Adaptive MFA policies (per-tenant MFA mode and thresholds)
CREATE TABLE IF NOT EXISTS adaptive_mfa_policies (
  id CHAR(36) PRIMARY KEY,
  tenant_id CHAR(36) NOT NULL,
  mode VARCHAR(20) NOT NULL DEFAULT 'always',
  risk_threshold TINYINT UNSIGNED NOT NULL DEFAULT 40,
  always_require_for_admins BOOLEAN NOT NULL DEFAULT TRUE,
  trust_device_days SMALLINT UNSIGNED NOT NULL DEFAULT 30,
  step_up_operations JSON NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  UNIQUE INDEX idx_adaptive_mfa_policies_tenant (tenant_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
