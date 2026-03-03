# 高级攻击 - gRPC 安全测试

**模块**: 高级攻击
**测试范围**: gRPC 认证、授权、协议攻击
**场景数**: 5
**风险等级**: 🔴 极高
**ASVS 5.0 矩阵ID**: M-ADV-02
**OWASP ASVS 5.0**: V4.1,V4.2,V8.1,V13.2
**回归任务映射**: Backlog #3, #20


---

## 背景

Auth9 Core 提供 gRPC API (端口 50051) 供业务服务调用。gRPC 使用 HTTP/2 和 Protocol Buffers，具有独特的安全挑战。

**当前状态**: gRPC 已支持 `GRPC_AUTH_MODE`（`none`/`api_key`/`mtls`），且生产环境对 `none` 与空 `GRPC_API_KEYS` 有启动失败保护；当前重点是防止认证配置回归与绕过。

**相关标准**:
- OWASP API Security Top 10: API1 - Broken Object Level Authorization
- CWE-287: Improper Authentication
- CWE-306: Missing Authentication for Critical Function（主要关注误配置退化路径）

---

## 场景 1：未认证的 gRPC 调用（认证配置回归）

### 前置条件
- Auth9 Core gRPC 服务运行在 localhost:50051
- 安装 grpcurl 工具

### 攻击目标
验证 gRPC 端点在不同认证配置下是否出现未认证访问回归

### 攻击步骤
1. 列出可用的 gRPC 服务：
   ```bash
   grpcurl -plaintext localhost:50051 list
   ```

2. 查看服务方法：
   ```bash
   grpcurl -plaintext localhost:50051 list auth9.TokenExchange
   ```

3. 调用核心方法（无认证）：
   ```bash
   # 不带认证调用 ValidateToken（仅示例）
   grpcurl -plaintext -d '{"access_token":"dummy"}' \
     localhost:50051 auth9.TokenExchange/ValidateToken
   
   # 不带认证调用 ExchangeToken（仅示例）
   grpcurl -plaintext -d '{"identity_token":"dummy","tenant_id":"dummy","service_id":"dummy"}' \
     localhost:50051 auth9.TokenExchange/ExchangeToken
   ```

4. 使用 Python grpc 客户端绕过限制：
   ```python
   import grpc
   from auth9_pb2 import ValidateTokenRequest
   from auth9_pb2_grpc import TokenExchangeStub
   
   channel = grpc.insecure_channel('localhost:50051')
   stub = TokenExchangeStub(channel)
   
   # 回归检查：无凭证调用
   response = stub.ValidateToken(ValidateTokenRequest(access_token="dummy"))
   print(response)
   ```

### 预期安全行为（修复后）
- 所有 gRPC 方法都需要有效的身份验证
- 未认证请求返回 `UNAUTHENTICATED` (gRPC 状态码 16)
- 错误信息不泄露服务内部细节

### 验证方法
```bash
# 安全基线（production + api_key）：未认证请求应失败
grpcurl -plaintext localhost:50051 list

# 非生产/误配置回归检查：若出现可匿名调用，应标记高风险缺陷
grpcurl -plaintext -d '{"access_token":"dummy"}' \
  localhost:50051 auth9.TokenExchange/ValidateToken
```

### 修复建议
- 实现 gRPC Interceptor 进行认证：
  ```rust
  // auth9-core/src/grpc/interceptor.rs
  use tonic::{Request, Status};
  
  pub fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
      // 检查 Authorization header
      let token = req.metadata()
          .get("authorization")
          .ok_or_else(|| Status::unauthenticated("Missing authorization token"))?;
      
      // 验证 JWT token
      validate_jwt(token)?;
      
      Ok(req)
  }
  ```
- 使用 mTLS (Mutual TLS) 进行双向认证
- 实施 IP 白名单限制内部服务调用

---

## 场景 2：mTLS 证书验证绕过

### 前置条件
- Docker 服务运行中（`docker compose up -d`）
- gRPC 服务通过 nginx mTLS 代理暴露在 `localhost:50051`
- **重要：必须从宿主机连接 `localhost:50051`（nginx mTLS 代理），不要从 Docker 容器内直接连接 `auth9-core:50051`（明文 gRPC，无 TLS）**

