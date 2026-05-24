# 安全审计报告 — Nakamasa-Ichika

**审计日期**: 2026-05-24
**审计范围**: 项目目录 `/data/data/com.termux/files/home/rust/web/Nakamasa-Ichika/src/`
**审计方法**: 静态代码分析（手动审查）

---

## 1. SQL 注入风险

### 1.1 [高危] QuickJS 云函数运行时 — 原始 SQL 执行（无参数绑定）

**文件**: `src/core/quickjs_runtime.rs`
**函数**: `execute_db_query()` (第 335-387 行), `__dbQuery` (第 653-667 行)

**问题描述**: 云函数 JS 代码中调用 `__dbQuery(rawSqlString)` 时，SQL 直接拼接后通过 `sqlx::query(&sql)` 执行，**未使用参数绑定**。仅依赖 `validate_sql_security()`（第 273-321 行）的黑名单校验（拦截 `--`, `;`, `UNION`, `LOAD_FILE` 等关键字）。

黑名单校验**本质上不完整**——攻击者可以通过替代编码、MySQL 非标准语法等方式绕过。例如 `UNION` 被拦截，但 `UNION ALL` 或 `UNION DISTINCT` 可能绕过。

**风险等级**: 高危

```rust
// 第 654-667 行 — 用户 JS 传入的 SQL 直接执行
let db_query_fn = Function::new(
    ctx.clone(),
    |_ctx: Ctx, sql: String| -> Result<String, rquickjs::Error> {
        let result = execute_db_query(&sql);  // 直接传用户 SQL
        ...
    },
)...
// 第 349-353 行 — 无参数绑定
rt.block_on(async {
    sqlx::query(&sql)  // 没有 bind() 调用
        .fetch_all(&db)
        ...
})
```

### 1.2 [中危] QuickJS 云函数运行时 — 自定义转义函数不完整

**文件**: `src/core/quickjs_runtime.rs`
**函数**: `escape_sql_value()` (第 323-333 行)

