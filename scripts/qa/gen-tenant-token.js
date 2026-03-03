const jwt = require('jsonwebtoken');
const fs = require('fs');
const path = require('path');

const privateKeyPath = path.join(__dirname, '../../deploy/dev-certs/jwt/private.key');
const privateKey = fs.readFileSync(privateKeyPath, 'utf8');

// Get user and tenant IDs from database
const { execSync } = require('child_process');

let userId, tenantId;
try {
  userId = execSync(
    `mysql -h 127.0.0.1 -P 4000 -u root auth9 -N -e "SELECT id FROM users WHERE email = 'admin@auth9.local' LIMIT 1;"`,
    { encoding: 'utf8' }
  ).trim();
} catch (e) {
  console.error("Failed to get user ID:", e.message);
  process.exit(1);
}

try {
  tenantId = execSync(
    `mysql -h 127.0.0.1 -P 4000 -u root auth9 -N -e "SELECT id FROM tenants WHERE slug = 'demo' LIMIT 1;"`,
    { encoding: 'utf8' }
  ).trim();
} catch (e) {
  console.error("Failed to get tenant ID:", e.message);
  process.exit(1);
}

console.error("User ID:", userId);
console.error("Tenant ID:", tenantId);

const now = Math.floor(Date.now() / 1000);
const payload = {
  sub: userId,
  email: "admin@auth9.local",
  name: "Admin User",
  iss: "http://localhost:8080",
  aud: "auth9-portal",
  token_type: "tenant_access",
  tenant_id: tenantId,
  roles: ["owner", "admin"],
  permissions: ["action:read", "action:write", "service:read", "service:write"],
  iat: now,
  exp: now + 3600
};

const token = jwt.sign(payload, privateKey, { algorithm: 'RS256', keyid: 'auth9-current' });
process.stdout.write(token);
