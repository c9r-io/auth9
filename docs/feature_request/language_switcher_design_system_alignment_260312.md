# Feature Request: 语言切换组件统一为项目设计语言组件

**Created**: 2026-03-12
**Source**: User Request

---

## 需求描述
当前语言切换组件使用原生 selector，与项目整体设计语言不一致。需要将其替换为符合现有设计系统的统一 UI 组件实现。

## 当前行为
- 语言切换功能基于原生 selector 实现
- 视觉样式、交互反馈与项目其他 UI 组件不一致
- 在 landing 等高可见页面中会削弱整体产品一致性

## 期望行为
- 语言切换组件使用项目统一的 UI 组件体系实现
- 样式、尺寸、状态反馈、可访问性与现有设计语言保持一致
- 保留当前语言切换能力，不引入功能回退

## 相关组件
- Frontend: `auth9-portal` 国际化切换组件
- Design System: 项目统一 UI 组件与样式规范

## Severity
Medium