**问题描述**: 该自定义转义函数仅处理了有限的字符（`'`, `"`, `\`, 换行等），未处理以下绕过方式：
- 多字节字符集绕过（如 GBK 编码下的宽字节注入）
- Unicode 规范化攻击
- MySQL 特定转义序列

JS 代码可以使用 `__escapeValue(userInput)` 拼接 SQL 字符串，然后通过 `__dbQuery()` 执行，此路径提供了伪参数化查询的错觉，但实际不安全。

**风险等级**: 中危

```rust
fn escape_sql_value(value: &str) -> String {
    value
        .replace("\\\\", "\\\\\\\\")
        .replace("'", "\\'")
        .replace("\"", "\\\"")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\x00", "\\0")
        .replace("\x1a", "\\Z")
}
```

### 1.3 [中危] QuickJS SELECT LIMIT 追加 — 字符串拼接

**文件**: `src/core/quickjs_runtime.rs`
**函数**: `normalize_select_limit()` (第 89-118 行)

**问题描述**: 第 118 行通过 `format!()` 将 LIMIT 子句追加到用户提供的 SQL 字符串末尾：

```rust
Ok(format!("{} LIMIT {}", sql, QUICKJS_DB_SELECT_DEFAULT_LIMIT))
```

虽然用户 SQL 先经过 `validate_sql_security()` 校验，但硬编码的限制值不会引入注入 — 风险在于这个 SQL 后续仍然无参数绑定的方式执行。

**风险等级**: 中危（与 1.1 关联）

### 1.4 [安全] format!() 构建 UPDATE SET 子句

在以下文件中，`format!()` 用于构建 `UPDATE table SET col1=?, col2=? WHERE id = ?` 格式的 SQL。SET 子句的列名来自内部代码（非用户输入），且值使用 `?` 占位符 + `bind()` 参数化。**安全，无注入风险。**

涉及的文件（均为同样模式）：
- `src/app/handlers/api/admin/user/user.rs:881`
- `src/app/handlers/api/admin/adm/adm_list.rs:291`
- `src/app/handlers/api/admin/finance/fen_order.rs:442`
- `src/app/handlers/api/admin/cdk/cdk_user.rs:558`
- `src/app/handlers/api/admin/cdk/cdk_kami.rs:766`
- `src/app/handlers/api/admin/app/app.rs:839`
- `src/app/handlers/api/admin/finance/fen_event.rs:722`
- `src/app/handlers/api/admin/blocklist/blocklist.rs:662`
- `src/app/handlers/api/admin/logs/logs.rs:923`

### 1.5 [安全] WHERE 子句构建（bind 参数化）

`src/app/handlers/api/admin/finance/fen_event.rs` 第 157-174 行和 `src/app/handlers/api/admin/system/encryption.rs` 第 303-351 行构建动态 WHERE 子句时虽然用了 `format!()` 拼接 SQL 骨架，但条件值使用 `?` 占位符 + 循环 `bind()`。**安全。**

---

## 2. 危险的 unwrap()/expect()

### 2.1 [中危] `get_db()` / `get_redis()` — 生产路径 expect

**文件**: `src/core/app_state.rs:427-439`

**问题描述**: 如果 `get_db()` 或 `get_redis()` 在数据库/Redis 初始化完成之前被调用，会 panic：

```rust
pub fn get_db(&self) -> &MySqlPool {
    self.db.as_ref().expect("Database not initialized")
}
pub fn get_redis(&self) -> &RedisPool {
    self.redis_pool.as_ref().expect("Redis not initialized")
}
```

这些方法被大量 handler 调用，虽然不是常见的运行时路径，但在启动时序异常时仍可能触发 panic。

**风险等级**: 中危

### 2.2 [中危] `admin_auth.rs` — 时间戳 unwrap

**文件**: `src/app/middleware/admin_auth.rs:22-27`

**问题描述**: `SystemTime::now().duration_since(UNIX_EPOCH).unwrap()` — 如果系统时间早于 UNIX EPOCH（1970-01-01），会 panic。虽然现代系统几乎不可能，但在嵌入式或 Docker 容器启动时偶有发生。

**风险等级**: 中危

### 2.3 [低危] `reg.rs` — Option unwrap

**文件**: `src/app/handlers/api/user/auth/reg.rs:281,294`

**问题描述**: `reg_req.code.unwrap()` 在验证码可能为 None 时直接 unwrap。虽然在调用前已有校验，但代码结构脆弱。

**风险等级**: 低危

### 2.4 [低危] 支付插件配置 Option unwrap

**多个文件**: `src/app/plugins/pay/*.rs`, `src/app/plugins/sms/*.rs`

在这些插件的 `create()` / `verify_notify()` 方法中，大量使用 `self.xxx.as_ref().unwrap()`（如 `ali.rs:207`, `qq.rs:542`, `wx.rs:459`, `jie.rs:160`）。虽然之前有 `is_none()` 检查，但代码重复且脆弱。

**风险等级**: 低危

### 2.5 [安全] LazyLock 初始化 unwrap

`src/app/utils/validator.rs:5-21`, `src/core/quickjs_runtime.rs:78-83`, `src/core/regex_cache.rs:13-35` 中的 `LazyLock` + `unwrap()`/`expect()` 在静态初始化时执行，保证只运行一次。**可接受（按任务说明）**。

### 2.6 [安全] `depot.obtain::<Arc<AppState>>()` 的 unwrap

按任务说明已确认可接受。

---

## 3. 输入验证缺失

### 3.1 [低危] 安装接口 — 安装后是否仍可调用 POST install？

**文件**: `src/app/routes.rs:143-147`

在 `build_production_routes()` 中，安装路由被替换为 `install_check_routes()`，仅暴露 `/api/install/check` 和 `/api/install/checkapi`（GET 方法）。**安全。**

### 3.2 [安全] 大多数 handler 已使用验证器

用户和 admin handler 普遍使用 `Validator` 工具类对输入进行校验（如 `wordnum`、`reg` 正则匹配）。云函数 handler（`cloud_function.rs:114`）使用正则 `[a-zA-Z][a-zA-Z\\d]{2,64}` 校验函数名。上传 handler 校验 MIME 类型和文件魔数。**整体良好。**

---

## 4. 越权风险

### 4.1 [中危] `/api/admin/system/dictAll` — 缺少认证中间件

**文件**: `src/app/handlers/api/admin.rs:116`

```rust
.push(Router::with_path("/system/dictAll").get(system::dict::dict_all))
```

与周围所有 admin 路由不同，`/system/dictAll` **没有挂载 `AdminAuth` 中间件**，任何人都可以访问此端点。虽然当前实现仅返回硬编码的字典数据（数据状态、性别等），但未来若修改该 handler 添加敏感数据，将形成信息泄露漏洞。

**风险等级**: 中危

### 4.2 [中危] `admin_token_verify_enabled()` 配置可全局绕过认证

**文件**: `src/app/middleware/admin_auth.rs:119-122`

```rust
if !security_conf.admin_token_verify_enabled() {
    ctrl.call_next(req, depot, res).await;
    return;
}
```

当配置项 `admin_token_verify_enabled` 设为 false 时，所有 admin 路由的 `AdminAuth` 中间件直接放行。这是一个管理员配置层面的单点失效风险，但属于功能设计。

**风险等级**: 中危

### 4.3 [安全] Admin 路由认证覆盖

审计确认所有 admin 管理接口（`/user`, `/app`, `/order`, `/goods`, `/logs`, `/cdk*`, `/agent*`, `/ver*`, `/encryption*`, `/functions*`, `/blocklist*`, `/admList*`, `/system/*` 等）均正确挂载了 `AdminAuth` 中间件。登录接口 `/login` 和 token 验证接口 `/admin/verify` 按设计不需要认证。**良好。**

### 4.4 [安全] User 路由认证覆盖

用户路由的认证边界清晰：
- 不需要 token：`logon`, `kamiLogin`, `reg`, `resetPwd`, `getCode`, `goods`, `pay`, `ini`, `wxlogon*`, `qq*` 等
- 需要 token：`info`, `order`, `vip`, `modify*`, `set*`, `bindUdid`, `cloudFunction`, `upload`, `message*` 等
- `heartbeat` 和 `logout` 虽然未挂载 `UserAuth`，但内部自行验证 token。**良好。**

---

## 5. 命令注入

### 5.1 [安全] `std::process::Command` 仅用于 TUI 仪表盘

**文件**: `src/core/tui_dashboard.rs:191-193`

```rust
std::process::Command::new("top")
    .args(["-b", "-n", "2", "-d", "1"])
```

所有参数均为硬编码常量，**无用户输入注入风险。无其他 `std::process::Command` 或 `system()` 调用。**

**风险等级**: 安全

---

## 6. 路径遍历

### 6.1 [安全] 静态文件访问 — 完整路径验证

**文件**: `src/app/handlers/static_files.rs:67-99` (`validate_static_path`)
**文件**: `src/app/handlers/static_files.rs:360-398` (`validate_upload_path`)

两个验证函数均实现了多层防护：
1. 检查空字节注入（`\0`）
2. 检查 `..` 路径遍历
3. 路径规范化（`canonicalize` 或 `components()` 检查）
4. 验证最终路径在基础目录内
5. 拒绝隐藏文件（以 `.` 开头）

**良好实现。**

### 6.2 [安全] 用户上传 — 文件名清理

**文件**: `src/app/handlers/api/user/misc/upload.rs:71-82`

```rust
fn sanitize_filename(filename: &str) -> String {
    Path::new(filename).file_name()...  // 只取最后文件名部分
}
fn contains_path_traversal(filename: &str) -> bool {
    filename.contains("..") || filename.contains('/')
}
```

使用 `Path::file_name()` 安全提取最终文件名，并检查 `..` 和 `/`。目录路径由 `appid` 和 `uid`（均为 u64 类型，从中间件获取的可靠数据）构建。

**风险等级**: 安全

---

## 7. 支付签名缺陷

### 7.1 [中危] 微信支付 — MD5 签名算法

**文件**: `src/app/plugins/pay/wx.rs:60-64, 67-72`

```rust
fn sign_md5(&self, data: &str) -> String {
    let bytes = md5_hex(data.as_bytes());
    md5_to_str(&bytes).to_string()
}
fn sign(&self, params: &BTreeMap<String, String>) -> String {
    let sign_string = Self::build_sign_string(params);
    let wx_key = self.wx_key.as_ref().unwrap_or(&empty_key);
    self.sign_md5(&format!("{}&key={}", sign_string, wx_key))
}
```

MD5 已被学术界和工业界认定为不安全（可构造碰撞）。虽然支付平台的 MD5 签名（带密钥的 HMAC-like 构造）比原始 MD5 更难攻击，但理论上碰撞攻击仍可构造两个不同 payload 的相同签名。

微信支付官方 API 文档仍使用 MD5，所以这更多是协议层面的限制。

**风险等级**: 中危（协议限制，但 MD5 本身不安全）

### 7.2 [中危] QQ 钱包 — MD5 签名算法

**文件**: `src/app/plugins/pay/qq.rs:76-80, 83-88`

与微信支付完全相同的 MD5 签名模式。同样受限于上游平台协议。

**风险等级**: 中危

### 7.3 [中危] 皆网支付 — MD5 签名算法

**文件**: `src/app/plugins/pay/jie.rs:41-56`

```rust
fn sign(&self, data: &BTreeMap<String, String>) -> String {
    let sign_str: String = data.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>().join("&");
    let decoded = urlencoding::decode(&sign_str).unwrap_or_default();
    let access_key = self.access_key.as_ref().unwrap_or(&empty_key);
    md5_concat_2(&decoded, access_key)
}
```

同样是 MD5 签名。此外，签名前对拼接字符串进行 URL 解码（`urlencoding::decode`），这可能引入意外的 payload 转换（如 `%3D` → `=` 改变结构）。

**风险等级**: 中危

### 7.4 [安全] 支付宝 — RSA2 (SHA256withRSA) 签名

**文件**: `src/app/plugins/pay/ali.rs:32-101`

```rust
fn sign_rsa2(&self, data: &str) -> Result<String, String> {
    let signing_key = SigningKey::<Sha256>::new(private_key);
    let signature = signing_key.sign_with_rng(&mut rand::thread_rng(), data.as_bytes());
    Ok(base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()))
}
```

使用 SHA256 摘要的 RSA 签名，符合支付宝官方规范。正确实现了 `RsaPrivateKey::from_pkcs8_pem` 和 `from_pkcs1_pem` 双格式兼容。验签也正确。

**风险等级**: 安全

### 7.5 [安全] PayPal 验证

PayPal 插件的通知验证遵循 PayPal 官方 Webhook 验证流程（验证回调来源 IP 和传输层安全），**实现正确**。

---

## 8. 其他发现

### 8.1 [低危] 云函数 SQL 注入防护 — 黑名单绕过潜力

**文件**: `src/core/quickjs_runtime.rs:273-321`

`validate_sql_security()` 拦截的黑名单关键词包括：`--`, `/*`, `;`, `UNION`, `INTO OUTFILE`, `LOAD_FILE`, `BENCHMARK`, `SLEEP`, `WAITFOR` 等。

绕过风险包括：
- MySQL 注释符 `#` 未拦截
- `UNION ALL` 中的空格变体
- 使用 `UNION DISTINCT` 替代 `UNION`
- 系统信息函数（`@@version`, `DATABASE()`）等未拦截
- 盲注（布尔/时间）的函数未被全面拦截

但函数名白名单检查（第 246-270 行）限制了只能访问 `u_xxx` 格式的表，且黑名单（第 210-243 行）阻止了系统表。

**风险等级**: 低危（有防护但不够完善）

### 8.2 [信息] 管理后台的 `AdminAuth::skip_verify()` 模式

**文件**: `src/app/handlers/api/admin.rs:46`
**路由**: `/api/admin/admin/verify`

此端点使用 `AdminAuth::new().skip_verify()` 模式，中间件仍会解析 token（若提供），但允许无 token 访问。这是设计意图 — 前端需要调用此接口验证 token 有效性。

---

## 总结

| 类别 | 高危 | 中危 | 低危 | 安全/信息 |
|------|------|------|------|-----------|
| SQL 注入 | 1 | 2 | 2 | 2 |
| unwrap/expect | 0 | 2 | 2 | 3 |
| 输入验证 | 0 | 0 | 1 | 3 |
| 越权 | 0 | 2 | 0 | 2 |
| 命令注入 | 0 | 0 | 0 | 1 |
| 路径遍历 | 0 | 0 | 0 | 2 |
| 签名安全 | 0 | 3 | 0 | 3 |
| 其他 | 0 | 0 | 1 | 1 |
| **总计** | **1** | **9** | **6** | **17** |

### 最需要优先修复的问题

1. **[高危]** `quickjs_runtime.rs:execute_db_query()` — 云函数中无参数绑定的 SQL 执行。建议：移除 `__dbQuery` 原始 SQL 接口，强制使用 `__dbQueryWithParams` 参数化查询。

2. **[中危]** `quickjs_runtime.rs:escape_sql_value()` — 不完整的自定义 SQL 转义。建议：移除自定义转义，禁止在云函数中通过字符串拼接构造 SQL。

3. **[中危]** `admin_auth.rs:25` — 系统时间 unwrap。建议：使用 `unwrap_or(0)` 或错误处理替代直接 unwrap。

4. **[中危]** `app_state.rs:428,438` — `get_db()`/`get_redis()` 的 expect。建议：返回 `Result<&Pool, Error>` 或使用 `try_get_db()` 模式。

5. **[中危]** 微信/QQ/皆网支付 MD5 签名。建议：虽然受上游平台协议限制，但记录此风险并监控上游是否升级到 HMAC-SHA256。

6. **[中危]** `/api/admin/system/dictAll` 缺少认证。建议：挂载 `AdminAuth` 中间件或确认 handler 永不返回敏感数据。

---

## 9. 错误处理审计（2026-05-24 新增）

### 9.1 [高危] `depot.get::<AppInfo>` 的 unwrap — 中间件中可能 panic

**文件**: `src/app/middleware/user_auth.rs:180`

```rust
let app_info = depot.get::<AppInfo>("app_info").unwrap();
```

**问题描述**: 此代码位于 `UserAuth` 中间件中（数据校验路径）。与项目中所有其他 `depot.get::<AppInfo>()` 调用（50+ 处）不同——它们全部使用 `match` 表达式正确处理 `Err` 分支并返回错误响应——此处直接 `.unwrap()`。如果 depot 中缺少 `app_info`（例如路由配置错误或 AppContext 中间件未正常执行），会导致 **panic 并崩溃整个 worker 线程**。这直接影响所有需要数据校验的用户 API。

**风险等级**: 高危

### 9.2 [高危] 支付通知事务 — `let _ = sqlx::query(...)` 静默吞没数据库错误

**文件**: `src/app/handlers/api/index/notify.rs:195-202, 206-216`

```rust
// 第195-202行 — agent UPDATE 错误被静默忽略
let _ = sqlx::query(
    "UPDATE u_agent SET pay_divide = GREATEST(pay_divide, ?), km_discount = LEAST(km_discount, ?) WHERE id = ?"
)
.bind(pay_divide.unwrap_or(0))
.bind(km_discount.unwrap_or(100))
.bind(agent_id)
.execute(&mut *tx)
.await;  // 忽略错误

// 第206-216行 — agent INSERT 错误被静默忽略
let _ = sqlx::query(
    "INSERT INTO u_agent (aggid, uid, pay_divide, km_discount, time, appid) VALUES (?, ?, ?, ?, ?, ?)"
)
...
.execute(&mut *tx)
.await;  // 忽略错误
```

**问题描述**: 在支付通知的 `update_order()` 事务中，当处理代理（agent）订单时，`UPDATE u_agent` 和 `INSERT INTO u_agent` 的执行结果被 `let _ = ` 静默忽略。如果这些写入失败，事务**不会回滚**而继续提交，导致订单状态已更新（state=2）但代理数据丢失，**资金数据不一致**。

**风险等级**: 高危

### 9.3 [中危] 支付通知事务 — 多处 `let _ = tx.rollback()/commit()` 静默吞没

**文件**: `src/app/handlers/api/index/notify.rs:93, 100, 106, 122, 152, 165, 219, 232`

**问题描述**: 共 **8 处** `let _ = tx.rollback().await` 或 `let _ = tx.commit().await`。rollback/commit 本身也可能失败（如连接断开、事务超时），但所有失败都被静默忽略。其中行 100（幂等分支）在应该提交事务时忽略 commit 错误，可能导致连接泄漏。

**风险等级**: 中危

### 9.4 [中危] 支付通知事务 — agent 分支缺少 Err 分支记录

**文件**: `src/app/handlers/api/index/notify.rs:179-221`

**问题描述**: 在 agent 订单处理中，`sqlx::query_as(...).fetch_optional()` 查询代理组和代理信息时，`Err` 分支使用 `_` 完全忽略，仅处理 `Ok(Some(...))` 和 `else`（空结果回滚）。如果数据库查询返回 Err，代码直接落入 `else` 分支回滚，但**无任何错误日志**，排查困难。

**风险等级**: 中危

### 9.5 [中危] `serde_json::to_string` unwrap 序列化 — 理论上安全但风格脆弱

**文件**: 
- `src/app/handlers/api/user/oauth/qq_login_sdk.rs:283, 338, 356`
- `src/app/handlers/api/user/oauth/wx_login_sdk.rs:283, 338, 356`
- `src/app/handlers/api/user/auth/logon.rs:550, 972`

**问题描述**: `serde_json::to_string(&new_sn_list).unwrap()` 和 `serde_json::to_string(&new_list).unwrap()` 在 HTTP handler 中直接 unwrap。虽然 `serde_json::to_string` 对 `serde_json::Value` 几乎不会失败，但若后续代码修改了数据类型（如使用自定义结构体）将静默 panic。

**风险等级**: 中危

### 9.6 [中危] `chrono::DateTime::from_timestamp` unwrap — 负时间戳 panic

**文件**: 
- `src/app/handlers/api/user/oauth/qq_login_sdk.rs:383, 602`
- `src/app/handlers/api/user/oauth/wx_login_sdk.rs:383, 595`

```rust
let dt = chrono::DateTime::<Utc>::from_timestamp(v, 0).unwrap();
```

**问题描述**: `v` 来自数据库用户 VIP 时间戳字段。如果值异常（如 0 或负数），`from_timestamp` 返回 `None`，`.unwrap()` 导致 panic。虽然行 382 有 `if v > 0` 检查，但行 395-602 在注册新用户路径上无此检查。

**风险等级**: 中危

### 9.7 [中危] 卡密批量生成 — 事务提交部分成功

**文件**: 
- `src/app/handlers/api/admin/cdk/cdk_kami.rs:428-504`
- `src/app/handlers/api/admin/cdk/cdk_user.rs:414-484`

**问题描述**: 批量生成卡密时开启事务，但循环内单条插入失败时**继续循环**而非回滚。最终事务提交时，部分卡密插入成功、部分失败。这破坏了事务的原子性语义，可能导致部分生成、部分丢失的不一致状态。虽然错误被 tracing 记录，但调用者收到的是`成功计数`，可能误以为全部成功。

**风险等级**: 中危

### 9.8 [中危] `let _ = sqlx::query(...)` 日志写入静默失败

**涉及文件**（多处, 部分列举）:
- `src/app/handlers/api/user/auth/logon.rs:456, 839`
- `src/app/handlers/api/user/auth/reg.rs:461`
- `src/app/handlers/api/user/auth/modify_pwd.rs:122`
- `src/app/handlers/api/user/auth/reset_pwd.rs:138`
- `src/app/handlers/api/user/profile/modify_name.rs:91`
- `src/app/handlers/api/user/profile/set_acctno.rs:119`
- `src/app/handlers/api/user/profile/re_phone.rs:127`
- `src/app/handlers/api/user/trade/kami_topup.rs:240`

**问题描述**: 大量日志写入（`INSERT INTO u_logs`）使用 `let _ = sqlx::query(...)` 静默忽略数据库错误。虽然日志写入失败不应阻塞主业务，但操作审计丢失可能影响安全事件追溯。

**风险等级**: 中危

### 9.9 [中危] `let _ = sqlx::query(...)` 设备绑定（sn_list）更新静默失败

**文件**: 
- `src/app/handlers/api/user/auth/logon.rs:517, 549, 557, 926, 937, 971, 978`
- `src/app/handlers/api/user/auth/reg.rs:364, 376`

**问题描述**: 设备绑定列表（sn_list）更新操作使用 `let _ = sqlx::query(...)` 静默忽略错误。这意味着设备绑定可能失败而不通知用户，削弱设备验证的安全性。

**风险等级**: 中危

### 9.10 [低危] 支付通知事务 — `Err(_)` 分支无日志

**文件**: `src/app/handlers/api/index/notify.rs:78, 92`

**问题描述**: 第 78 行 `Err(_) => return false` 和 92 行 `Err(_) => { let _ = tx.rollback().await; return false; }` 在数据库操作失败时仅返回 false，**无任何 tracing::error 或 tracing::warn 日志**。生产排障困难。

**风险等级**: 低危

### 9.11 [低危] MCP 服务器 — `SESSIONS.write().unwrap_or_else()`

**文件**: `src/app/handlers/mcp/server.rs:654`

```rust
let mut sessions = SESSIONS.write().unwrap_or_else(|e| e.into_inner());
```

**问题描述**: 使用 `RwLock::write().unwrap_or_else(|e| e.into_inner())` 在锁被毒化（poisoned）时强制获取。若持有锁的线程 panic，锁被毒化，`into_inner()` 可能返回不一致的数据。与项目中其他使用 `lock().unwrap()` 的地方类似，但此为 Websocket/SSE 连接路径，毒化锁影响范围更大。

**风险等级**: 低危

### 9.12 [低危] `reg.rs` 中 code unwrap

**文件**: `src/app/handlers/api/user/auth/reg.rs:281, 294`

**问题描述**: `reg_req.code.unwrap()` 虽然之前有 `is_none()` 检查，但第 294 行的二次 unwrap（日志打印）若 `code` 在两次检查间被修改（Rust 所有权规则下极难发生）会 panic。

**风险等级**: 低危

### 9.13 [低危] 支付插件配置 unwrap — 在 `is_none()` 检查后使用

**文件**（部分列举）:
- `src/app/plugins/pay/ali.rs:47, 80, 131, 207, 424`（private_key, alipay_public_key, appid）
- `src/app/plugins/pay/qq.rs:124, 127, 209, 212, 295, 298, 542, 545`（qq_appid, qq_mchid）
- `src/app/plugins/pay/paypal.rs:81, 82`（client_id, client_secret）
- `src/app/plugins/pay/jie.rs:160`（pid）
- `src/app/plugins/sms/ali.rs:156-158`（access_key_id, sign_name, template_code）
- `src/app/plugins/sms/tencent.rs:143-146`（appid, appkey, sname, mid）
- `src/app/plugins/sms/jie.rs:113-114`（access_key, mid）

**问题描述**: 所有 .unwrap() 调用之前都有 `is_none()` 检查或 `create()` 时已验证过配置完整性。但如果后期修改 `create()` 或添加新方法时忘记前置检查，将直接 panic。

**风险等级**: 低危

### 9.14 [信息] 通知回调返回格式正确

**文件**: `src/app/handlers/api/index/notify.rs:331, 340, 354, 362, 375, 398, 407, 417, 423, 425`

**审计结果**: 所有 10 处响应均使用 `res.render(Text::Plain("success"))` 或 `res.render(Text::Plain("fail"))` 返回纯文本格式，符合支付平台通知回调协议要求，**无 JSON 响应问题**。

**风险等级**: 安全

### 9.15 [信息] `depot.obtain::<Arc<AppState>>()` 审计

**审计结果**: 全部 50+ 处使用均正确使用 `match` 表达式处理 `Err` 分支，返回 HTTP 错误响应。**统一合规。**（已确认覆盖 `user_auth.rs`, `admin_auth.rs`, `app_context.rs`, `notify.rs`, `return_.rs`, `static_files.rs`, `mcp/server.rs` 及所有 handler 文件）

**风险等级**: 安全

### 9.16 [信息] `depot.get::<AppInfo>` / `depot.get::<UserInfo>` 审计

**审计结果**: 除 `user_auth.rs:180`（见 9.1）外，其余 50+ 处 `depot.get::<AppInfo>("app_info")`、`depot.get::<UserInfo>("user_info")` 和 `depot.get::<u64>("admin_id")` 均正确使用 `match` 处理 `Err`，返回错误响应或默认值。

**风险等级**: 安全（除 9.1）

---

## 错误处理审计汇总

| 类别 | 高危 | 中危 | 低危 | 安全/信息 |
|------|------|------|------|-----------|
| 未处理的 Result (let _) | 1 | 2 | 0 | 0 |
| 静默失败 | 0 | 2 | 1 | 0 |
| 不恰当的 unwrap/expect | 1 | 2 | 3 | 0 |
| depot.get 模式 | 1 | 0 | 0 | 1 |
| depot.obtain 模式 | 0 | 0 | 0 | 1 |
| 事务处理 | 0 | 1 | 0 | 0 |
| 通知回调格式 | 0 | 0 | 0 | 1 |
| **总计（新增）** | **2** | **7** | **4** | **3** |

### 最需要优先修复的问题（错误处理）

1. **[高危]** `user_auth.rs:180` — `depot.get::<AppInfo>().unwrap()` 在中间件中可直接引发 panic。建议：改为 match 表达式，与其他 50+ 处调用保持一致。

2. **[高危]** `notify.rs:195-202, 206-216` — 支付通知事务中 agent update/insert 错误被 `let _` 静默吞没，可能导致资金数据不一致。建议：检查结果，失败时回滚事务并记录错误。

3. **[中危]** `notify.rs:93,100,106,122,152,165,219,232` — 8 处 `let _ = tx.rollback()/commit()` 静默吞没事务控制错误。建议：记录 rollback/commit 失败日志。

4. **[中危]** `cdk_kami.rs:428-504, cdk_user.rs:414-484` — 批量卡密生成事务部分成功不回滚。建议：单条失败时回滚整个事务或使用逐条插入+错误收集。