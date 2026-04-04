# Client Secret 显示 - Configuration Tab 密钥展示与交互

**模块**: 服务与客户端
**测试范围**: Service 详情页 Configuration 标签页中 Client Secret 的显示、遮蔽、揭示、复制及降级处理
**场景数**: 5
**优先级**: 高

---

## 背景说明

在 Service 详情页的 Configuration 标签页中，每个 Client 卡片现在除了 Client ID 外，还展示 Client Secret。Secret 默认以遮蔽形式（`••••••••••••••••••••••••`）显示，用户可通过 Eye 图标切换明文/遮蔽状态，并通过 Copy 按钮将实际 secret 复制到剪贴板。

**数据来源**: Secret 通过 Integration API (`GET /api/v1/services/{id}/integration`) 获取，匹配 `client_id` 后展示。

> **安全说明**: client_secret 仅在创建时一次性返回明文，之后数据库仅存储 bcrypt 哈希。对于 DB seed 数据或已过缓存周期的 client，Integration API 返回占位符字符串，Portal 将显示 fallback 信息。

---

## 步骤 0：Gate Check（前置条件）

1. 以管理员身份登录 auth9-portal（`http://localhost:3000`），使用 `admin@auth9.local` 账号
2. 若出现 `/tenant/select` 页面，先完成 tenant 选择
3. 确认当前 tenant 下至少存在一个 Service，且该 Service 包含至少一个 Client
4. 在侧边栏导航至 **Services**，点击目标 Service 进入详情页
5. 确认当前处于 **Configuration** 标签页

> 若需新建 Service 和 Client，可通过 Portal UI 或 API 创建：
> ```bash
> curl -s -X POST http://localhost:8080/api/v1/services \
>   -H "Authorization: Bearer {access_token}" \
>   -H "Content-Type: application/json" \
>   -d '{"name": "Secret Display Test", "redirect_uris": ["https://app.example.com/callback"]}' | jq .
> ```

---

## 场景 1：Client Secret 默认遮蔽显示

### 初始状态
- 已完成 Gate Check
- 当前在 Service 详情页 Configuration 标签页
- 该 Service 包含至少一个 Confidential Client

### 目的
验证 Client Secret 默认以遮蔽（masked）形式展示，不泄露明文

### 测试操作流程
1. 侧边栏点击 **Services** → 点击目标 Service 进入详情页
2. 在右侧 Clients 卡片区域，观察每个 Client 条目
3. 确认 Client ID 下方出现 "Client Secret" 标签
4. 检查 secret 值的显示状态

### 预期结果
- 每个 Client 条目在 Client ID 下方显示 "Client Secret" 标签文字
- Secret 值默认显示为 `••••••••••••••••••••••••`（24 个圆点）
- Secret 旁边显示两个图标按钮：Eye 图标（揭示/隐藏）和 Copy 图标（复制）
- 遮蔽状态下 Eye 图标为 `EyeOpenIcon`（表示"点击查看"）

---

## 场景 2：揭示/隐藏切换正常工作

### 初始状态
- 已完成 Gate Check
- Secret 当前为默认遮蔽状态

### 目的
验证 Eye 图标切换 secret 的明文/遮蔽显示，且状态在多个 Client 间独立

### 测试操作流程
1. 点击某个 Client 的 Eye 图标按钮
2. 观察 secret 是否从圆点变为实际值
3. 观察 Eye 图标是否变为 `EyeClosedIcon`（表示"点击隐藏"）
4. 再次点击同一 Eye 图标
5. 确认 secret 恢复为圆点遮蔽
6. 若 Service 有多个 Client，揭示其中一个 secret，确认其他 Client 的 secret 仍为遮蔽状态

### 预期结果
- 点击 Eye 图标后，secret 显示实际值（明文字符串或占位符 `"(set — use the secret configured at creation)"`）
- 揭示状态下 Eye 图标变为 `EyeClosedIcon`
- 再次点击恢复遮蔽状态，图标变回 `EyeOpenIcon`
- 多个 Client 的揭示/隐藏状态互相独立
- 切换过程无页面闪烁或布局抖动

