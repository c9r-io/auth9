# 技术负债 #002: QA 文档 Keycloak UI 泄漏整改清单

**创建日期**: 2026-03-04
**状态**: 🟡 Validation Pending
**优先级**: High
**影响范围**: `docs/qa`、认证流程、身份提供商、品牌设置、会话/安全测试
**预计修复时间**: 1-2 周

---

## 当前进度

- 已完成第一轮与第二轮 `docs/qa` 文档整改，已修正 20+ 份文档中的主测试路径和术语泄漏。
- 已将高误导性文档（认证、品牌、身份提供商、Passkeys、安全告警、初始化）从“直接操作 Keycloak UI/控制台”改为以 Auth9 代理页、Auth9 API 和“Auth9 品牌认证页”为主。
- 已完成 `/login` 首入口整改：登录页现在直接提供 `Forgot password?`，并在品牌配置允许注册时显示 `Create account`。
- 已完成已登录态社交身份主动关联入口：
  - Account 页新增「Link another identity」入口
  - 通过受控授权链路发起关联，并在回调后返回 Linked Identities 页面
  - 登录/回调阶段会同步底层 federated identities 到 `linked_identities`
- 当前剩余工作以真实环境验收为主，而非代码层功能缺口：
  - 需在配置了真实社交 IdP 凭据的环境中验证 `kc_action=idp_link:*` 链路
  - 需确认目标 Keycloak client 在部署配置中允许账户关联所需的链接动作

---

## 问题描述

项目设计要求是：

- Keycloak 仅作为 headless OIDC engine 存在
- 必要认证 UI 仅由 `auth9-keycloak-theme` 承载
- 其他任何 UI/管理界面都应由 Auth9 Portal 或 Auth9 API 代理

当前多份 QA 文档仍把测试路径、验收断言或环境准备直接绑定到 Keycloak UI、Keycloak URL、Keycloak Admin Console。结果是：

- QA 文档暴露了不该暴露的实现细节
- 测试人员会被引导去操作 Keycloak，而不是优先验证 Auth9 对外产品面
- 少数文档还暴露出 Auth9 入口整合或功能闭环缺失

本清单用于逐文档整改，明确每个文件该删什么、改什么、保留什么，以及问题归属。

---

## 归因分类

### A. 文档错误

文档把测试者错误地引导到 Keycloak UI 或 Admin Console，但仓库里已经存在 Auth9 对应入口或更合适的测试方式。

### B. 文案泄漏实现细节

场景本身成立，但文档把 “Auth9 品牌认证页” 写成了 “Keycloak 页面”，或把 `/realms/auth9` 等内部路由当成主断言。

### C. 产品/UI 缺口

文档之所以依赖 Keycloak，是因为 Auth9 还没有把必要入口整合到自己的 UI，或 Auth9 缺少对应能力。

### D. Theme 合理承载

该场景属于认证链路中的必要托管页，允许继续由 `auth9-keycloak-theme` 承载，但 QA 文档应避免直接把 “Keycloak” 作为主验收对象。

---

## 逐文档整改清单

### [docs/qa/auth/01-oidc-login.md](/Volumes/Yotta/auth9/docs/qa/auth/01-oidc-login.md)

**分类**: B + D  
**优先级**: Medium

**删除**

- 删除把 “Keycloak 默认登录页” 作为用户可见产品名的表述。
- 删除把 “Keycloak 验证成功” 作为主步骤文案。

**修改**

- 将 “Keycloak 品牌化登录页 / Keycloak 页面” 统一改为 “Auth9 品牌认证页（由 auth9-keycloak-theme 承载）”。
- 将所有 URL 断言从 “进入 Keycloak 页面” 改为 “进入 Auth9 品牌认证页并继续认证”。
- MFA 场景中保留失败事件来自底层认证引擎的说明，但不要把 Keycloak 作为测试主对象。

**保留**

- 保留 `Sign in with password` 会跳转到托管认证页的事实。
- 保留 MFA 失败依赖事件桥接的说明。
- 保留 tenant 选择、token exchange、Dashboard 这些 Auth9 侧主流程断言。

**说明**

- 这是必要认证 UI，由 theme 承载，问题不在功能缺失，而在文案暴露实现细节。

---

### [docs/qa/auth/03-password.md](/Volumes/Yotta/auth9/docs/qa/auth/03-password.md)

**分类**: C（主） + B  
**优先级**: Critical

