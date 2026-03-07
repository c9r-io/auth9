# Ticket: Keycloak 主题未实现 i18n 国际化

**Created**: 2026-03-07 16:27:02
**Module**: Keycloak Theme / i18n
**Status**: FAILED

---

## 测试内容

自定义 Keycloak 主题（auth9 keycloak theme）中未实现国际化（i18n）支持，所有界面文本仅有单一语言版本，无法跟随用户选择的语言自动切换。

**Test Location**: Keycloak 登录页面、注册页面、MFA 页面等（Keycloak 托管的所有页面）

---

## 预期结果

- Keycloak 主题应支持多语言（至少支持中文 `zh-CN` 和英文 `en`）
- 当用户在 auth9-portal 中切换语言时，跳转至 Keycloak 页面（如登录、2FA 等）后，界面应使用对应语言
- Keycloak 提供的 i18n 机制（`messages/messages_zh.properties`）应被正确实现

---

## 再现方法

### Prerequisites
- Docker 服务运行正常，Keycloak 使用自定义 auth9 主题

### Steps to Reproduce
1. 在 auth9-portal 语言切换中选择中文（或其他非默认语言）
2. 点击登录，跳转至 Keycloak 登录页面
3. 观察 Keycloak 页面的文本语言
4. 期望显示中文文本，实际显示英文（或默认语言文本）

### Environment
- Keycloak: http://localhost:8081
- 使用 auth9 自定义主题

---

## 实际结果

- Keycloak 自定义主题页面始终以单一语言显示（英文）
- 未实现 Keycloak i18n properties 文件（如 `messages_zh.properties`）
- 语言切换在 Keycloak 托管页面中无效

---

## Analysis

**Root Cause**: 
Keycloak 主题目录下缺少 i18n 语言文件，需在主题的 `messages/` 目录下为每种支持的语言创建对应的 `.properties` 文件，并在 `theme.properties` 中声明支持的语言列表。

**Keycloak i18n 实现方式**:
```
keycloak-theme/
└── login/
    ├── messages/
    │   ├── messages_en.properties   # English (default)
    │   └── messages_zh.properties  # 中文 (需创建)
    └── theme.properties             # 需添加: locales=en,zh
```

**Severity**: Medium（影响 Keycloak 托管页面的用户体验一致性，尤其面向中文用户）

**Related Components**: `keycloak-theme` / Keycloak Login Theme / i18n / FreeMarker Templates

**Reference**: 
- Keycloak 文档: [Keycloak Themes - Internationalization](https://www.keycloak.org/docs/latest/server_development/#_themes)
- 项目文档: `docs/keycloak-theme.md`

## 跟进任务

1. 查阅 `docs/keycloak-theme.md` 了解当前主题结构
2. 在主题的 `messages/` 目录下创建 `messages_en.properties` 和 `messages_zh.properties`
3. 在 `theme.properties` 中启用语言支持：`locales=en,zh`
4. 翻译所有 Keycloak 页面文本（登录、注册、MFA、错误页面等）
5. 实现语言参数透传机制（从 portal 跳转至 Keycloak 时携带 `ui_locales` 参数）

---

*Ticket created manually based on QA observation*
