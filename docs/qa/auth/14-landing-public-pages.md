# 认证流程 - Landing 公共页面（Privacy / Terms / Docs）

**模块**: 认证流程
**测试范围**: Landing 页面 Footer 链接可达性、Privacy / Terms / Docs 页面渲染、PublicPageLayout 共享布局、三语翻译完整性
**场景数**: 5
**优先级**: 中

---

## 背景说明

Landing 页面（`/`）的 Footer 包含指向 `/privacy` 和 `/terms` 的链接，Hero 区域包含指向 `/docs` 的按钮。这三个路由此前指向不存在的页面（404）。现已补完为：

- `/privacy` — 隐私政策页面，6 个章节
- `/terms` — 服务条款页面，8 个章节
- `/docs` — 文档导航页面，卡片网格布局（Getting Started、API Reference、GitHub）

三个页面共享 `PublicPageLayout` 布局组件（简化 header + 居中内容 + footer），内容通过 i18n 翻译键渲染，支持 `zh-CN` / `en-US` / `ja` 三语。

---

## 场景 1：Landing Footer 链接入口可见性与可达性

### 初始状态
- 用户未登录
- 浏览器访问 `http://localhost:3000`

### 目的
验证用户可以从 Landing 页面的 Footer 导航到 Privacy 和 Terms 页面，从 Hero 区域导航到 Docs 页面，不再出现 404。

### 测试操作流程
1. 打开 Landing 页面（`/`）
2. 滚动到页面底部 Footer 区域
3. 确认存在「隐私」和「条款」（或对应当前语言的文案）两个链接
4. 点击「隐私」链接
5. 确认跳转到 `/privacy`，页面正常渲染
6. 点击浏览器返回，回到 Landing 页面
7. 点击「条款」链接
8. 确认跳转到 `/terms`，页面正常渲染
9. 返回 Landing 页面，点击 Hero 区域的「阅读文档」按钮
10. 确认跳转到 `/docs`，页面正常渲染

### 预期结果
- Footer 中「隐私」链接指向 `/privacy`，「条款」链接指向 `/terms`
- Hero 区域「阅读文档」按钮指向 `/docs`
- 三个页面均正常渲染，不出现 404
- 三个页面均包含 Auth9 logo（链回 `/`）、语言切换器、主题切换器

---

## 场景 2：Privacy 页面内容完整性与布局

### 初始状态
- 用户未登录
- 浏览器访问 `http://localhost:3000/privacy`

### 目的
验证隐私政策页面包含完整的 6 个章节，布局符合 Liquid Glass 设计语言。

### 测试操作流程
1. 打开 `/privacy`
2. 确认页面标题为「隐私政策」（中文）/ "Privacy Policy"（英文）/ "プライバシーポリシー"（日语）
3. 确认「最后更新」日期显示
4. 逐一检查 6 个章节：数据收集、数据使用、数据共享、Cookie、安全措施、联系方式
5. 确认每个章节有标题（`h2`）和正文内容
6. 确认 Header 中 Auth9 Logo 可点击回到首页
7. 确认 Footer 中 Privacy/Terms 链接存在

### 预期结果
- 页面标题使用 `h1`，字体大且加粗，使用 `var(--text-primary)` 颜色
- 6 个章节标题使用 `h2`，正文使用 `.prose-glass` 排版样式
- 正文颜色为 `var(--text-secondary)`，行高 `1.75`
- 章节标题颜色为 `var(--text-primary)`
- 背景使用 `page-backdrop` 动态渐变
- Header 使用 `liquid-glass` 毛玻璃效果

---

## 场景 3：Terms 页面内容完整性

### 初始状态
- 用户未登录
- 浏览器访问 `http://localhost:3000/terms`

### 目的
验证服务条款页面包含完整的 8 个章节。