### 攻击目标
验证证书链验证的正确性

### 攻击步骤
1. 尝试使用自签名证书连接：
   ```bash
   # 生成自签名证书
   openssl req -x509 -newkey rsa:2048 -keyout /tmp/test-key.pem -out /tmp/test-cert.pem -days 1 -nodes -subj "/CN=test"

   # 从宿主机尝试连接（必须使用 localhost:50051，即 nginx mTLS 代理）
   grpcurl -cert /tmp/test-cert.pem -key /tmp/test-key.pem \
     -cacert deploy/dev-certs/grpc/ca.crt localhost:50051 list
   # 预期：400 Bad Request（nginx 拒绝未受信任的客户端证书）
   ```

2. 尝试使用过期证书：
   ```bash
   # 使用已过期的证书
   grpcurl -cert expired-cert.pem -key expired-key.pem \
     -cacert ca.pem localhost:50051 list
   ```

3. 尝试使用被吊销的证书：
   ```bash
   # 使用 CRL (Certificate Revocation List) 中的证书
   grpcurl -cert revoked-cert.pem -key revoked-key.pem \
     -cacert ca.pem localhost:50051 list
   ```

4. 中间人攻击（证书固定测试）：
   ```bash
   # 使用不同的 CA 签发的证书
   grpcurl -cert other-ca-cert.pem -key other-ca-key.pem \
     -cacert other-ca.pem localhost:50051 list
   ```

### 预期安全行为
- 自签名证书被拒绝（nginx 返回 HTTP 400 Bad Request）
- 过期证书被拒绝
- 被吊销的证书被拒绝（需要 OCSP 或 CRL 检查）
- 证书固定（Certificate Pinning）防止中间人攻击
- 合法客户端证书（由 CA 签发）应能正常连接

### 验证方法
```bash
# 1. 验证自签名证书被拒绝（预期：400 Bad Request）
openssl req -x509 -newkey rsa:2048 -keyout /tmp/test-key.pem -out /tmp/test-cert.pem -days 1 -nodes -subj "/CN=test"
grpcurl -cert /tmp/test-cert.pem -key /tmp/test-key.pem \
  -cacert deploy/dev-certs/grpc/ca.crt localhost:50051 list

# 2. 验证合法证书可以连接（预期：成功建立连接）
grpcurl -cert deploy/dev-certs/grpc/client.crt -key deploy/dev-certs/grpc/client.key \
  -cacert deploy/dev-certs/grpc/ca.crt localhost:50051 list
```

### 常见误报排查

| 症状 | 原因 | 解决方法 |
|------|------|----------|
| 自签名证书连接成功 | 直接连接了 `auth9-core:50051`（明文），绕过了 nginx mTLS 代理 | 确保连接 `localhost:50051`（nginx） |
| 连接被拒但不是 400 | 可能是 TLS 握手失败，`-cacert` 参数使用了错误的 CA 证书 | 使用 `deploy/dev-certs/grpc/ca.crt` |
| 从 Docker 容器内测试通过 | 容器内 `auth9-grpc-tls:50051` 指向 nginx，但 `auth9-core:50051` 是明文端口 | 测试必须使用 `auth9-grpc-tls:50051` 而非 `auth9-core:50051` |
| `-plaintext` 连接 nginx 成功 | 实际连接了 `auth9-core:50051`（明文）而非 `auth9-grpc-tls:50051`（nginx mTLS） | nginx 配置 `listen 50051 ssl http2;` 会拒绝明文连接；确认目标主机名正确 |
| 宿主机 `-plaintext localhost:50051` 成功 | 不可能，`localhost:50051` 映射到 nginx（强制 SSL） | 如确实成功，检查是否有其他进程占用 50051 端口：`lsof -i :50051` |

### 修复建议
- 使用 `tonic` 的 TLS 配置：
  ```rust
  use tonic::transport::ServerTlsConfig;
  
  let cert = tokio::fs::read("server-cert.pem").await?;
  let key = tokio::fs::read("server-key.pem").await?;
  let ca = tokio::fs::read("ca-cert.pem").await?;
  
  let tls_config = ServerTlsConfig::new()
      .identity(Identity::from_pem(cert, key))
      .client_ca_root(Certificate::from_pem(ca));  // 验证客户端证书
  
  Server::builder()
      .tls_config(tls_config)?
      .add_service(service)
      .serve(addr)
      .await?;
  ```
