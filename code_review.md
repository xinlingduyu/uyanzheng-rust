# Nakamasa-Ichika 代码审查报告

> 审查日期: 2026-05-12
> 审查范围: 后端 Rust 代码（Nakamasa-Ichika, Nakamasa-utils, Nakamasa-proc）
> 审查重点: 安全问题、性能瓶颈、逻辑缺陷

---

## 目录

1. [严重安全问题](#1-严重安全问题)
2. [中高危安全问题](#2-中高危安全问题)
3. [性能瓶颈](#3-性能瓶颈)
4. [逻辑缺陷与代码异味](#4-逻辑缺陷与代码异味)
5. [总结与优先修复建议](#5-总结与优先修复建议)

---

## 1. 严重安全问题

### 1.1 [CRITICAL] PayPluginManager::get_plugin — 未定义行为导致 Use-After-Free / 双重释放

**文件:** `Nakamasa-Ichika/src/app/plugins/pay/manager.rs:41`

```rust
pub fn get_plugin(&self, plugin_type: &str) -> Result<Arc<dyn PayPlugin>, String> {
    let plugins = self.plugins.read()
        .map_err(|e| format!("获取读锁失败: {}", e))?;

    plugins.get(plugin_type)
        .map(|p| {
            // 这里使用unsafe是因为我们需要从Box获取引用
            // 在实际生产环境中应该使用更好的设计模式
            unsafe { Arc::from_raw(&**p as *const dyn PayPlugin) }
        })
        .ok_or_else(|| format!("插件 {} 不存在", plugin_type))
}
```

**问题分析:**

`Arc::from_raw` 要求传入的指针必须是之前由 `Arc::into_raw` 产生的有效 `Arc` 指针，其内存布局包含引用计数元数据。但这里传入的是 `Box<dyn PayPlugin>` 的解引用指针，`Box` 的内存布局**不包含引用计数**。这导致以下后果：

1. 每次调用 `get_plugin` 创建一个新的 `Arc`（虚拟引用计数=1），指向 `Box` 拥有的堆内存
2. 当这个 `Arc` 被 drop 时，会尝试释放内存并递减引用计数，但该内存实际属于 `Box`
3. `Box` 在 `HashMap` 被 drop 时也会释放同一块内存 → **双重释放**
4. 攻击者可以通过反复调用 `get_plugin` 创建多个 `Arc` 指向同一内存，触发多次释放 → 任意内存破坏

**影响:** 此代码在运行时会导致段错误、内存损坏、或潜在的远程代码执行。任何调用支付查询的接口都可能触发此路径。

**修复建议:** 将 `HashMap<String, Box<dyn PayPlugin>>` 改为 `HashMap<String, Arc<dyn PayPlugin>>`，或者返回 `&dyn PayPlugin`（需要调整生命周期设计）。

---

### 1.2 [CRITICAL] Redis 连接密码硬编码为 `***`

**文件:** `Nakamasa-Ichika/src/config/redis.rs:49-52` 和 `:94-97`

```rust
// connection_url()
if let Some(password) = self.password() {
    url.push(':***@');       // <-- 推入的是字面字符串 ":***@" 而不是实际密码
}

// decrypted_connection_url()
if let Some(password) = self.decrypted_password(secret) {
    url.push(':***@');       // <-- 同上
}
```

**问题分析:**

构造的 URL 形如 `redis://:***@127.0.0.1:6379/0`，密码被硬编码为 `***`。
`init_redis_pool()` (redis.rs:29) 使用这个 URL 连接 Redis:
```rust
let pool = Config::from_url(&connection_url)
```

如果 Redis 配置了密码认证，连接必定失败。如果 Redis 没有密码，这段代码恰好能工作（因为 `:***@` 被解析为密码 `***`，服务器忽略认证）。

**影响:** Redis 有密码时整个应用无法启动。这显然是从调试/脱敏代码遗留的 bug。

---

### 1.3 [CRITICAL] Admin 登录 Token IP 硬编码为 localhost

**文件:** `Nakamasa-Ichika/src/app/handlers/api/admin/login.rs:132`

```rust
let ip = "127.0.0.1";   // <-- 硬编码 IP

// JWT 中绑定 IP
.add_claim("ip", ip)
```

然后在 `admin_auth.rs:165` 严格验证 IP:
```rust
if claim_ip != ip_str {
    res.render(Json(ApiResponse::<()>::error(ERR_TOKEN_INVALID, -1)));
    return;
}
```

**问题分析:** Admin JWT 中的 IP 永远是 `127.0.0.1`，但实际请求来源 IP 可以是任意地址（通过 `X-Real-IP` / `X-Forwarded-For` 获取或 fallback 到 `127.0.0.1`）。

- 如果用户从远程访问管理后台 → `X-Real-IP` 获取到真实 IP → 与 `127.0.0.1` 不匹配 → 登录后每个请求都被拒绝（token 刚生成就失效）
- 如果 `get_client_ip` 获取不到任何 IP → fallback 到 `127.0.0.1` → 勉强匹配但失去了 IP 绑定的安全意义
- 实际上这导致 admin token 对所有远程请求**立即失效**

**影响:** 管理后台远程登录后无法进行任何操作，所有接口返回 Token 失效。同时注释掉的 `// let ip = get_client_ip(req);` 表明原来的正确实现被替换为硬编码值，疑似调试遗留。

---

### 1.4 [CRITICAL] SQL 注入风险 — BatchInserter 手动转义不完整

**文件:** `Nakamasa-Ichika/src/core/mysql.rs:315-317`

```rust
let escaped: Vec<String> = values.iter()
    .map(|v| format!("'{}'", v.replace('\'', "''")))
    .collect();
```

**问题分析:**

手动 SQL 转义仅替换了单引号，存在以下绕过方式：

- **反斜杠逃逸:** 在 MySQL 中 `\` 是转义字符，`\'` 会被解析为单引号。输入 `\'; DROP TABLE u_user; --` 会导致注入
- **宽字节注入:** 如果客户端使用 GBK 编码，`%bf%27` 可以绕过 `'` 过滤
- **二次注入:** 数据先被插入，后在其他查询中被取出使用

虽然代码注释说"仅用于受信任的数据"，但 `BatchInserter` 仍然是一个风险敞口。建议使用参数化查询（项目已有 `BatchInserterSafe`）。

---

## 2. 中高危安全问题

### 2.1 [HIGH] CORS 配置冲突 + 通配符 Origin

**文件:**
- `Nakamasa-Ichika/src/app/middleware/cors.rs:7-8` → 设置 `Access-Control-Allow-Origin: *`
- `Nakamasa-Ichika/src/core/server.rs:66-71` → 设置精确 Origin 列表 + `allow_credentials(true)`

**问题:** 两个 CORS 中间件都被应用（`cors.rs` 被加到路由中，`server.rs` 也创建了 Cors handler）。根据 CORS 规范，`Access-Control-Allow-Origin: *` 与 `Access-Control-Allow-Credentials: true` 不能共存，浏览器会拒绝携带凭证的跨域请求。

**影响:** 前端使用 fetch 带 cookie/authorization 头跨域访问 API 时，浏览器会报错。

### 2.2 [HIGH] 密码使用无盐 MD5 哈希

**文件:** `logon.rs:246-247`, `login.rs:103-105`, `reg.rs:293-294`

```rust
let password_hash_bytes = md5_hex(login_req.password.as_bytes());
let password_hash = md5_to_str(&password_hash_bytes);
```

**问题:** MD5 没有加盐，且是单一轮次。MD5 本身已被证明不安全（碰撞攻击），无盐 MD5 使得：
- 彩虹表攻击直接还原明文
- 相同密码产生相同哈希值
- 现代 GPU 可以每秒计算数十亿次 MD5

**影响:** 一旦数据库泄露，所有用户密码可被快速还原。作为认证/支付系统，这是严重的设计缺陷。

**建议:** 使用 `argon2` 或至少 `bcrypt`。如果保持 MD5 兼容性，至少加 **per-user salt**。

### 2.3 [HIGH] Admin JWT 签名密钥与密码 Salt 相同

**文件:** `login.rs:103-127`

```rust
let adm_pwd_salt = app_state.config().app().admin().keys();  // password verification salt
// ...
let jwt_builder = JwtBuilder::new(adm_pwd_salt, 3);          // JWT signing key (same value)
```

**问题:** 同一个字符串既用作密码验证的盐，又用作 JWT 签名的 HMAC 密钥。这意味着：
- 拿到 JWT 的人可以尝试推导签名密钥
- 密码验证和 Token 签名的安全边界被打破

**建议:** 使用独立的安全随机密钥用于 JWT 签名。

### 2.4 [HIGH] 多处 `unwrap()` 可能导致生产环境 panic

**文件:** `admin_auth.rs:103`, `login.rs:78`, `logon.rs:163`, `pay.rs:53`, 等

```rust
let app_state = depot.obtain::<Arc<AppState>>().unwrap();
```

**问题:** 如果 `AppState` 未正确注入（如测试、中间件顺序错误、路由变更），`unwrap()` 直接 panic 导致整个 HTTP 处理线程崩溃。在生产环境中，这意味着返回 500 错误或服务不可用。

虽然当前代码路径下 AppState 总会注入，但**任何代码变更**（路由调整、中间件重排）可能触底。应该用 `match` 或 `ok()` + 返回错误响应。

### 2.5 [HIGH] IP 验证函数逻辑错误 — 总是返回 true

**文件:** `admin_auth.rs:305-306`

```rust
fn is_valid_ip(ip: &str) -> bool {
    // ...
    // IPv4 最多 3 个点，IPv6 最多 7 个冒号
    dot_count <= 3 || colon_count <= 7
}
```

**问题:** 条件 `dot_count <= 3 || colon_count <= 7` 对于任何输入都成立（因为一个字符要么是点要么是冒号要么是其它，dot_count 和 colon_count 不可能同时超过其阈值）。例如 `"rm -rf /"` 中 dot_count=0 ≤ 3 → true。

**影响:** `get_client_ip` 中的 IP 验证形同虚设。任何伪造的 `X-Forwarded-For` 或 `X-Real-IP` 头都会被接受。

### 2.6 [MEDIUM] 支付宝 RSA 签名使用标准 Base64 而非 URL 安全的

**文件:** `ali.rs:72`

```rust
Ok(base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()))
```

**问题:** 支付宝签名规范要求使用 URL-safe Base64（或特定格式）。标准 Base64 含 `+` 和 `/` 字符，在 URL 参数传递中会被解码为空格或导致路径错误。实际查询字符串构建时 `urlencoding::encode` 会再编码一次，但标准 Base64 仍有可能在某些边缘情况下产生问题。

### 2.7 [MEDIUM] 支付宝订单查询使用 `block_in_place` + `block_on`

**文件:** `ali.rs:437-441`

```rust
let response = tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(async {
        http_client::post_form(&self.gateway_url, &form_data).await
    })
});
```

**问题:** 在异步上下文中同步阻塞一个线程来执行异步 HTTP 请求，浪费了异步的优势。此外，如果在 tokio 的当前线程运行时（`current_thread`）下调用，`block_in_place` 不会起作用，可能导致死锁。

**建议:** 将 `query` 方法改为异步函数（`async fn query`），或者使用 `reqwest` 的阻塞客户端在后台线程中执行。

---

## 3. 性能瓶颈

### 3.1 [HIGH] 重复的 CORS Handler — 每次请求执行两次

**文件:**
- `server.rs:74` → `service.hoop(cors)` （Salvo Cors 中间件）
- `routes.rs` 中全局路由挂载了 `cors.rs` 的 `cors` handler

每次 HTTP 请求经过两个 CORS handler，各自设置响应头（后一个覆盖前一个），浪费 CPU 和内存。

### 3.2 [MEDIUM] Token L1 缓存无 TTL 淘汰

**文件:** `user_auth.rs:446-474`

Token L1 缓存（内存 LRU）仅在发现密码不匹配时手动移除条目，没有 TTL 自动淘汰机制。这意味着：
- 已过期 Token 在 L1 缓存中继续存在，每次需要去 Redis 验证是否真过期（抵消了缓存的意义）
- 长期不活跃的 Token 占用 LRU 空间
- `is_valid` 方法检查了时间，但缓存条目本身不会被主动清理

### 3.3 [MEDIUM] 上传日志使用 `info!` 级别记录敏感信息

**文件:** `upload.rs:128-196`

```rust
tracing::info!("========== 开始处理上传图片请求 ==========");
tracing::info!("请求方法: {:?}", req.method());
// ... 遍历打印所有请求头
for (name, value) in req.headers() {
    tracing::info!("{}: {:?}", name, value);
}
tracing::info!("========== 表单字段 ==========");
for (name, values) in form_data.fields.iter_all() {
    tracing::info!("字段: {} = {:?}", name, values);
}
```

**问题:**
- `info!` 级别在生产中通常启用，会记录所有上传请求的 header（含 Authorization Token）
- 性能影响：每次上传需要 O(n) 遍历和格式化输出
- Token 泄露风险：日志文件若被读取，管理员 Token 直接泄露

**建议:** 使用 `debug!` 级别，并过滤 Authorization 等敏感 header。

### 3.4 [LOW] MySQL 默认端口错误

**文件:** `mysql.rs:20`

```rust
pub fn port(&self) -> u16 {
    self.port.unwrap_or(5432)  // 5432 是 PostgreSQL 的端口，MySQL 默认是 3306
}
```

配置中指定了正确的 3306 端口所以不会触发这个 fallback，但在配置缺失时会导致迷惑性错误。

### 3.5 [LOW] `once_cell::sync::OnceCell` 替代 `std::sync::OnceLock`

**文件:** `logon.rs:25` 使用了 `once_cell::sync::OnceCell`。Rust 1.70+ 标准库已有 `std::sync::OnceLock`，可以减少一个外部依赖。

### 3.6 [LOW] `batch_parse` 方法名与实现不符

**文件:** `json_optimize.rs:44-50`

```rust
pub fn batch_parse<'a, I>(json_strings: I) -> Vec<Result<Value, serde_json::Error>>
where
    I: Iterator<Item = &'a str>,
{
    json_strings.map(serde_json::from_str).collect()
}
```

这不是"批量解析"（并行解析），只是单线程的 map + collect。将其标注为性能优化可能引起误解。

---

## 4. 逻辑缺陷与代码异味

### 4.1 未知加密类型静默降级为 AES

**文件:** `encryption/mod.rs:78`

```rust
let enc_type = EncryptionType::from_str(enc_type).unwrap_or(EncryptionType::Aes);
```

如果配置中指定了不存在的加密类型（如 `"aes128"` 或拼写错误），系统会静默使用 AES 而不是报错。客户端和服务端的加解密方式不匹配，导致所有请求解密失败，排查困难。

### 4.2 Admin 登录日志记录硬编码 IP + 数据竞争风险

**文件:** `login.rs:150-164`

```rust
tokio::spawn(async move {
    let _ = sqlx::query(
        "INSERT INTO u_logs (ug, uid, type, state, time, ip, appid) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("adm").bind(admin_id).bind("login").bind(true).bind(now)
    .bind("127.0.0.1")          // <-- 硬编码 IP
    .bind(Option::<u64>::None)
    .execute(db.as_ref().expect("DB not initialized"))  // <-- 可能 panic
    .await;
});
```

问题：
- 日志记录的 IP 永远是 `127.0.0.1`，审计失效
- `db.as_ref().expect(...)` 在 `tokio::spawn` 中 panic 会导致 tokio 任务崩溃（静默错误）

### 4.3 注册账号重复检查使用 OR 条件无法利用组合索引

**文件:** `reg.rs:185-187`

```sql
SELECT id FROM u_user WHERE (phone = ? OR email = ? OR acctno = ?) AND appid = ?
```

MySQL 对 `OR` 条件通常无法有效使用复合索引。如果有索引 `(appid, phone)`，`email` 和 `acctno` 的条件无法利用。建议拆分为三条 UNION 查询或用三个独立查询+`if`。

### 4.4 二次序列化冗余

**文件:** `user_auth.rs:296`（通过 `sn_list_json.as_str()` 获取 JSON 字符串后再与 `sn_list_json.map(|v| v.to_string())` 双重处理）

```rust
let sn_list = sn_list_json.as_ref().and_then(|v| v.as_str()).map(|s| s.to_string())
    .or_else(|| sn_list_json.map(|v| v.to_string()));
```

如果 `as_str()` 成功（JSON 值是字符串），`or_else` 不会被调用。但如果 JSON 值是对象/数组，它会先尝试 `as_str()(None)` 然后 `or_else` 对 Value 做 `to_string()`。逻辑正确但写法晦涩且多一次可能的克隆。

### 4.5 多处 Debug 级别日志使用 `tracing::info!`

除了上述的上传日志，多个 handler 中有大量 `info!` 级别的调试日志（如注册流程中的 `tracing::warn!("[注册调试] ...")`）。这些应该使用 `debug!`。

### 4.6 BatchInserter.add_row 静默忽略列数不匹配

**文件:** `mysql.rs:229-232`

```rust
pub fn add_row(&mut self, values: &[&str]) {
    if values.len() != self.columns.len() {
        return;  // 静默返回，无任何日志
    }
```

建议记录 warning 日志以便排查问题。

### 4.7 admin_cache 查询使用 tuple 而非命名结构体

**文件:** `admin_auth.rs:172-174`

```rust
let admin_result = sqlx::query_as::<_, (u64, String, String, Option<String>, String, Option<String>, Option<String>, bool, Option<u64>)>(...)
```

16 个字段的元组查询，可读性极差，改字段顺序或新增字段容易引发难以追踪的 bug。建议使用 `#[derive(FromRow)]` 结构体。

---

## 5. 总结与优先修复建议

### 立即修复（优先级 P0 — 可能导致崩溃或数据泄露）

| # | 问题 | 影响 | 建议方案 |
|---|------|------|---------|
| 1.1 | `Arc::from_raw` UB (#manager.rs:41) | 段错误/任意内存破坏 | 改为 `Arc` 存储或返回引用 |
| 1.2 | Redis 密码硬编码 `***` (#redis.rs:50,95) | Redis 有密码时连接失败 | 使用实际密码生成 URL |
| 1.3 | Admin 登录 IP 硬编码 (#login.rs:132) | 远程 admin 所有请求被拒 | 使用 `get_client_ip(req)` |
| 1.4 | BatchInserter SQL 注入风险 (#mysql.rs:315) | 数据泄露/篡改 | 统一用参数化查询 |

### 尽快修复（优先级 P1 — 安全漏洞）

| # | 问题 | 影响 | 建议方案 |
|---|------|------|---------|
| 2.1 | CORS 通配符冲突 | 跨域请求失败 | 只用一处 CORS 配置 |
| 2.2 | 密码无盐 MD5 | 数据库泄露可还原密码 | 加盐或换 Argon2 |
| 2.3 | JWT 密钥与密码盐相同 | 密钥可被推导 | 独立随机密钥 |
| 2.4 | `unwrap()` 在生产路径 | 意外 panic | 改用 match + 错误响应 |
| 2.5 | `is_valid_ip` 总是 true | IP 伪造绕过 | 修正逻辑条件 |
| 2.7 | `block_in_place` + `block_on` | 可能死锁 | 改为 async fn |

### 后续优化（优先级 P2）

| # | 问题 | 受益 |
|---|------|------|
| 3.1 | 重复 CORS handler | 减少请求处理开销 |
| 3.2 | Token L1 缓存无 TTL | 提升缓存效率 |
| 3.3 | 上传日志 info! 级别 | 避免 Token 泄露 |
| 4.1 | 加密类型静默降级 | 减少排查成本 |
| 4.4 | 注册 OR 查询索引失效 | 提升查询性能 |

---

## 附录：审查统计

| 指标 | 值 |
|------|-----|
| 审查 Rust 文件数 | ~40+ |
| 总代码行数(项目) | ~15,000+ |
| 严重安全问题 | 4 |
| 中高风险问题 | 7 |
| 性能瓶颈 | 5 |
| 逻辑/代码异味 | 7 |

---

*本报告基于静态代码审查，未运行测试或运行时分析。建议修复后补充集成测试和模糊测试。*
