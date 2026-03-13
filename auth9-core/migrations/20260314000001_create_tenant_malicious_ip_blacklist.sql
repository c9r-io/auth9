CREATE TABLE IF NOT EXISTS tenant_malicious_ip_blacklist (
    id CHAR(36) PRIMARY KEY,
    tenant_id CHAR(36) NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    reason VARCHAR(255) NULL,
    created_by CHAR(36) NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    UNIQUE INDEX idx_tenant_malicious_ip_blacklist_tenant_ip (tenant_id, ip_address),
    INDEX idx_tenant_malicious_ip_blacklist_tenant_id (tenant_id),
    INDEX idx_tenant_malicious_ip_blacklist_created_by (created_by),
    INDEX idx_tenant_malicious_ip_blacklist_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
