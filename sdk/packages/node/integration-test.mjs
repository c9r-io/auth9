#!/usr/bin/env node
import express from "express";
import { auth9Middleware, requirePermission, requireRole } from "@auth9/node/middleware/express";

const DOMAIN = "http://localhost:8080";
const AUDIENCE = "auth9-portal";
const PORT = 5000;

const app = express();
app.use(express.json());

// Test token - must be generated with valid signature from auth9-core's JWKS
// Using pre-generated token for QA testing
const TEST_TOKEN = process.env.TEST_TOKEN;

function createTestServer(optional = false) {
  const router = express.Router();

  router.get("/test", (req, res) => {
    res.json({
      userId: req.auth?.userId,
      email: req.auth?.email,
      tokenType: req.auth?.tokenType,
      tenantId: req.auth?.tenantId,
      roles: req.auth?.roles,
      permissions: req.auth?.permissions,
    });
  });

  router.get("/public-or-private", (req, res) => {
    if (req.auth) {
      res.json({ message: "Authenticated", user: req.auth.email });
    } else {
      res.json({ message: "Anonymous" });
    }
  });

  router.get("/users", requirePermission("user:read"), (req, res) => {
    res.json({ success: true });
  });

  router.post("/users", requirePermission(["user:read", "user:write"]), (req, res) => {
    res.json({ success: true });
  });

  router.delete("/users/:id", requirePermission("user:delete"), (req, res) => {
    res.json({ success: true });
  });

  router.patch("/users/:id", requirePermission(["user:write", "user:admin"], { mode: "any" }), (req, res) => {
    res.json({ success: true });
  });

  router.get("/admin", requireRole("admin"), (req, res) => {
    res.json({ success: true });
  });

  router.get("/superadmin", requireRole("superadmin"), (req, res) => {
    res.json({ success: true });
  });

  router.get("/any-admin", requireRole(["admin", "superadmin"], { mode: "any" }), (req, res) => {
    res.json({ success: true });
  });

  router.get("/check-helpers", (req, res) => {
    res.json({
      hasReadPerm: req.auth?.hasPermission("user:read"),
      hasDeletePerm: req.auth?.hasPermission("user:delete"),
      isAdmin: req.auth?.hasRole("admin"),
      isSuperAdmin: req.auth?.hasRole("superadmin"),
      hasAnyWritePerm: req.auth?.hasAnyPermission(["user:write", "user:admin"]),
      hasAllPerms: req.auth?.hasAllPermissions(["user:read", "user:write"]),
      hasAllPermsIncDelete: req.auth?.hasAllPermissions(["user:read", "user:delete"]),
    });
  });

  return router;
}

app.use(auth9Middleware({ domain: DOMAIN, audience: AUDIENCE, optional: false }));
app.use(createTestServer());

const optionalApp = express();
optionalApp.use(express.json());
optionalApp.use(auth9Middleware({ domain: DOMAIN, optional: true }));
optionalApp.use("/", createTestServer(true));

let server;
let optionalServer;

