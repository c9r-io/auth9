# Ticket: 部分错误消息未映射为人类可读文本

**Created**: 2026-03-07 16:27:02
**Module**: UI / Error Handling / i18n
**Status**: FAILED

---

## 测试内容

在触发各种错误场景时，部分错误信息直接将原始错误码（error code）、技术性错误字符串或英文后端错误消息暴露给用户，而未映射为本地化的、人类可读的友好提示文本。

**Test Location**: 多个页面（登录、注册、用户管理、权限管理等）

---

## 预期结果

- 所有面向用户的错误消息应为本地化的、友好的人类可读文本
- 例如：`INVALID_CREDENTIALS` → "邮箱或密码错误，请重试"
- 例如：`USER_ALREADY_EXISTS` → "该邮箱已被注册"
- 不应向用户展示原始错误码、技术栈信息或英文机器错误字符串

---

## 再现方法

### Steps to Reproduce
1. 触发各类错误场景（登录失败、表单验证失败、权限不足、服务不可用等）
2. 观察 Toast 通知、表单错误提示、错误页面中显示的错误文本
3. 记录所有未被正确映射的错误消息

### Known Error Scenarios (待逐一验证)
- 登录失败（错误密码）
- 注册已存在邮箱
- 权限不足访问受保护资源
- 网络超时/服务不可用
- 表单字段验证失败（格式不正确等）

### Environment
- Portal: http://localhost:3000 or http://localhost:5173

---

## 实际结果

- 部分场景下用户看到原始英文错误码或技术性错误消息（具体错误码待进一步测试补充）

---

## Analysis

**Root Cause (假设)**: 
1. 前端错误处理层（API response handler）缺少完整的 error code → 友好文本的映射表
2. 部分后端返回的错误消息直接被透传到 UI 展示，未经过本地化转换
3. i18n 国际化文件（locale files）中缺少对应 error key 的翻译条目

**Severity**: Medium（影响用户体验和产品专业性）

**Related Components**: `auth9-portal` / Error Handling / i18n / API Client / Toast Notifications

## 跟进任务

1. 整理所有可能的 API 错误码列表（来自 `auth9-core` error types）
2. 检查前端 i18n 文件中已有的 error message 映射是否完整
3. 补充缺失的错误消息映射
4. 为 API Client 添加统一的 error response 拦截和映射层

---

*Ticket created manually based on QA observation*
