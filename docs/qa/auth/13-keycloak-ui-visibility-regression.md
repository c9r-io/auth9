# 认证流程 - Keycloak UI 可视性异常路径回归

**模块**: 认证流程
**测试范围**: 社交登录、账户关联、异常路径下的托管认证页可视性
**场景数**: 5

---

## 目的与判定标准

本用例专门用于验证：即使认证链路底层经过 Keycloak，用户在可视范围内也不应看到未被 Auth9 接管的 Keycloak 原生 UI。

**允许的情况**
- 浏览器地址短暂经过 Keycloak broker URL（如 `/realms/{realm}/broker/{provider}/link`、`/broker/{provider}/endpoint`）
- 在异常场景下出现由 `auth9-keycloak-theme` 承载的托管认证页或信息页

**不允许的情况**
- 出现 Keycloak 默认 logo、默认标题、默认布局
- 页面文案把产品直接暴露为 Keycloak，而非 Auth9 品牌认证页
- 需要用户手动操作 Keycloak Admin Console 或直接输入 Keycloak URL 才能完成主流程验证

**回归凭据管理**
- 社交登录 OAuth 凭据和测试账号凭据应从团队密钥系统（推荐 Vault）读取
- 当前标准 secret：
  - `secret/auth9-github-oauth`
  - `secret/auth9-github-test-account`

> 本文档关注“可视 UI 是否泄漏”。功能成功/失败本身由 [auth/04-social.md](./04-social.md) 主文档覆盖。

---

## 场景 1：从用户可见入口发起关联，不出现额外 Keycloak 交互页

### 初始状态
- 已登录 Auth9 Portal
- 已在「设置」→「身份提供商」中启用 GitHub provider
- 当前用户尚未关联 GitHub

### 目的
验证用户从可见入口发起账户关联时，不会看到额外的 Keycloak 确认 UI 或原生中间页

### 测试操作流程
1. 进入「Account」→「Linked Identities」
2. 确认页面显示「Link another identity」区域
3. 点击「Link GitHub」
4. 观察浏览器跳转过程，直到进入 GitHub 授权页

### 预期结果
- 主流程从 Auth9 Portal 的可见入口开始，而非直接输入 URL
- 点击后可直接到达 GitHub 授权页
- 过程中不应出现可见的 Keycloak 默认确认页或默认登录页
- 即使地址栏短暂经过 Keycloak broker URL，也不判定为 UI 泄漏

---

## 场景 2：第三方用户主动取消授权，返回后不出现原生 Keycloak 错误页

### 初始状态
- 满足场景 1 的前置条件
- 有可用于测试的 GitHub 账号

### 目的
验证用户在第三方 IdP 取消授权时，不会看到未被主题接管的 Keycloak 原生错误页

### 测试操作流程
1. 按场景 1 进入 GitHub 授权页
2. 在 GitHub 点击取消/拒绝授权（如页面提供「Cancel」或等价操作）
3. 观察回跳后的页面

### 预期结果
- 如果回到 Portal，应显示可理解的 Auth9 结果页或停留在原入口页
- 如果回到托管认证页/信息页，该页面必须保持 Auth9 品牌样式
- 不应出现 Keycloak 默认 logo、默认配色或 “Keycloak” 作为主产品标识

---

## 场景 3：Identity Provider 配置错误时，异常页仍受主题控制

### 初始状态
- 已存在可用 GitHub provider
- 测试人员可临时将 provider 的 `client_secret` 修改为错误值，并在测试结束后恢复

### 目的
验证第三方回调失败时，异常页仍由 Auth9 主题承载，而不是泄漏 Keycloak 默认 UI

### 测试操作流程
1. 在「设置」→「身份提供商」中将 GitHub provider 的 `client_secret` 临时改为错误值
2. 进入「Account」→「Linked Identities」，点击「Link GitHub」
3. 在 GitHub 完成登录并授权
4. 观察回跳后的错误页面
5. 将 GitHub provider 的 `client_secret` 恢复为正确值

### 预期结果
- 流程会失败，但失败页若由 Keycloak 渲染，必须仍使用 Auth9 主题外观
- 页面不得显示 Keycloak 默认品牌元素
- 恢复正确配置后，后续功能回归不受影响

---

## 场景 4：首次社交登录触发补充信息/信息页时，页面仍是 Auth9 品牌认证页

### 初始状态
- 已启用至少一个社交 Identity Provider
- 使用一个未在 Auth9 中建立映射的新第三方账号

### 目的
验证首次社交登录若触发补充资料、信息提示或中间说明页，仍由 `auth9-keycloak-theme` 承载

### 测试操作流程
1. 打开 Portal `/login`
2. 点击「Sign in with password」进入 Auth9 品牌认证页
3. 在认证页选择社交登录按钮（如 GitHub 或 Google）
4. 使用一个新的第三方账号完成授权
5. 如果流程中出现信息页、确认页、补充资料页，观察页面样式和品牌

### 预期结果
- 允许出现托管认证链路的中间页
- 这些页面必须保持 Auth9 品牌认证页的样式和主题壳
- 不应出现裸 Keycloak 默认页面

---

## 场景 5：无可用提供商时，不应出现误导性的 Keycloak UI 回退路径

### 初始状态
- 当前环境未启用任何社交 Identity Provider，或目标 provider 已禁用

### 目的
验证在无可用社交登录配置时，用户不会因 UI 缺口被迫进入 Keycloak 页面

### 测试操作流程
1. 打开 Portal `/login`
2. 点击「Sign in with password」进入 Auth9 品牌认证页
3. 检查认证页是否显示社交登录按钮
4. 打开「Account」→「Linked Identities」
5. 检查是否显示可发起关联的 provider 按钮

### 预期结果
- 认证页不显示不可用 provider 的社交按钮
- 「Linked Identities」不显示不可执行的「Link {provider}」按钮
- 用户不会被引导去直接访问任何 Keycloak 页面来完成缺失配置

---

## 回归测试检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | 从用户可见入口发起关联，不出现额外 Keycloak 交互页 | ☐ | | | |
| 2 | 第三方用户主动取消授权，返回后不出现原生 Keycloak 错误页 | ☐ | | | |
| 3 | Identity Provider 配置错误时，异常页仍受主题控制 | ☐ | | | 需测试后恢复正确配置 |
| 4 | 首次社交登录触发补充信息/信息页时，页面仍是 Auth9 品牌认证页 | ☐ | | | |
| 5 | 无可用提供商时，不应出现误导性的 Keycloak UI 回退路径 | ☐ | | | |