async function runTests() {
  return new Promise((resolve) => {
    server = app.listen(PORT, async () => {
      optionalServer = optionalApp.listen(PORT + 1, async () => {
        const results = [];

        // Helper to make requests
        async function request(path, options = {}) {
          const port = options.port || PORT;
          const url = `http://localhost:${port}${path}`;
          const headers = { ...options.headers };
          if (options.token) {
            headers["Authorization"] = `Bearer ${options.token}`;
          }

          try {
            const resp = await fetch(url, { headers, method: options.method || "GET" });
            const body = await resp.text();
            let json;
            try { json = JSON.parse(body); } catch { json = body; }
            return { status: resp.status, body: json, headers: resp.headers };
          } catch (e) {
            return { error: e.message };
          }
        }

        console.log("\n=== Express Middleware Integration Tests ===\n");

        // Scenario 1: Valid token authentication
        console.log("Scenario 1: Valid token authentication");
        if (!TEST_TOKEN) {
          console.log("SKIP: No TEST_TOKEN provided");
          results.push({ scenario: 1, status: "SKIP", reason: "No TEST_TOKEN" });
        } else {
          const r = await request("/test", { token: TEST_TOKEN });
          console.log(`  Status: ${r.status}`);
          console.log(`  Body:`, JSON.stringify(r.body, null, 2));
          const pass = r.status === 200 &&
            r.body.userId &&
            r.body.email &&
            r.body.tokenType === "tenantAccess" &&
            r.body.roles?.length > 0 &&
            r.body.permissions?.length > 0;
          results.push({ scenario: 1, status: pass ? "PASS" : "FAIL", response: r });
          console.log(`  Result: ${pass ? "✅ PASS" : "❌ FAIL"}\n`);
        }

        // Scenario 2: Authentication failures
        console.log("Scenario 2: Authentication failures");
        const tests2 = [
          { name: "No token", path: "/test", expectStatus: 401, expectMsg: "Missing authorization token" },
          { name: "Invalid token", path: "/test", token: "invalid", expectStatus: 401, expectMsg: "Invalid or expired token" },
          { name: "Basic auth", path: "/test", headers: { "Authorization": "Basic dXNlcjpwYXNz" }, expectStatus: 401 },
        ];

        let scenario2Pass = true;
        for (const t of tests2) {
          const r = await request(t.path, { headers: t.headers, token: t.token });
          const pass = r.status === t.expectStatus;
          console.log(`  ${t.name}: ${r.status} - ${pass ? "✅" : "❌"}`);
          if (!pass) scenario2Pass = false;
        }
        results.push({ scenario: 2, status: scenario2Pass ? "PASS" : "FAIL" });
        console.log(`  Result: ${scenario2Pass ? "✅ PASS" : "❌ FAIL"}\n`);

        // Scenario 3: Optional mode
        console.log("Scenario 3: Optional mode");
        if (!TEST_TOKEN) {
          console.log("SKIP: No TEST_TOKEN provided");
          results.push({ scenario: 3, status: "SKIP" });
        } else {
          const tests3 = [
            { name: "No token (anonymous)", path: "/public-or-private", port: PORT + 1, expectStatus: 200, expectMsg: "Anonymous" },
            { name: "Valid token", path: "/public-or-private", port: PORT + 1, token: TEST_TOKEN, expectStatus: 200, expectMsg: "Authenticated" },
            { name: "Invalid token (graceful)", path: "/public-or-private", port: PORT + 1, token: "bad", expectStatus: 200, expectMsg: "Anonymous" },
          ];

          let scenario3Pass = true;
          for (const t of tests3) {
            const r = await request(t.path, { port: t.port, token: t.token });
            const pass = r.status === t.expectStatus && r.body.message === t.expectMsg;
            console.log(`  ${t.name}: ${r.status} (${r.body.message}) - ${pass ? "✅" : "❌"}`);
            if (!pass) scenario3Pass = false;
          }
          results.push({ scenario: 3, status: scenario3Pass ? "PASS" : "FAIL" });
          console.log(`  Result: ${scenario3Pass ? "✅ PASS" : "❌ FAIL"}\n`);
        }

        // Scenario 4: requirePermission
        console.log("Scenario 4: requirePermission");
        if (!TEST_TOKEN) {
          console.log("SKIP: No TEST_TOKEN provided");
          results.push({ scenario: 4, status: "SKIP" });
        } else {
          const tests4 = [
            { name: "GET /users (has user:read)", path: "/users", token: TEST_TOKEN, expectStatus: 200 },
            { name: "POST /users (has user:read+write)", path: "/users", token: TEST_TOKEN, method: "POST", expectStatus: 200 },
            { name: "DELETE /users/1 (no user:delete)", path: "/users/1", token: TEST_TOKEN, method: "DELETE", expectStatus: 403 },
            { name: "PATCH /users/1 (any mode, has user:write)", path: "/users/1", token: TEST_TOKEN, method: "PATCH", expectStatus: 200 },
          ];

          let scenario4Pass = true;
          for (const t of tests4) {
            const r = await request(t.path, { token: t.token, method: t.method });
            const pass = r.status === t.expectStatus;
            console.log(`  ${t.name}: ${r.status} - ${pass ? "✅" : "❌"}`);
            if (!pass) scenario4Pass = false;
          }
          results.push({ scenario: 4, status: scenario4Pass ? "PASS" : "FAIL" });
          console.log(`  Result: ${scenario4Pass ? "✅ PASS" : "❌ FAIL"}\n`);
        }

        // Scenario 5: requireRole and AuthInfo helpers
        console.log("Scenario 5: requireRole and AuthInfo helpers");
        if (!TEST_TOKEN) {
          console.log("SKIP: No TEST_TOKEN provided");
          results.push({ scenario: 5, status: "SKIP" });
        } else {
          const tests5 = [
            { name: "GET /admin (has admin role)", path: "/admin", token: TEST_TOKEN, expectStatus: 200 },
            { name: "GET /superadmin (no superadmin role)", path: "/superadmin", token: TEST_TOKEN, expectStatus: 403 },
            { name: "GET /any-admin (any mode, has admin)", path: "/any-admin", token: TEST_TOKEN, expectStatus: 200 },
            { name: "GET /check-helpers", path: "/check-helpers", token: TEST_TOKEN, expectStatus: 200 },
          ];

          let scenario5Pass = true;
          for (const t of tests5) {
            const r = await request(t.path, { token: t.token });
            const pass = r.status === t.expectStatus;
            console.log(`  ${t.name}: ${r.status} - ${pass ? "✅" : "❌"}`);
            if (!pass) scenario5Pass = false;
          }

          // Check helpers specifically
          const helpers = await request("/check-helpers", { token: TEST_TOKEN });
          console.log("\n  AuthInfo helpers:");
          console.log(`    hasReadPerm: ${helpers.body.hasReadPerm} (expected: true)`);
          console.log(`    hasDeletePerm: ${helpers.body.hasDeletePerm} (expected: false)`);
          console.log(`    isAdmin: ${helpers.body.isAdmin} (expected: true)`);
          console.log(`    isSuperAdmin: ${helpers.body.isSuperAdmin} (expected: false)`);
          console.log(`    hasAnyWritePerm: ${helpers.body.hasAnyWritePerm} (expected: true)`);
          console.log(`    hasAllPerms: ${helpers.body.hasAllPerms} (expected: true)`);
          console.log(`    hasAllPermsIncDelete: ${helpers.body.hasAllPermsIncDelete} (expected: false)`);

          const helpersPass = helpers.body.hasReadPerm === true &&
            helpers.body.hasDeletePerm === false &&
            helpers.body.isAdmin === true &&
            helpers.body.isSuperAdmin === false &&
            helpers.body.hasAnyWritePerm === true &&
            helpers.body.hasAllPerms === true &&
            helpers.body.hasAllPermsIncDelete === false;

          console.log(`\n  Helpers result: ${helpersPass ? "✅ PASS" : "❌ FAIL"}`);
          if (!helpersPass) scenario5Pass = false;

          results.push({ scenario: 5, status: scenario5Pass ? "PASS" : "FAIL" });
          console.log(`  Result: ${scenario5Pass ? "✅ PASS" : "❌ FAIL"}\n`);
        }

        console.log("\n=== Summary ===");
        const passed = results.filter(r => r.status === "PASS").length;
        const failed = results.filter(r => r.status === "FAIL").length;
        const skipped = results.filter(r => r.status === "SKIP").length;
        console.log(`Total: ${results.length} scenarios`);
        console.log(`Passed: ${passed}`);
        console.log(`Failed: ${failed}`);
        console.log(`Skipped: ${skipped}`);

        server.close(() => {});
        optionalServer.close(() => {});
        resolve({ results, passed, failed, skipped });
      });
    });
  });
}

runTests().then((summary) => {
  process.exit(summary.failed > 0 ? 1 : 0);
}).catch((e) => {
  console.error("Test error:", e);
  process.exit(1);
});