### 测试操作流程
1. 打开 `/terms`
2. 确认页面标题正确（「服务条款」/「Terms of Service」/「利用規約」）
3. 确认「最后更新」日期显示
4. 逐一检查 8 个章节：条款接受、账户责任、使用规范、知识产权、责任限制、终止、管辖法律、联系方式
5. 确认每个章节有标题和正文

### 预期结果
- 8 个章节均完整渲染，无翻译键泄露（不显示 `legal.terms.sections.xxx`）
- 布局与 Privacy 页面一致（共享 `PublicPageLayout`）
- Header 和 Footer 结构与 Privacy 页面相同

---

## 场景 4：Docs 文档导航页面

### 初始状态
- 用户未登录
- 浏览器访问 `http://localhost:3000/docs`

### 目的
验证文档导航页面显示卡片网格，Getting Started 和 API Reference 显示"即将推出"徽章，GitHub 卡片可外链。

### 测试操作流程
1. 打开 `/docs`
2. 确认页面标题为「文档」/「Documentation」/「ドキュメント」
3. 确认页面描述文案存在
4. 检查「快速入门」/「Getting Started」卡片，确认有"即将推出"/"Coming soon"/"近日公開" 徽章
5. 检查「API 参考」/「API Reference」卡片，确认有相同徽章
6. 检查「GitHub」卡片，确认有外链图标
7. 点击 GitHub 卡片，确认在新标签页打开 GitHub 仓库页面

### 预期结果
- 三张卡片使用 `liquid-glass` 样式，呈网格布局（桌面端 2 列）
- "Coming soon" 徽章使用 `Badge` 组件，`variant="secondary"`
- GitHub 卡片包含外链图标（箭头指向右上方）
- GitHub 链接包含 `target="_blank"` 和 `rel="noopener noreferrer"`
- Getting Started 和 API Reference 卡片不可点击（无链接）

---

## 场景 5：三语翻译完整性（公共页面）

### 初始状态
- 用户未登录
- 当前语言为 `zh-CN`

### 目的
验证 Privacy、Terms、Docs 页面在三种语言下均无翻译键泄露，内容完整切换。

### 测试操作流程
1. 打开 `/privacy`
2. 确认中文内容显示正确
3. 使用语言切换器切换到 English
4. 确认所有章节标题和正文切换为英文，无残留中文或翻译键
5. 切换到日本語
6. 确认所有章节标题和正文切换为日文
7. 对 `/terms` 重复步骤 2~6
8. 对 `/docs` 重复步骤 2~6，特别确认"Coming soon"徽章文案正确切换

### 预期结果
- 中文：「隐私政策」「服务条款」「文档」「即将推出」
- 英文："Privacy Policy" "Terms of Service" "Documentation" "Coming soon"
- 日文：「プライバシーポリシー」「利用規約」「ドキュメント」「近日公開」
- 无翻译键泄露（不显示 `legal.xxx` 或 `docs.xxx` 形式的原始键名）
- 语言切换后页面不闪烁、不重载

---

## 通用场景：公共页面无需认证

### 测试操作流程
1. 清除所有 Cookie 和 Session（无痕模式或清除 Cookie）
2. 直接访问 `/privacy`、`/terms`、`/docs`

### 预期结果
- 三个页面均可在未登录状态下正常访问
- 不出现登录跳转或 401/403 错误

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | Landing Footer 链接入口可见性与可达性 | ☐ | | | 含 Hero 区域「阅读文档」按钮 |
| 2 | Privacy 页面内容完整性与布局 | ☐ | | | 6 个章节 + prose-glass 排版 |
| 3 | Terms 页面内容完整性 | ☐ | | | 8 个章节 |
| 4 | Docs 文档导航页面 | ☐ | | | 卡片网格 + Coming soon 徽章 |
| 5 | 三语翻译完整性（公共页面） | ☐ | | | Privacy + Terms + Docs 三页 × 三语 |
| - | 公共页面无需认证（通用） | ☐ | | | |
