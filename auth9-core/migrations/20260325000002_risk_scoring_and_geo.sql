-- FR-004: Enhanced anomaly detection — risk scoring engine & auto response
-- Adds GeoIP columns to login_events, user login profiles, tenant risk policies,
-- and new alert type for high-risk logins.

-- 1. Add geo columns and risk score to login_events
ALTER TABLE login_events
  ADD COLUMN latitude DOUBLE NULL,
  ADD COLUMN longitude DOUBLE NULL,
  ADD COLUMN country_code VARCHAR(2) NULL,
  ADD COLUMN risk_score TINYINT UNSIGNED NULL;

-- 2. User login profiles (behavioral baseline for anomaly detection)
CREATE TABLE IF NOT EXISTS user_login_profiles (
  id CHAR(36) PRIMARY KEY,
  user_id CHAR(36) NOT NULL,
  known_ips JSON NOT NULL,
  known_devices JSON NOT NULL,
  known_countries JSON NOT NULL,
  typical_login_hours JSON NOT NULL,
  avg_login_frequency DOUBLE NOT NULL DEFAULT 0,
  total_logins INT NOT NULL DEFAULT 0,
  last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX idx_user_login_profiles_user_id (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- 3. Tenant risk policies (configurable thresholds per tenant)
CREATE TABLE IF NOT EXISTS tenant_risk_policies (
  id CHAR(36) PRIMARY KEY,
  tenant_id CHAR(36) NOT NULL,
  mfa_threshold TINYINT UNSIGNED NOT NULL DEFAULT 51,
  block_threshold TINYINT UNSIGNED NOT NULL DEFAULT 76,
  notify_admin BOOLEAN NOT NULL DEFAULT TRUE,
  auto_lock_account BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  UNIQUE INDEX idx_tenant_risk_policies_tenant_id (tenant_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- 4. Extend security_alerts alert_type to include high_risk_login
ALTER TABLE security_alerts MODIFY COLUMN alert_type
  ENUM('brute_force','slow_brute_force','password_spray','new_device',
       'impossible_travel','suspicious_ip','high_risk_login') NOT NULL;
