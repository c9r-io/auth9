# UI/UX 测试 - 公共页面布局（Privacy / Terms / Docs）

**模块**: 页面专项
**测试范围**: PublicPageLayout 共享布局组件、prose-glass 排版样式、公共页面 Liquid Glass 一致性
**场景数**: 5

---

## 背景说明

Auth9 新增三个公共页面：`/privacy`（隐私政策）、`/terms`（服务条款）、`/docs`（文档导航）。三者共享 `PublicPageLayout` 布局组件，提供：

- `page-backdrop` 动态背景
- 简化 header：Auth9 logo（链回 `/`）+ LanguageSwitcher + ThemeToggle
- `max-w-3xl` 居中内容区 + `h1` 标题 + `.prose-glass` 排版
- Footer：copyright + Privacy/Terms 链接

`.prose-glass` 是新增的排版样式类，使用设计系统 token（`--text-primary`、`--text-secondary`、`--accent-blue`），不依赖 `@tailwindcss/typography`。

---

## 场景 1：PublicPageLayout Header 与 Landing Header 风格一致

### 初始状态
- 用户未登录
- 桌面端（宽度 ≥ 1024px）

### 目的
验证公共页面 Header 与 Landing 页面 Header 使用相同的 Liquid Glass 风格，但结构更简化（无导航链接、无 Sign In 按钮）。

### 测试操作流程
1. 打开 Landing 页面（`/`），截图 Header 区域
2. 打开 `/privacy`，截图 Header 区域
3. 对比两者的毛玻璃效果、边框、阴影
4. 确认公共页面 Header 包含：Auth9 Logo + 语言切换 + 主题切换
5. 确认不包含：导航链接、Sign In 按钮、Get Started 按钮
6. 点击 Auth9 Logo，确认跳转回首页（`/`）

### 预期视觉效果
- Header 使用 `liquid-glass` 类，具有毛玻璃效果（`backdrop-filter: blur(...)`, `background: var(--glass-bg)`）
- Header 固定在顶部（`position: fixed`）
- Logo 与 Landing 页面相同风格（渐变背景 + "A9" 白色文字）
- 语言切换和主题切换按钮与 Landing Navbar 中的外观一致

---

## 场景 2：prose-glass 排版样式验证（Light / Dark）

### 初始状态
- 打开 `/privacy`（包含多个 h2 标题和正文段落）

### 目的
验证 `.prose-glass` 排版样式在 Light 和 Dark 模式下的视觉效果，颜色 token 正确应用。

### 测试操作流程
1. 在 Light 模式下检查：
   - 正文颜色是否为 `var(--text-secondary)` → `#6E6E73`
   - h2 标题颜色是否为 `var(--text-primary)` → `#1D1D1F`
   - 行高是否约为 `1.75`
   - 段落间距是否合理（约 `1.25em`）
2. 切换到 Dark 模式
3. 检查相同元素：
   - 正文颜色应切换为 `#98989D`
   - h2 标题颜色应切换为 `#FFFFFF`
4. 确认链接（如有）颜色为 `var(--accent-blue)` → `#007AFF`

### 预期视觉效果
- Light 模式：正文深灰，标题近黑，背景浅色渐变
- Dark 模式：正文浅灰，标题白色，背景深色渐变
- 两种模式下文字均有足够对比度，可读性良好
- 切换模式时平滑过渡，不闪烁

### 验证工具
```javascript
const proseEl = document.querySelector('.prose-glass');
if (proseEl) {
  const styles = getComputedStyle(proseEl);
  console.log('prose-glass color:', styles.color);
  console.log('prose-glass lineHeight:', styles.lineHeight);

  const h2 = proseEl.querySelector('h2');
  if (h2) console.log('h2 color:', getComputedStyle(h2).color);
}
```

---

## 场景 3：内容区域居中与最大宽度约束

### 初始状态
- 桌面端（宽度 ≥ 1440px）

### 目的
验证公共页面内容不会在超宽屏幕上拉伸到全宽，保持阅读舒适度。

### 测试操作流程
1. 在 1440px 宽度下打开 `/privacy`
2. 确认内容区域水平居中，两侧有明显留白
3. 调整宽度到 1920px，确认内容不继续拉伸
4. 调整宽度到 768px（平板），确认内容仍可读，不溢出
5. 对 `/terms` 和 `/docs` 重复检查

### 预期视觉效果
- 内容区域最大宽度为 `max-w-3xl`（48rem = 768px）
- Header 和 Footer 同样约束在 `max-w-3xl`
- 在任何宽度下内容不超出容器边界
- 移动端（768px 以下）内容区域有 `px-6` 左右留白

### 验证工具
```javascript
const main = document.querySelector('main > div');
if (main) {
  const rect = main.getBoundingClientRect();
  console.log('Content width:', rect.width, 'max expected: 768px');
  console.log('Left offset:', rect.left, 'Right offset:', window.innerWidth - rect.right);
}
```

---

## 场景 4：Docs 页面卡片网格布局

### 初始状态
- 桌面端（宽度 ≥ 768px）
- 打开 `/docs`

### 目的
验证 Docs 页面的卡片网格使用 Liquid Glass 卡片组件，布局合理。

### 测试操作流程
1. 确认 3 张卡片呈 2 列网格布局（`sm:grid-cols-2`）
2. 确认卡片使用 `liquid-glass` 样式（毛玻璃效果）
3. 将鼠标悬停在 GitHub 卡片上，确认有悬停效果（上移、阴影增强）
4. 确认 "Coming soon" / "即将推出" 徽章使用圆角药丸样式
5. 缩小到 640px 以下，确认卡片变为单列堆叠

### 预期视觉效果
- 卡片间距均匀（`gap-4` = 16px）
- 卡片有 `liquid-glass` 毛玻璃效果
- 徽章使用 `secondary` 变体：浅色背景 + 灰色文字 + 细边框
- GitHub 卡片悬停时边框变为 `var(--accent-blue)`
- 移动端单列布局，卡片全宽显示

---

## 场景 5：公共页面 Footer 与 Landing Footer 一致性

### 初始状态
- 打开 `/privacy`、`/terms`、`/docs` 中任一页面

### 目的
验证公共页面 Footer 结构与 Landing 页面 Footer 一致，链接正确。

### 测试操作流程
1. 滚动到页面底部
2. 确认 Footer 包含 copyright 文案和 Privacy/Terms 链接
3. 确认 Privacy 链接指向 `/privacy`
4. 确认 Terms 链接指向 `/terms`
5. 在 `/privacy` 页面点击 Footer 中的「条款」链接，确认跳转到 `/terms`
6. 在 `/terms` 页面点击 Footer 中的「隐私」链接，确认跳转到 `/privacy`
7. 切换语言，确认 Footer 文案跟随翻译

### 预期视觉效果
- Footer 上方有 `border-top`（使用 `var(--glass-border-subtle)`）
- Copyright 文案位于左侧，链接位于右侧
- 链接颜色为 `var(--text-tertiary)`，悬停变为 `var(--text-primary)`
- Footer 最大宽度与 Header/内容区域一致（`max-w-3xl`）

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | PublicPageLayout Header 风格一致性 | ☐ | | | 对比 Landing Header |
| 2 | prose-glass 排版样式（Light / Dark）| ☐ | | | 颜色 token 验证 |
| 3 | 内容区域居中与最大宽度约束 | ☐ | | | 多断点测试 |
| 4 | Docs 页面卡片网格布局 | ☐ | | | 含悬停效果与响应式 |
| 5 | 公共页面 Footer 一致性 | ☐ | | | 含跨页面链接验证 |
