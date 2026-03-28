---
name: ticket-fix
description: Read the ticket document(s) under docs/ticket/, and verify the issue against the actual implementation. Classify as bug, false positive, or feature gap. For bugs, fix the code. For false positives, update QA docs. For feature gaps, fix in-place if small or create a Feature Request if large. Regardless of category, reset the environment using ./scripts/reset-docker.sh, then rerun QA testing. After testing confirms resolution, delete the ticket document(s). Use when a user asks to fix a ticket or resolve a QA failure.
---

# Ticket Fix

Resolve QA tickets end-to-end: read the ticket, classify as bug / false positive / feature gap, take appropriate action, reset environment, re-run QA steps, and clean up the ticket on success.

## Workflow

1. **Discover tickets**
   - List `docs/ticket/*.md` and confirm which ticket(s) to handle if not specified.

2. **Read ticket and validate**
   - Parse: scenario, steps, expected/actual, environment, SQL checks.
   - Reproduce quickly against current implementation.
   - If issue no longer reproducible, document why and still run QA verification.

3. **Classify ticket (分类判定)**
   - After reproduction (or failed reproduction), classify the ticket into exactly one category:
     - **代码缺陷 (Bug)**: Code behaves incorrectly relative to its own design intent. The feature exists but produces wrong results.
     - **误报 (False Positive)**: QA procedure is flawed — wrong commands, missing prerequisites, incorrect assumptions. Code is correct AND feature is complete.
     - **功能缺口 (Feature Gap)**: Code is correct for what it implements, but the feature the QA test expects is **missing, incomplete, or not enabled**. The test expectation is reasonable but the implementation hasn’t caught up.
   - Decision heuristics:
     - If the code has a clear logic error → **Bug**
     - If changing the test procedure alone would make it pass → **False Positive**
     - If passing the test requires new code, new config, or enabling a disabled feature → **Feature Gap**

4. **Path A: Bug fix (代码缺陷修复)**
   - If confirmed as bug, implement code fix.
   - Keep change scope minimal and aligned to ticket.

5. **Path B: False positive analysis (误报分析)**
   - Determine whether the ticket is a **false positive** caused by flawed test procedures rather than a code bug.
   - Common false-positive patterns:
     - Test commands missing required authentication/signatures (e.g., HMAC headers).
     - Prerequisites incomplete or ambiguous (e.g., "webhook must exist" without creation steps).
     - Environment assumptions wrong (e.g., assuming signature verification is disabled when Docker default enables it).
     - Test data referencing non-existent entities without fallback handling.
     - **Wrong Token type**: Used Identity Token where Tenant Access Token is required (403 with "Identity token is only allowed for tenant selection and exchange"). Check if scenario has a「步骤 0: 验证 Token 类型」gate.
     - **Non-UUID test data**: Manual INSERT SQL used non-UUID strings for `id` fields, causing `ColumnDecode` errors on subsequent API calls. Check if scenario has a「步骤 0: 验证测试数据完整性」gate.
     - **Missing environment config**: Test assumed a service/config was present that requires explicit setup (IdP, MFA brute-force protection, observability stack). Check if scenario has a「步骤 0: 验证环境状态」gate.
   - If confirmed as false positive:
     1. Identify the **root cause in the QA document** (not the code).
     2. Update the referenced QA doc (`docs/qa/`) to prevent recurrence:
        - Make implicit requirements explicit and prominent (bold, moved to prerequisites).
        - Ensure example commands are **copy-paste-ready** for the default Docker environment.
        - Add a **troubleshooting table** for common failure modes with symptoms, causes, and fixes.
     3. Still proceed with environment reset and re-test to confirm the flow works correctly.