- 启用 OCSP Stapling 检查证书吊销
- 实施证书固定（Certificate Pinning）

---

## 场景 3：gRPC 元数据注入攻击

### 前置条件
- gRPC 服务已实现认证
- 攻击者可发送自定义 metadata

### 攻击目标
验证 gRPC metadata 处理的安全性

### 攻击步骤
1. 注入恶意 metadata 头：
   ```bash
   # 尝试注入 SQL 注入 payload
   grpcurl -plaintext \
     -H "Authorization: Bearer {valid_token}" \
     -H "X-Tenant-Id: 1' OR '1'='1" \
     -d '{"page": 1}' \
     localhost:50051 auth9.Auth9Service/ListUsers
   ```

2. 尝试伪造用户身份：
   ```bash
   grpcurl -plaintext \
     -H "Authorization: Bearer {valid_token}" \
     -H "X-User-Id: admin-user-id" \
     -H "X-Is-Admin: true" \
     -d '{"page": 1}' \
     localhost:50051 auth9.Auth9Service/ListTenants
   ```

3. Header 注入攻击：
   ```bash
   # 尝试注入换行符
   grpcurl -plaintext \
     -H "Authorization: Bearer {token}\r\nX-Admin: true" \
     -d '{"page": 1}' \
     localhost:50051 auth9.Auth9Service/ListUsers
   ```

4. Oversized metadata DoS：
   ```bash
   # 发送超大 metadata
   grpcurl -plaintext \
     -H "X-Large-Header: $(python3 -c 'print("A"*1000000)')" \
     -d '{"page": 1}' \
     localhost:50051 auth9.Auth9Service/ListUsers
   ```

### 预期安全行为
- metadata 值经过严格验证和清理
- 不信任客户端提供的身份信息（X-User-Id 等）
- 拒绝超大 metadata（返回 `RESOURCE_EXHAUSTED`）
- SQL 注入 payload 被转义或拒绝

### 验证方法
```bash
# 检查 gRPC interceptor 代码
grep -r "metadata" auth9-core/src/grpc/

# 测试 metadata 大小限制
grpcurl -plaintext -H "X-Test: $(head -c 10M < /dev/zero | tr '\0' 'A')" \
  localhost:50051 list
```

### 修复建议
- 在 Interceptor 中验证 metadata：
  ```rust
  fn validate_metadata(req: &Request<()>) -> Result<(), Status> {
      let metadata = req.metadata();
      
      // 限制 header 大小
      if metadata.len() > 100 {
          return Err(Status::invalid_argument("Too many headers"));
      }
      
      // 验证关键 header
      if let Some(tenant_id) = metadata.get("x-tenant-id") {
          validate_uuid(tenant_id)?;
      }
      
      Ok(())
  }
  ```
- 设置 metadata 大小限制（默认 8KB）
- 不从 metadata 中提取敏感信息（如用户 ID）

---

## 场景 4：gRPC 拒绝服务 (DoS) 攻击

### 前置条件
- gRPC 服务对外可访问

### 攻击目标
验证系统对 gRPC DoS 攻击的抵抗力

### 攻击步骤
1. **Slowloris 攻击**（慢速连接）：
   ```python
   import grpc
   import time
   
   # 打开大量慢速连接
   channels = []
   for i in range(1000):
       channel = grpc.insecure_channel('localhost:50051')
       channels.append(channel)
       time.sleep(0.1)  # 慢速建立连接
   
   # 保持连接打开但不发送请求
   time.sleep(3600)
   ```

2. **大payload 攻击**：
   ```bash
   # 发送超大请求体
   grpcurl -plaintext -d @large-payload.json \
     localhost:50051 auth9.Auth9Service/CreateUser
   
   # large-payload.json 包含 100MB 数据
   ```

3. **流式 RPC 滥用**：
   ```python
   import grpc
   from auth9_pb2_grpc import Auth9ServiceStub
   
   channel = grpc.insecure_channel('localhost:50051')
   stub = Auth9ServiceStub(channel)
   
   # 打开流但不读取响应
   stream = stub.StreamUsers(request)
   # 不调用 next() 读取，导致服务器缓冲区积压
   time.sleep(3600)
   ```

