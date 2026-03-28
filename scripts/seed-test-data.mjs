#!/usr/bin/env node
/**
 * Seed test data into the auth9 database.
 *
 * Called by reset-docker.sh AFTER all services are healthy. Seeds:
 *
 * 1. MFA test user (mfa-user@auth9.local) with deterministic TOTP + recovery codes
 * 2. PKCE test client (auth9-qa-test) — public client for QA OIDC testing
 */

import { execSync } from "node:child_process";
import { createCipheriv, createHash, randomBytes } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const PROJECT_DIR = resolve(__dirname, "..");

// ── Constants ──────────────────────────────────────────────────────────────────

const MFA_USER_EMAIL = "mfa-user@auth9.local";
const MFA_USER_DISPLAY_NAME = "MFA Test User";
const MFA_USER_IDENTITY_SUBJECT = `seed-mfa-${MFA_USER_EMAIL}`;
// 20-byte (160-bit) deterministic TOTP secret — meets minimum 128-bit requirement
const TOTP_SECRET_BASE32 = "JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP";
const RECOVERY_CODES = [
  "rc-test-0001",
  "rc-test-0002",
  "rc-test-0003",
  "rc-test-0004",
  "rc-test-0005",
  "rc-test-0006",
  "rc-test-0007",
  "rc-test-0008",
];

// ── Helpers ────────────────────────────────────────────────────────────────────

function sql(query) {
  // Execute SQL via host mysql client connecting to TiDB on localhost:4000
  return execSync(
    `mysql -h 127.0.0.1 -P 4000 -u root auth9 -N`,
    { input: query + ";", encoding: "utf8", timeout: 10_000 }
  ).trim();
}

function uuid() {
  return crypto.randomUUID();
}

/**
 * AES-256-GCM encrypt matching auth9-core's crypto::encrypt format.
 * Output: base64(nonce):base64(ciphertext + authTag)
 */
function aes256gcmEncrypt(keyBase64, plaintext) {
  const key = Buffer.from(keyBase64, "base64");
  if (key.length !== 32) {
    throw new Error(`Invalid key length: ${key.length} (expected 32)`);
  }
  const nonce = randomBytes(12);
  const cipher = createCipheriv("aes-256-gcm", key, nonce);
  const ciphertext = Buffer.concat([
    cipher.update(plaintext, "utf8"),
    cipher.final(),
  ]);
  const authTag = cipher.getAuthTag(); // 16 bytes
  // auth9-core format: nonce_b64:concat(ciphertext, authTag)_b64
  return `${nonce.toString("base64")}:${Buffer.concat([ciphertext, authTag]).toString("base64")}`;
}

function sha256hex(input) {
  return createHash("sha256").update(input).digest("hex");
}

function readEncryptionKey() {
  // Try env var first
  if (process.env.SETTINGS_ENCRYPTION_KEY) {
    return process.env.SETTINGS_ENCRYPTION_KEY;
  }
  // Read from .env file
  try {
    const envFile = readFileSync(resolve(PROJECT_DIR, ".env"), "utf8");
    const match = envFile.match(/^SETTINGS_ENCRYPTION_KEY=(.+)$/m);
    if (match) return match[1].trim();
  } catch { /* ignore */ }

  console.warn("  SETTINGS_ENCRYPTION_KEY not found, using zero key (dev fallback)");
  return Buffer.alloc(32).toString("base64");
}