---

## 场景 3：Copy 按钮复制实际 secret 到剪贴板

### 初始状态
- 已完成 Gate Check
- 浏览器已授予剪贴板权限

### 目的
验证 Copy 按钮复制的是实际 secret 值（非遮蔽圆点），且有视觉反馈

### 测试操作流程
1. 在 Configuration 标签页找到一个 Client 条目
2. **不揭示 secret**（保持遮蔽状态），点击 secret 旁的 Copy 图标按钮
3. 在文本编辑器中粘贴，确认粘贴内容
4. 观察 Copy 图标是否变为绿色 checkmark（&#10003;）
5. 等待约 2 秒，确认图标恢复为 Copy 图标
6. 揭示 secret，再次点击 Copy 按钮，粘贴确认内容一致

### 预期结果
- 复制的内容为实际 secret 字符串（非 `••••••••••••••••••••••••`）
- 点击 Copy 后立即显示绿色 checkmark 反馈
- 约 2 秒后 checkmark 自动恢复为 Copy 图标
- 无论 secret 处于遮蔽或揭示状态，复制的内容相同（均为实际值）
- Client ID 的 Copy 按钮与 Secret 的 Copy 按钮互不干扰

---

## 场景 4：长 secret 正确换行无溢出

### 初始状态
- 已完成 Gate Check
- 至少有一个 Client 的 secret 为长字符串（通常 UUID 格式或更长）

### 目的
验证长 secret 在不同视口宽度下正确换行，不撑破容器

### 测试操作流程
1. 揭示一个 Client 的 secret，观察明文显示
2. 确认 secret 使用 monospace 字体显示
3. 缩小浏览器窗口宽度至移动端尺寸（约 375px）
4. 观察 secret 文本是否自动换行
5. 检查 secret 文本是否出现水平溢出或横向滚动条
6. 确认 Eye 和 Copy 按钮在窄屏下仍可点击

### 预期结果
- Secret 使用 `font-mono` 等宽字体显示
- 长 secret 自动换行，使用 `break-all` 策略（任意字符处断行）
- 无水平溢出，无横向滚动条
- Eye 和 Copy 按钮不被挤压（`shrink-0`），保持可点击状态
- 移动端按钮尺寸为 `h-11 w-11`（44px 触摸目标），桌面端为 `sm:h-6 sm:w-6`

---

## 场景 5：Secret 不可用时显示 fallback 信息

### 初始状态
- 已完成 Gate Check
- 至少有一个 Client 的 secret 在 Integration API 中不可用（`client_secret` 为 `null` 或该 client 为 Public Client）

### 目的
验证当 secret 数据不可用时，UI 显示 fallback 提示而非空白或报错

### 测试操作流程
1. 使用 DB seed 创建的 Public Client，或 Integration API 返回 `client_secret: null` 的场景
2. 在 Configuration 标签页观察该 Client 条目的 secret 区域
3. 确认不显示 Eye 和 Copy 按钮
4. 确认显示 fallback 提示文字

### 预期结果
- 当 `client_secret` 为 `null` 或空时，显示斜体提示信息（对应 i18n key: `services.integration.clientSecretUnavailable`）
- 不显示 Eye 揭示/隐藏按钮
- 不显示 Copy 按钮
- 提示文字使用 `text-[var(--text-secondary)]` 次要文字颜色
- 页面不崩溃，其他 Client 的 secret 展示不受影响

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 备注 |
|---|------|------|----------|----------|------|
| 1 | Client Secret 默认遮蔽显示 | ☐ | | | |
| 2 | 揭示/隐藏切换正常工作 | ☐ | | | |
| 3 | Copy 按钮复制实际 secret 到剪贴板 | ☐ | | | |
| 4 | 长 secret 正确换行无溢出 | ☐ | | | |
| 5 | Secret 不可用时显示 fallback 信息 | ☐ | | | |