4. **并发连接耗尽**：
   ```bash
   # 使用 ghz 进行压力测试
   ghz --insecure \
     --connections=10000 \
     --duration=60s \
     --proto=auth9.proto \
     --call=auth9.Auth9Service/ListTenants \
     localhost:50051
   ```

5. **超大 Metadata 头攻击**：
   ```bash
   # 生成 100KB 的 header 值
   LARGE_HEADER=$(python3 -c "print('A' * 102400)")
   grpcurl -cert client.crt -key client.key -cacert ca.crt \
     -H "x-api-key: dev-grpc-api-key" \
     -H "X-Large-Header: ${LARGE_HEADER}" \
     -d '{"access_token":"dummy"}' \
     auth9-grpc-tls:50051 auth9.TokenExchange/ValidateToken
   ```

### 预期安全行为
- 限制并发连接数（如 1000）
- 限制请求体大小（如 4MB）
- **限制 metadata 大小（8KB），超大 metadata 返回错误而非关闭连接**
- 流式 RPC 超时机制（idle timeout）
- 连接速率限制（rate limiting）
- 返回 `RESOURCE_EXHAUSTED` 而不是崩溃

### 验证方法
```bash
# 压力测试
ghz --insecure --connections=100 --duration=10s \
  --proto=auth9.proto --call=auth9.Auth9Service/ListTenants \
  localhost:50051

# 监控服务器资源
htop  # 观察 CPU/内存使用
netstat -an | grep 50051 | wc -l  # 连接数
```

### 修复建议
- 配置 gRPC 服务器限制：
  ```rust
  Server::builder()
      .max_concurrent_streams(100)  // 限制并发流
      .max_frame_size(Some(4 * 1024 * 1024))  // 限制帧大小
      .tcp_keepalive(Some(Duration::from_secs(60)))
      .http2_keepalive_interval(Some(Duration::from_secs(30)))
      .http2_keepalive_timeout(Some(Duration::from_secs(10)))
      .add_service(service)
      .serve(addr)
      .await?;
  ```
- 使用 rate limiter 限制请求频率
- 部署在负载均衡器后（如 Envoy, Nginx）

---

## 场景 5：gRPC 反射滥用与信息泄露

### 前置条件
- gRPC 服务启用了反射 (gRPC Server Reflection)
- **重要：反射测试需从 Docker 网络内部连接 `auth9-core:50051`（明文 gRPC），而非 `localhost:50051`（nginx mTLS 代理）**
- `localhost:50051` 映射到 nginx mTLS 代理，使用 `-plaintext` 会被 SSL 握手拒绝（这是 nginx 正常行为，不是 Bug）

### 攻击目标
验证 gRPC 反射是否泄露敏感信息

### 攻击步骤

> **注意**：以下命令使用 `grpcurl-docker.sh` 从 Docker 网络内部连接 `auth9-core:50051`（明文 gRPC）。

1. 列出所有可用服务：
   ```bash
   .claude/skills/tools/grpcurl-docker.sh -plaintext -import-path /proto -proto auth9.proto auth9-core:50051 list
   ```

2. 获取服务方法定义：
   ```bash
   .claude/skills/tools/grpcurl-docker.sh -plaintext -import-path /proto -proto auth9.proto auth9-core:50051 describe auth9.TokenExchange
   ```

3. 获取完整的 proto 定义：
   ```bash
   .claude/skills/tools/grpcurl-docker.sh -plaintext -import-path /proto -proto auth9.proto auth9-core:50051 describe auth9.ExchangeTokenRequest
   ```

4. 发现未文档化的 API：
   ```bash
   .claude/skills/tools/grpcurl-docker.sh -plaintext -import-path /proto -proto auth9.proto auth9-core:50051 list | grep -i "admin\|internal\|debug"
   ```

### 预期安全行为（生产环境）
- 生产环境应**禁用** gRPC 反射
- 开发/测试环境可启用反射
- 如启用反射，应需要认证
- 不暴露内部/调试接口

### 验证方法
```bash
# 检查反射是否启用
grpcurl -plaintext localhost:50051 list

# 生产环境应返回错误：
# "server does not support the reflection API"

# 检查代码中是否启用反射
grep -r "tonic_reflection" auth9-core/src/
```