function escapeSQL(str) {
  return str.replace(/\\/g, "\\\\").replace(/'/g, "\\'");
}

// ── Main ───────────────────────────────────────────────────────────────────────

try {
  // 1. Check if MFA user already exists with correct state
  let mfaSkip = false;
  const existing = sql(
    `SELECT id, mfa_enabled FROM users WHERE email = '${MFA_USER_EMAIL}' LIMIT 1`
  );
  if (existing) {
    const [existingId, mfaEnabled] = existing.split("\t");
    if (mfaEnabled === "1") {
      const totpCount = sql(
        `SELECT COUNT(*) FROM credentials WHERE user_id = '${existingId}' AND credential_type = 'totp'`
      );
      const rcCount = sql(
        `SELECT COUNT(*) FROM credentials WHERE user_id = '${existingId}' AND credential_type = 'recovery_code'`
      );
      if (parseInt(totpCount) > 0 && parseInt(rcCount) >= 8) {
        console.log("  MFA test user already exists with credentials, skipping.");
        mfaSkip = true;
      }
    }
  }

  if (!mfaSkip) {
  // 2. Get admin password hash (same password, avoid needing argon2 in Node)
  const adminHash = sql(
    `SELECT credential_data FROM credentials c
     JOIN users u ON c.user_id = u.id
     WHERE u.email = 'admin@auth9.local' AND c.credential_type = 'password'
     LIMIT 1`
  );
  if (!adminHash) {
    console.error("  ERROR: Admin password credential not found. Cannot seed MFA user.");
    process.exit(1);
  }

  // 3. Get tenant IDs
  const platformId = sql(
    "SELECT id FROM tenants WHERE slug = 'auth9-platform' LIMIT 1"
  );
  const demoId = sql(
    "SELECT id FROM tenants WHERE slug = 'demo' LIMIT 1"
  );
  if (!platformId || !demoId) {
    console.error("  ERROR: Platform or demo tenant not found.");
    process.exit(1);
  }

  // 4. Create/update MFA user
  const userId = existing ? existing.split("\t")[0] : uuid();
  if (!existing) {
    sql(`INSERT INTO users (id, identity_subject, email, display_name, mfa_enabled, created_at, updated_at)
         VALUES ('${userId}', '${MFA_USER_IDENTITY_SUBJECT}', '${MFA_USER_EMAIL}', '${MFA_USER_DISPLAY_NAME}', TRUE, NOW(), NOW())
         ON DUPLICATE KEY UPDATE mfa_enabled = TRUE, display_name = '${MFA_USER_DISPLAY_NAME}'`);
  } else {
    sql(`UPDATE users SET mfa_enabled = TRUE WHERE id = '${userId}'`);
  }

  // 5. Resolve actual user ID (ON DUPLICATE KEY may have used existing row)
  const actualUserId = sql(
    `SELECT id FROM users WHERE email = '${MFA_USER_EMAIL}' LIMIT 1`
  );

  // 6. Insert password credential (reuse admin's hash)
  sql(`DELETE FROM credentials WHERE user_id = '${actualUserId}' AND credential_type = 'password'`);
  sql(`INSERT INTO credentials (id, user_id, credential_type, credential_data)
       VALUES ('${uuid()}', '${actualUserId}', 'password', '${escapeSQL(adminHash)}')`);

  // 7. Insert TOTP credential
  const encryptionKey = readEncryptionKey();
  const encryptedSecret = aes256gcmEncrypt(encryptionKey, TOTP_SECRET_BASE32);
  const totpData = JSON.stringify({
    secret_encrypted: encryptedSecret,
    algorithm: "SHA1",
    digits: 6,
    period: 30,
  });
  sql(`DELETE FROM credentials WHERE user_id = '${actualUserId}' AND credential_type = 'totp'`);
  sql(`INSERT INTO credentials (id, user_id, credential_type, credential_data, user_label)
       VALUES ('${uuid()}', '${actualUserId}', 'totp', '${escapeSQL(totpData)}', 'TOTP')`);

  // 8. Insert recovery codes
  sql(`DELETE FROM credentials WHERE user_id = '${actualUserId}' AND credential_type = 'recovery_code'`);
  for (const code of RECOVERY_CODES) {
    const codeData = JSON.stringify({
      code_hash: sha256hex(code),
      used: false,
    });
    sql(`INSERT INTO credentials (id, user_id, credential_type, credential_data)
         VALUES ('${uuid()}', '${actualUserId}', 'recovery_code', '${escapeSQL(codeData)}')`);
  }

  // 9. Associate with tenants
  sql(`INSERT IGNORE INTO tenant_users (id, tenant_id, user_id, role_in_tenant, joined_at)
       VALUES ('${uuid()}', '${platformId}', '${actualUserId}', 'admin', NOW())`);
  sql(`INSERT IGNORE INTO tenant_users (id, tenant_id, user_id, role_in_tenant, joined_at)
       VALUES ('${uuid()}', '${demoId}', '${actualUserId}', 'admin', NOW())`);

  // 10. Verify
  const verify = sql(
    `SELECT COUNT(*) FROM credentials WHERE user_id = '${actualUserId}'`
  );
  console.log(`  MFA test user seeded: ${MFA_USER_EMAIL} (${verify} credentials)`);
  } // end if (!mfaSkip)
} catch (e) {
  console.error(`  ERROR seeding MFA test user: ${e.message}`);
}

// ── Part 2: PKCE Test Client ───────────────────────────────────────────────────
//
// Public OIDC client for QA testing. Uses an unlistened port (19876) as
// redirect_uri so authorization codes are NOT auto-consumed by any service.

const QA_CLIENT_ID = "auth9-qa-test";
const QA_REDIRECT_URI = "http://localhost:19876/callback";

try {
  const existing = sql(
    `SELECT client_id FROM clients WHERE client_id = '${QA_CLIENT_ID}' LIMIT 1`
  );
  if (existing) {
    console.log(`  QA test client already exists (${QA_CLIENT_ID}), skipping.`);
  } else {
    const serviceId = uuid();
    sql(`INSERT IGNORE INTO services (id, tenant_id, name, base_url, redirect_uris, logout_uris, status, created_at, updated_at)
         VALUES ('${serviceId}', NULL, 'Auth9 QA Test Service', 'http://localhost:19876',
                 '["${QA_REDIRECT_URI}"]', '[]', 'active', NOW(), NOW())`);
    sql(`INSERT IGNORE INTO clients (id, service_id, client_id, client_secret_hash, name, public_client, created_at)
         VALUES ('${uuid()}', '${serviceId}', '${QA_CLIENT_ID}', '', 'QA Test Client', 1, NOW())`);
    console.log(`  QA test client seeded: ${QA_CLIENT_ID} (public, PKCE-capable)`);
  }
} catch (e) {
  console.error(`  ERROR seeding QA test client: ${e.message}`);
  process.exit(1);
}