**删除**

- 删除 “忘记密码 / 重置密码页面均由 Keycloak 托管” 的默认结论。
- 删除 “QA 需要先点击 `Sign in with password` 进入 Keycloak 页面才能看到忘记密码” 的强绑定路径。
- 删除把 “在 Keycloak 重置页面输入新密码” 作为标准步骤。

**修改**

- 改写为双层说明：
  - 当前实现：认证链路仍可落到 theme 托管页
  - 目标设计：用户应优先通过 Auth9 代理页完成密码找回/重置
- 将主测试路径改为：
  - `/forgot-password`
  - 邮件中的 `/reset-password?token=...`
  - 已登录后的 `/dashboard/account/security`
- 将强度验证主路径改为：
  - `/reset-password`
  - `/dashboard/account/security`
- 将 Keycloak realm 密码策略同步校验降级为实现级排障附录，而不是主测试步骤。

**保留**

- 保留密码策略由 Auth9 同步到底层认证引擎的事实。
- 保留 Mailpit 邮件验证、token 过期、已登录修改密码等业务断言。

**说明**

- 仓库已经存在 Auth9 自有页面和 API：
  - `/forgot-password`
  - `/reset-password`
  - `/api/v1/auth/forgot-password`
  - `/api/v1/auth/reset-password`
  - `/api/v1/users/me/password`
- 登录页首入口现已补齐忘记密码/注册链接展示逻辑：
  - 始终显示 `Forgot password?`
  - 当品牌配置允许注册时显示 `Create account`
- 该文档的产品缺口已关闭，后续仅需持续保持 QA 步骤与登录页行为一致。

---

### [docs/qa/auth/04-social.md](/Volumes/Yotta/auth9/docs/qa/auth/04-social.md)

**分类**: A + C + B + D  
**优先级**: High

**删除**

- 删除 “访问 Keycloak Admin Console 配置 IdP” 作为标准前置步骤。
- 删除把 “Keycloak 登录页底部社交按钮” 作为唯一用户视角的表述。

**修改**

- 将 IdP 配置前置条件改为：
  - 优先通过 Auth9 Portal 「设置 -> 身份提供商」
  - 或使用 Auth9 API / seed 脚本
- 将 “Keycloak 品牌化登录页” 统一改为 “Auth9 品牌认证页（theme 承载）”。
- 将 “Google 回调到 Keycloak” 改为 “第三方 IdP 完成后，返回 Auth9 托管认证链路并继续登录”。
- 场景 2 现已可改回可执行场景：
  - Account 页提供已登录态主动发起关联入口
  - 完成授权后回到 Linked Identities 页面
  - 前提是环境中已配置可用的社交 IdP 凭据

**保留**

- 保留社交登录按钮由托管认证页承载的事实。
- 保留登录成功后 `linked_identities` 和 `login_events` 的数据断言。
- 保留解除关联场景。

**说明**

- “去 Keycloak Admin Console 配 IdP” 是文档错误。
- “已登录后主动关联 GitHub” 的产品缺口已在代码层补齐，文档应回到可执行路径描述。

---

### [docs/qa/auth/05-boundary.md](/Volumes/Yotta/auth9/docs/qa/auth/05-boundary.md)

**分类**: A  
**优先级**: Medium

**删除**

- 删除 “在 Keycloak 管理界面创建测试用户” 一节。

**修改**

- 将测试数据准备统一改为：
  - 使用 Auth9 API 创建用户
  - 或使用测试 seed / fixture / 脚本
- 若确需 Keycloak 对应账户，改为说明 “通过 Auth9 正常注册或受控测试脚本生成”，不要要求 QA 操作 Keycloak UI。

**保留**

- 保留并发登录、refresh token、CORS 等场景本身。

---

### [docs/qa/auth/08-demo-auth-flow.md](/Volumes/Yotta/auth9/docs/qa/auth/08-demo-auth-flow.md)

**分类**: B + D  
**优先级**: Low

**删除**

- 删除把 “跳转到 Keycloak 登录页” 作为用户产品语义的表述。

**修改**

- 改为 “跳转到 Auth9 托管认证页（底层由 OIDC 引擎处理）”。
- 将 URL 断言集中在：
  - authorize 请求正确发起
  - redirect_uri 正确
  - 回到 demo 应用成功

**保留**

- 保留 demo 作为第三方 RP 的测试定位。
- 保留对 redirect_uri 注册错误的回归断言。

