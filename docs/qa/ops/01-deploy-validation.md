# 运维 - 部署校验与开发环境初始化

**模块**: 运维（ops）
**测试范围**: K8s 部署占位符拦截、docker-compose 密钥外部化、开发环境初始化脚本
**场景数**: 5

---

## 背景

Auth9 的 K8s ConfigMap 中包含 6 个必须替换的 `example.com` 占位符域名。若未替换即部署，OAuth 回调、CORS、WebAuthn 全部失效。`deploy/upgrade.sh` 现已在升级前自动检查这些占位符。

同时，JWT RSA 密钥已从 `docker-compose.yml` 内联值外部化到 `.env` 文件，首次开发需运行 `scripts/init-dev-env.sh` 生成密钥。

---

## 场景 1：upgrade.sh 拦截未替换的 ConfigMap 占位符

### 初始状态
- K8s 集群可达，`auth9` 命名空间存在
- ConfigMap `auth9-config` 包含 `example.com` 占位符（未修改的默认值）

### 目的
验证 `upgrade.sh` 在检测到 `example.com` 占位符时中止升级

### 测试操作流程

```bash
# 确认 ConfigMap 包含占位符
kubectl get configmap auth9-config -n auth9 -o jsonpath='{.data.JWT_ISSUER}'
# 预期: https://api.auth9.example.com

# 执行升级（dry-run 模式）
./deploy/upgrade.sh --dry-run
# 预期: 输出包含以下错误并以非零退出码终止：
#   ✗ JWT_ISSUER 仍是示例域名: https://api.auth9.example.com
#   ✗ WEBAUTHN_RP_ID 仍是示例域名: auth9.example.com
#   ✗ CORS_ALLOWED_ORIGINS 仍是示例域名: https://auth9.example.com
#   ✗ APP_BASE_URL 仍是示例域名: https://auth9.example.com
#   ✗ AUTH9_CORE_PUBLIC_URL 仍是示例域名: https://api.auth9.example.com
#   ✗ AUTH9_PORTAL_URL 仍是示例域名: https://auth9.example.com
#   ✗ ConfigMap 包含未替换的 example.com 占位符，中止升级

echo $?
# 预期: 1
```

### 预期结果
- 列出全部 6 个占位符字段名和当前值
- 打印修复提示（指向 configmap.yaml 中的 REQUIRED 注释）
- 退出码非零，未执行任何 kubectl rollout 操作

---

## 场景 2：upgrade.sh --skip-validation 跳过检查

### 初始状态
- 同场景 1

### 目的
验证 `--skip-validation` 参数可跳过占位符检查

### 测试操作流程

```bash
./deploy/upgrade.sh --dry-run --skip-validation
# 预期: 输出包含 "⚠ 跳过 ConfigMap 占位符检查 (--skip-validation)"
# 并继续执行升级流程（dry-run 模式下显示将要执行的命令）
```

### 预期结果
- 输出警告信息但不中止
- 后续 dry-run 输出正常的升级流程

---

## 场景 3：validate-k8s-deploy.sh 检查全部 6 个域名字段

### 初始状态
- K8s 集群可达，ConfigMap 含占位符

### 目的
验证部署校验脚本检查全部 6 个 `example.com` 字段（此前仅检查 3 个）

### 测试操作流程

```bash
./scripts/validate-k8s-deploy.sh --skip-public
# 预期输出中包含以下 6 项检查结果（全部为 ✗）：
#   ✗ APP_BASE_URL 仍是示例域名
#   ✗ AUTH9_CORE_PUBLIC_URL 仍是示例域名
#   ✗ AUTH9_PORTAL_URL 仍是示例域名
#   ✗ JWT_ISSUER 仍是示例域名
#   ✗ WEBAUTHN_RP_ID 仍是示例域名
#   ✗ CORS_ALLOWED_ORIGINS 仍是示例域名
```

### 预期结果
- 6 个 `check_not_example_domain` 检查全部触发
- 脚本以非零退出码结束

