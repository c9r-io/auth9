# 邮件模板管理

**类型**: 功能
**严重程度**: Medium
**影响范围**: auth9-core (Backend - migration, service, API), auth9-portal (Frontend - 设置 UI)
**前置依赖**: 无
**被依赖**: 无

---

## 背景

Auth9 已有成熟的邮件发送基础设施（`EmailProvider` trait），但邮件内容模板目前是硬编码在 `auth9-core/src/email/templates/` 目录中。QA 测试 (`docs/qa/settings/03-email-templates.md`) 期望存在 `email_templates` 数据库表以支持运行时自定义邮件模板，但该表及相关功能尚未实现。

### 现有邮件模板架构

```
auth9-core/src/email/templates/
├── mod.rs              # 模板渲染引擎
├── verification.rs     # 邮件验证模板（硬编码）
├── password_reset.rs   # 密码重置模板（硬编码）
└── invitation.rs       # 邀请模板（硬编码）
```

当前所有邮件模板均为编译时硬编码，无法在运行时按租户自定义。

---

## 期望行为

### R1: 数据库表

创建 `email_templates` 表，支持按模板类型存储自定义内容：

```sql
CREATE TABLE email_templates (
    id CHAR(36) PRIMARY KEY,
    tenant_id CHAR(36),          -- NULL = system default
    template_type VARCHAR(50) NOT NULL,  -- verification, password_reset, invitation, etc.
    subject VARCHAR(255) NOT NULL,
    html_body TEXT NOT NULL,
    text_body TEXT,
    is_customized BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_email_templates_tenant_type (tenant_id, template_type)
);
```

### R2: CRUD API

- `GET /api/v1/tenants/{tenant_id}/email-templates` — 列出租户的邮件模板
- `GET /api/v1/tenants/{tenant_id}/email-templates/{type}` — 获取特定类型模板
- `PUT /api/v1/tenants/{tenant_id}/email-templates/{type}` — 更新模板
- `DELETE /api/v1/tenants/{tenant_id}/email-templates/{type}` — 重置为默认

### R3: 模板渲染回退

邮件发送时按优先级查找模板：
1. 租户自定义模板 (`tenant_id = {id}`)
2. 系统默认模板 (`tenant_id = NULL`)
3. 硬编码模板（当前行为，作为最终回退）

### R4: Portal UI

Settings → Email Templates 页面，支持预览和编辑各类型模板。

---

## 涉及文件

| 文件 | 变更 |
|------|------|
| `auth9-core/migrations/` | 新增 email_templates 表迁移 |
| `auth9-core/src/repository/` | 新增 EmailTemplateRepository |
| `auth9-core/src/service/` | 新增 EmailTemplateService |
| `auth9-core/src/domains/platform/api/` | 新增 email template API handlers |
| `auth9-core/src/email/templates/mod.rs` | 增加数据库模板查找回退 |
| `auth9-portal/app/routes/dashboard.settings.email-templates.tsx` | 新增模板管理 UI |

---

## 验证方法

1. 运行迁移后 `SHOW TABLES LIKE 'email_templates'` 返回结果
2. CRUD API 端点正常工作
3. 自定义模板后，邮件发送使用自定义内容
4. 删除自定义模板后，回退到系统默认
5. 参照 `docs/qa/settings/03-email-templates.md` 执行完整 QA 测试