---

### [docs/qa/auth/10-b2b-onboarding-flow.md](/Volumes/Yotta/auth9/docs/qa/auth/10-b2b-onboarding-flow.md)

**分类**: B + D  
**优先级**: Low

**删除**

- 删除 “在 Keycloak 页面输入用户名密码” 这种实现导向措辞。

**修改**

- 改成 “通过 `Sign in with password` 进入 Auth9 托管认证页完成认证”。
- 认证方式部分统一强调：重点验收的是 `/tenant/select`、`/onboard`、`/dashboard` 的 Auth9 路由分发，而不是底层认证页品牌或域名。

**保留**

- 保留三种登录方式都能进入同一 B2B 路由分发链路的设计说明。

---

### [docs/qa/auth/12-enterprise-sso-ui-regression.md](/Volumes/Yotta/auth9/docs/qa/auth/12-enterprise-sso-ui-regression.md)

**分类**: B  
**优先级**: Low

**删除**

- 删除把 `/realms/auth9` 作为主通过标准的措辞。

**修改**

- 将主断言改为：
  - `/login` 页面可见
  - 企业邮箱输入可交互
  - 触发 SSO 发现后离开 `/login`
  - 请求包含 `kc_idp_hint`
- `/realms/auth9` 仅作为实现级辅助断言保留在备注中。

**保留**

- 保留对 loader auto-redirect 回归的重点覆盖。

---

### [docs/qa/identity-provider/02-toggle-validation.md](/Volumes/Yotta/auth9/docs/qa/identity-provider/02-toggle-validation.md)

**分类**: B + D  
**优先级**: Medium

**删除**

- 删除 “如需排障，可直接访问 Keycloak 登录页面 URL 对比验证” 作为正常流程的一部分。

**修改**

- 将登录页验证改写为：
  - 通过 Auth9 登录入口触发认证
  - 验证 Auth9 托管认证页上的社交入口是否显示/隐藏
- Keycloak Admin API 校验可保留为实现级校验，但应标注为 “后台同步校验”，不是用户视角。

**保留**

- 保留启停切换后前台社交入口可见性变化。
- 保留后台实例状态校验。

---

### [docs/qa/integration/06-init-seed-data.md](/Volumes/Yotta/auth9/docs/qa/integration/06-init-seed-data.md)

**分类**: A  
**优先级**: Medium

**删除**

- 删除 “通过 Keycloak Admin Console 查看管理员邮箱”。

**修改**

- 改为：
  - 优先使用 DB 验证
  - 必要时使用容器内管理 API（非 UI）
- 如果保留与底层 IdP 的一致性校验，明确标注为 “后端集成校验”，不是 QA 操作控制台。

**保留**

- 保留 `auth9-core init` 幂等性、seed 结果和日志断言。

---

### [docs/qa/passkeys/01-passkeys.md](/Volumes/Yotta/auth9/docs/qa/passkeys/01-passkeys.md)

**分类**: A  
**优先级**: High

**删除**

- 删除自动化步骤中的旧按钮名 “Sign in with SSO”。
- 删除 “页面跳转到 Keycloak，查看登录表单” 作为登录准备步骤。

**修改**

- 将自动化登录准备统一改为：
  - 如果需要密码登录，点击 `Sign in with password`
  - 如果需要企业 SSO，点击 `Continue with Enterprise SSO`
- 该文档主旨是原生 WebAuthn 注册与管理，应尽量使用已有登录态 fixture，减少无关认证链路依赖。

**保留**

- 保留 “Passkey 注册不应跳转 Keycloak” 的核心断言。
- 保留 CDP 虚拟认证器方案。

**说明**

- 这是自动化文档过时，不是产品缺口。

---

### [docs/qa/passkeys/02-passkey-auth.md](/Volumes/Yotta/auth9/docs/qa/passkeys/02-passkey-auth.md)

**分类**: B  
**优先级**: Low

**删除**

- 删除 “只能进入 Keycloak 默认用户名/密码表单” 这类面向终端用户的表述。

**修改**

- 改为 “被错误地直接重定向到托管密码认证链路”。
- 保留 `auto-redirect` 回归背景，但弱化 Keycloak 名词出现频率。

**保留**

- 保留 `/login` 必须稳定渲染三种认证方式的核心回归目标。

---