---

## 场景 4：init-dev-env.sh 一键生成开发环境密钥

### 初始状态
- 项目根目录不存在 `.env` 文件（或存在但 JWT 密钥为空）
- `.env.example` 存在

### 测试操作流程

#### 步骤 0: 验证环境状态
```bash
# 备份现有 .env（如有）
[ -f .env ] && cp .env .env.backup

# 移除 .env 以测试首次生成
rm -f .env

# 同时清理旧的 dev 密钥
rm -f deploy/dev-certs/jwt/private.key deploy/dev-certs/jwt/public.key
```

#### 步骤 1: 执行初始化
```bash
./scripts/init-dev-env.sh
# 预期输出:
#   === Auth9 Dev Environment Setup ===
#   [1/3] Created .env from .env.example
#   [2/3] Generating random secrets...
#     Generated JWT_SECRET
#     Generated SESSION_SECRET
#     Generated PASSWORD_RESET_HMAC_KEY
#     Generated SETTINGS_ENCRYPTION_KEY
#   [3/3] Generating JWT RSA key pair...
#   ...
#   === Setup complete ===
```

#### 步骤 2: 验证生成结果
```bash
# 验证 .env 中密钥非空
grep '^JWT_PRIVATE_KEY=-----' .env | wc -l
# 预期: 1

grep '^JWT_PUBLIC_KEY=-----' .env | wc -l
# 预期: 1

grep '^JWT_SECRET=' .env | cut -d= -f2 | wc -c
# 预期: 65（64 hex chars + newline）

grep '^SETTINGS_ENCRYPTION_KEY=' .env | cut -d= -f2 | wc -c
# 预期: >40（base64 编码的 32 字节）

# 验证 dev-certs 密钥文件
openssl rsa -in deploy/dev-certs/jwt/private.key -check -noout
# 预期: RSA key ok
```

#### 步骤 3: 验证 docker-compose 可解析
```bash
docker-compose config > /dev/null 2>&1
echo $?
# 预期: 0
```

#### 清理
```bash
# 恢复原 .env
[ -f .env.backup ] && mv .env.backup .env
```

### 预期结果
- `.env` 从模板创建，所有必填密钥生成且非空
- `deploy/dev-certs/jwt/private.key` 是有效 RSA 2048 密钥
- `docker-compose config` 成功解析（无 `JWT_PRIVATE_KEY must be set` 错误）

---

## 场景 5：docker-compose.yml 缺少 .env 时报错清晰

### 初始状态
- 不存在 `.env` 文件，或 `.env` 中无 `JWT_PRIVATE_KEY`

### 目的
验证 docker-compose 在缺少必需密钥时给出明确错误提示

### 测试操作流程

```bash
# 备份并清空 .env
[ -f .env ] && cp .env .env.backup
echo "SETTINGS_ENCRYPTION_KEY=test" > .env

# 尝试启动
docker-compose up auth9-core 2>&1 | head -5
# 预期: 包含 "JWT_PRIVATE_KEY must be set in .env" 错误信息

# 恢复
[ -f .env.backup ] && mv .env.backup .env
```

### 预期结果
- docker-compose 拒绝启动并显示清晰的错误信息
- 错误信息包含 `run scripts/init-dev-env.sh` 修复提示

---

## 检查清单

| # | 场景 | 状态 | 测试日期 | 测试人员 | 发现问题 |
|---|------|------|----------|----------|----------|
| 1 | upgrade.sh 拦截未替换的 ConfigMap 占位符 | ☐ | | | |
| 2 | upgrade.sh --skip-validation 跳过检查 | ☐ | | | |
| 3 | validate-k8s-deploy.sh 检查全部 6 个域名字段 | ☐ | | | |
| 4 | init-dev-env.sh 一键生成开发环境密钥 | ☐ | | | |
| 5 | docker-compose.yml 缺少 .env 时报错清晰 | ☐ | | | |