### 修复建议
- 生产环境禁用反射：
  ```rust
  // 开发环境
  #[cfg(debug_assertions)]
  let reflection_service = tonic_reflection::server::Builder::configure()
      .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
      .build()?;
  
  let mut builder = Server::builder();
  
  #[cfg(debug_assertions)]
  builder = builder.add_service(reflection_service);
  
  builder
      .add_service(auth9_service)
      .serve(addr)
      .await?;
  ```
- 使用环境变量控制：`ENABLE_GRPC_REFLECTION=false`
- 如必须启用，添加认证保护

---

## 自动化安全测试脚本

```bash
#!/bin/bash
# grpc-security-test.sh

set -e

GRPC_HOST="localhost:50051"

echo "=== Auth9 gRPC Security Test ==="

# 1. 测试未认证访问
echo "\n[1/5] Testing unauthenticated access..."
grpcurl -plaintext $GRPC_HOST list && echo "⚠️  Reflection enabled" || echo "✅  Reflection disabled"

# 2. 测试 TLS 配置
echo "\n[2/5] Testing TLS configuration..."
openssl s_client -connect $GRPC_HOST -tls1_2 2>&1 | grep -q "Protocol.*TLSv1.2" && echo "⚠️  TLS 1.2 enabled" || echo "✅  TLS 1.2 disabled"

# 3. 测试速率限制
echo "\n[3/5] Testing rate limiting..."
for i in {1..100}; do
    grpcurl -plaintext -d '{"page":1}' $GRPC_HOST auth9.Auth9Service/ListTenants > /dev/null 2>&1 &
done
wait
echo "✅  Rate limit test complete"

# 4. 测试大payload
echo "\n[4/5] Testing large payload..."
dd if=/dev/zero bs=1M count=10 | base64 > /tmp/large.json
grpcurl -plaintext -d @/tmp/large.json $GRPC_HOST auth9.Auth9Service/CreateUser && echo "⚠️  Large payload accepted" || echo "✅  Large payload rejected"

# 5. 测试元数据注入
echo "\n[5/5] Testing metadata injection..."
grpcurl -plaintext -H "X-Tenant-Id: 1' OR '1'='1" $GRPC_HOST list && echo "⚠️  SQL injection possible" || echo "✅  SQL injection blocked"

echo "\n=== Test Complete ==="
```

---

## 参考资料

- [gRPC Security Guide](https://grpc.io/docs/guides/security/)
- [OWASP gRPC Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/gRPC_Security_Cheat_Sheet.html)
- [CWE-306: Missing Authentication](https://cwe.mitre.org/data/definitions/306.html)
- [tonic Security Best Practices](https://github.com/hyperium/tonic/blob/master/examples/README.md#security)

---


---

## 标准化回归 Checklist（ASVS 5.0）

**矩阵ID**: M-ADV-02  
**适用控制**: V4.1,V4.2,V8.1,V13.2  
**关联任务**: Backlog #3, #20  
**建议回归频率**: 每次发布前 + 缺陷修复后必跑  
**场景总数**: 5

### 执行清单
- [ ] M-ADV-02-C01 | 控制: V4.1 | 任务: #3, #20 | 动作: 执行文档内相关攻击步骤并记录证据
- [ ] M-ADV-02-C02 | 控制: V4.2 | 任务: #3, #20 | 动作: 执行文档内相关攻击步骤并记录证据
- [ ] M-ADV-02-C03 | 控制: V8.1 | 任务: #3, #20 | 动作: 执行文档内相关攻击步骤并记录证据
- [ ] M-ADV-02-C04 | 控制: V13.2 | 任务: #3, #20 | 动作: 执行文档内相关攻击步骤并记录证据

### 回归记录表
| 检查项ID | 执行结果(pass/fail) | 风险等级 | 证据（请求/响应/日志/截图） | 备注 |
|---|---|---|---|---|
|  |  |  |  |  |

### 退出准则
1. 所有检查项执行完成，且高风险项无 `fail`。
2. 如存在 `fail`，必须附带漏洞单号、修复计划和复测结论。
3. 回归报告需同时记录矩阵ID与 Backlog 任务号，便于跨版本追溯。