### [docs/qa/service/06-service-branding.md](/Volumes/Yotta/auth9/docs/qa/service/06-service-branding.md)

**分类**: B + D  
**优先级**: Medium

**删除**

- 删除 “观察 Keycloak 登录页外观” 这种措辞。

**修改**

- 改为 “观察由 `auth9-keycloak-theme` 承载的 Auth9 品牌认证页外观”。
- 将 Network 中 `GET /api/v1/public/branding?client_id=...` 保留为主技术断言。

**保留**

- 保留按 `client_id` 加载 Service 品牌的验证目标。

---

### [docs/qa/session/02-login-events.md](/Volumes/Yotta/auth9/docs/qa/session/02-login-events.md)

**分类**: B + D  
**优先级**: Low

**删除**

- 删除把 “登录操作发生在 Keycloak” 写成用户测试主语的表述。

**修改**

- 改为 “用户名/密码与 MFA 验证由底层认证引擎处理；QA 应从 Auth9 登录入口触发”。
- 故障排查中若提到 Keycloak realm Event Listeners，明确标注为平台运维级排障信息。

**保留**

- 保留事件桥接和 `login_events` 校验。

---

### [docs/qa/session/03-alerts.md](/Volumes/Yotta/auth9/docs/qa/session/03-alerts.md)

**分类**: A  
**优先级**: High

**删除**

- 删除前置步骤 “在 Keycloak 中创建目标用户”。

**修改**

- 将测试准备改为：
  - 通过 Auth9 用户 API 或测试 seed 创建用户
  - 若测试需要对齐底层 `keycloak_id`，使用受控脚本完成，不要求 QA 手工操作底层管理接口
- 如必须注入底层用户映射，单独放到 “测试夹具脚本” 章节，不放在手工操作流程里。

**保留**

- 保留安全告警 webhook 模拟。
- 保留 seed 数据用于触发 `new_device` / `impossible_travel` 的逻辑。

---

### [docs/qa/session/07-oauth-state-csrf.md](/Volumes/Yotta/auth9/docs/qa/session/07-oauth-state-csrf.md)

**分类**: B + D  
**优先级**: Low

**删除**

- 删除 “在 Keycloak 页面停留超过 5 分钟” 这类文案。

**修改**

- 改为 “在外部托管认证页停留超过 5 分钟”。
- 场景重点放在：
  - state cookie
  - callback 校验
  - 超时/重放行为

**保留**

- 保留 CSRF 核心安全断言。

---

### [docs/qa/settings/01-branding.md](/Volumes/Yotta/auth9/docs/qa/settings/01-branding.md)

**分类**: B + D  
**优先级**: High

**删除**

- 删除 “Keycloak 验证” 作为主章节标题。
- 删除直接访问 `http://localhost:8081/realms/...` 作为主验证步骤。

**修改**

- 将场景 4 改写为两层验证：
  - 用户视角：通过 Auth9 登录入口验证是否出现 “Create account”
  - 后端同步视角：可选地通过容器内 API 确认 `registrationAllowed`
- 将所有 “Keycloak 登录页” 改为 “Auth9 品牌认证页（theme 承载）”。
- 明确说明：注册可见性是 Auth9 产品策略开关，底层同步仅是实现细节。

**保留**

- 保留品牌设置页本身所有 Portal UI 验证。
- 保留容器内 Admin API 校验作为排障附录。

---

### [docs/qa/user/02-advanced.md](/Volumes/Yotta/auth9/docs/qa/user/02-advanced.md)

**分类**: B  
**优先级**: Low

**删除**

- 删除 “目标用户必须在 Keycloak 中存在对应账户” 这种直接面向 QA 的表述。

**修改**

- 改为 “目标用户必须具备有效底层认证主体映射（`keycloak_id`）”。
- 将 “Keycloak 中 MFA 配置同步” 改为 “底层认证引擎 MFA 状态同步”。

**保留**

- 保留删除用户级联、MFA 启停、列表、租户关联等场景。

---

### [docs/qa/README.md](/Volumes/Yotta/auth9/docs/qa/README.md)

**分类**: A  
**优先级**: High

**删除**

- 删除 “Keycloak 管理” 作为常规 QA 基础信息块。
- 删除默认暴露 `http://localhost:8081/admin` 和默认凭证。

**修改**

- 改为：
  - 如无特殊说明，QA 不应直接操作 Keycloak UI
  - 底层认证引擎仅用于运维/排障/受控脚本