6. **Path C: Feature gap analysis (功能缺口分析)**
   - After confirming the ticket describes a missing or incomplete feature (not a code error), perform structured analysis.
   - Common feature gap patterns:
     - **Feature disabled by default**: Feature exists in code but is toggled off (e.g., `allow_registration=false`). QA expects it enabled.
     - **API incomplete**: Endpoint exists and returns success, but critical side effects are missing (e.g., user created but credentials not stored).
     - **Missing test tooling**: No test client, mock, or harness exists to exercise the feature path QA needs (e.g., no PKCE-capable test client).
     - **Partial implementation**: Config/mode accepted by API but logic not wired through (e.g., `breach_check_mode=warn` accepted but all modes still reject).
     - **Dependency not yet built**: Feature requires infrastructure that doesn’t exist yet (e.g., SMS OTP requires SMS Provider).
   - **Decision framework** — choose one action:
     1. **Fix in-place (就地修复)**: When the gap is small and self-contained — typically enabling a config, wiring an existing code path, or adding a missing DB write. Scope must fit within a single ticket-fix session.
        - Example: setting `allow_registration=true` in seed data, or adding credential INSERT to user creation flow.
     2. **Create Feature Request (创建 FR)**: When the gap requires substantial new code, new infrastructure, or cross-cutting design decisions. Create a doc in `docs/feature_request/` following existing FR format.
        - Example: implementing `breach_check_mode=warn` response format, building a PKCE test client.
     3. **Update QA doc to defer (标记延后)**: When the QA scenario is valid but premature — the feature is planned but not yet prioritized. Update the QA doc scenario to mark it as `[DEFERRED - pending FR: {name}]` and remove it from active test rotation.
   - For **fix in-place**: proceed to step 7 (reset) and step 8 (re-test) as normal.
   - For **create FR**:
     1. Write the FR doc in `docs/feature_request/` following the established format (背景, 期望行为, 涉及文件, 验证方法).
     2. Update the QA doc scenario with a note linking to the FR.
     3. Proceed to close ticket (step 9) with feature-gap outcome.
   - For **update QA doc to defer**:
     1. Add `[DEFERRED]` marker to the specific scenario in the QA doc.
     2. Proceed to close ticket (step 9) with feature-gap outcome.

7. **Reset environment (always)**
   - Prefer running `./scripts/reset-docker.sh` (generated by `project-bootstrap`).
   - If missing, fall back to `docker compose -f docker/docker-compose.yml down -v` then `docker compose ... up -d`.

8. **Re-run QA steps**
   - Follow the ticket’s steps and SQL validation.
   - Capture evidence (log snippets, DB results).

9. **Close ticket**
   - If verified (by code fix, false-positive doc update, or feature-gap resolution), delete the ticket file from `docs/ticket/`.
   - Summarize outcome in response:
     - For code fixes (代码缺陷): describe the fix + verification evidence.
     - For false positives (误报): list the misreport causes + QA doc changes made.
     - For feature gaps (功能缺口): state the gap identified + action taken:
       - If fixed in-place: describe the fix + verification evidence.
       - If FR created: link to `docs/feature_request/{name}.md` + QA doc annotation.
       - If deferred: note which QA scenario was marked `[DEFERRED]` and why.

## Rules

- Always reset environment before final verification.
- Do not delete ticket unless QA re-test passes.
- If issue cannot be reproduced, explain why and still re-test.
- If re-test fails, keep the ticket and report remaining issue.
- **False positive handling**: When a ticket is a misreport, fixing the QA doc is as important as fixing code — prevent the same false positive from being filed again.
- **Feature gap handling**: When a ticket reveals a missing feature, assess scope honestly. Small gaps (config toggle, missing DB write) can be fixed in-place. Larger gaps (new API logic, new infrastructure, design decisions) MUST become a Feature Request — do not attempt oversized fixes within ticket-fix scope.
- **FR format**: Feature Request docs must follow the established format in `docs/feature_request/` — include 背景, 期望行为 with requirement IDs (R1, R2...), 涉及文件, and 验证方法.

## Notes

- Use `docs/qa/` for any referenced test cases.
- Use Docker logs and DB queries from the ticket for evidence.
- When updating QA docs for false positives, focus on the **specific scenario** that caused the misreport — do not over-generalize or rewrite unrelated sections.
- Use `docs/feature_request/` for any new Feature Request documents created from feature-gap tickets. Reference existing FRs (e.g., `infra_sms_provider.md`) for format and level of detail.