- 新增统一规范：
  - 用户可见认证页统一称为 “Auth9 品牌认证页”
  - Keycloak 相关 URL、控制台、realm 配置仅能出现在实现级附录

**保留**

- 保留数据库连接和常规测试结构说明。

---

## 不建议在本轮主修中改动的文件

以下文件可在文案层面轻微收敛 Keycloak 名词，但不属于当前高优先级阻塞：

- [docs/qa/integration/02-password-policy.md](/Volumes/Yotta/auth9/docs/qa/integration/02-password-policy.md)
- [docs/qa/integration/10-security-hardening-p2.md](/Volumes/Yotta/auth9/docs/qa/integration/10-security-hardening-p2.md)
- [docs/qa/session/04-boundary.md](/Volumes/Yotta/auth9/docs/qa/session/04-boundary.md)

理由：

- 这几份文档更多是在做后端集成或底层同步校验
- 出现 Keycloak 名词不一定等于 “UI 泄漏”
- 可以放到第二轮做术语统一和附录重构

---

## 推荐执行顺序

1. 先修文档基线：
   - `docs/qa/README.md`
   - `docs/qa/settings/01-branding.md`
   - `docs/qa/auth/01-oidc-login.md`
2. 再修高误导性手册：
   - `docs/qa/auth/03-password.md`
   - `docs/qa/auth/04-social.md`
   - `docs/qa/session/03-alerts.md`
   - `docs/qa/passkeys/01-passkeys.md`
3. 再做术语统一：
   - 其余认证/会话文档里的 “Keycloak 页面” 统一替换为 “Auth9 品牌认证页”
4. 最后处理产品缺口对应文档：
   - 密码找回入口整合到 `/login` 后，持续校验 `auth/03-password.md` 与登录页入口一致
   - 已登录态社交账号主动关联 UI 已完成，持续校验 `auth/04-social.md` 与真实 IdP 环境行为一致

---

## 对应产品整改项

### 必须补齐

- 已完成：在 Auth9 `/login` 页面增加用户可见的找回密码入口。
- 已完成：在 Auth9 `/login` 按品牌配置显示注册入口，避免 QA 只能从托管认证页验证注册可见性。
- 已完成：在 Auth9 Account 区域增加主动发起社交身份关联的入口。
- 已完成：通过受控认证链路发起账户关联，并在登录/回调时同步 `linked_identities`。

### 可延后

- 将更多认证辅助流程从 theme 承载页前移到 Auth9 Portal 自有页面。
- 为测试准备提供统一脚本，替代文档里手工调用底层管理接口。

---

## 验收标准

- [x] `docs/qa/README.md` 不再把 Keycloak Admin Console 作为常规 QA 工具暴露
- [x] 所有高优先级 QA 文档不再把 “去 Keycloak 页面/控制台” 作为主测试步骤
- [x] “Auth9 品牌认证页” 成为认证托管页的统一文案
- [x] `docs/qa/auth/03-password.md` 改为以 Auth9 代理页为主路径，或明确标记当前产品阻塞
- [x] `docs/qa/auth/04-social.md` 中 IdP 配置前置条件改为 Auth9 管理入口
- [x] `docs/qa/auth/04-social.md` 中 “主动关联社交账户” 已恢复为可执行路径，并以 Auth9 Portal 为入口
- [x] `docs/qa/passkeys/01-passkeys.md` 自动化步骤不再引用已不存在的旧按钮名
- [x] Auth9 `/login` 页面补齐找回密码与注册的用户可见入口
- [x] Auth9 Account 页面补齐“主动新增社交身份关联”入口

---

## 历史记录

| 日期 | 状态 | 变更 | 负责人 |
|------|------|------|--------|
| 2026-03-04 | 🔴 Active | 基于现状审查新增逐文档整改清单 | Codex |
| 2026-03-04 | 🔴 Active | 完成 docs/qa 第一轮与第二轮整改，文档侧主问题已收口，剩余为产品入口缺口 | Codex |
| 2026-03-04 | 🔴 Active | 补齐 `/login` 的找回密码/注册链接并通过前端测试，剩余阻塞收敛为“主动新增社交身份关联”缺少后端接口 | Codex |
| 2026-03-04 | 🟡 Validation Pending | 补齐 Account 主动关联入口、回调跳回与 `linked_identities` 同步；剩余为真实 IdP 环境验收 | Codex |
