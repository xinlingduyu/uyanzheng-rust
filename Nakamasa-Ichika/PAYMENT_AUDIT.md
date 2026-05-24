# 支付模块深度审计报告

审计日期: 2026-05-24
审计范围: pay.rs(下单) / notify.rs(异步通知) / ali.rs / wx.rs / jie.rs / qq.rs / paypal.rs / trait_def.rs / manager.rs

---

## [CRITICAL] 通知验签分支逻辑错误导致 QQ/PayPal 回调控用微信配置

**文件**: `src/app/handlers/api/index/notify.rs`
**行号**: 383-391

```rust
let pay_type_val: Option<String> = if payment == "ali" {
    app.try_get("pay_ali_type").ok()
} else {
    app.try_get("pay_wx_type").ok()  // ← else 分支仅处理 wx
};
let pay_config_val: Option<String> = if payment == "ali" {
    app.try_get("pay_ali_config").ok()
} else {
    app.try_get("pay_wx_config").ok()  // ← 同样只读 wx
};
```

`handle_notify_inner` 被 `ali_notify`/`wx_notify`/`qq_notify`/`paypal_notify` 四个 handler 共用，但内部列名提取使用 `if payment == "ali" { ali } else { wx }` 的硬编码分支。当 `payment` 为 `"qq"` 或 `"paypal"` 时，会错误地读取 `pay_wx_type`/`pay_wx_config` 列。

**影响**: QQ 钱包和 PayPal 的异步通知永远无法正确验签，因为它们会使用微信支付的配置（或 NULL）去验签，结果永远是 `"fail"`。资金不会被发放，但这是业务阻断级 bug。

**修复方案**: 改为根据 `payment` 参数动态选择列名（如 `pay_{payment}_type`/`pay_{payment}_config`）或做完整匹配。

**风险等级**: 🔴 **CRITICAL**

---

## [CRITICAL] QQ 异步通知 SQL 查询列名与数据库定义不匹配

**文件**: `src/app/handlers/api/index/notify.rs`
**行号**: 465

```rust
"SELECT app_type, pay_qq_type, pay_qq_config FROM u_app WHERE id = ?"
```

数据库中对应列名为 `pay_qqpay_type` / `pay_qqpay_config`（见 `src/core/migration.rs:110-111` 和 `src/app/handlers/api/index/install.rs:744-746`）。

**影响**: QQ 通知 SQL 查询返回 NULL，导致 `pay_config_val` 为 `None`，进入 `res.render(Text::Plain("fail"))` 分支。结合上一条 bug（走 else 分支读 wx 列），双重失效。

**修复方案**: 将 SQL 改为 `pay_qqpay_type` / `pay_qqpay_config`。

**注意**: PayPal 的列名匹配（`pay_paypal_*` vs DB `pay_paypal_*`）正确，但受第1个 bug 影响同样无法工作。

**风险等级**: 🔴 **CRITICAL**

---

## [HIGH] 用户下单 QQ/PayPal 传空配置，支付插件无法初始化

**文件**: `src/app/handlers/api/user/trade/pay.rs`
**行号**: 384-385 (QQ), 408 (PayPal)

```rust
// QQ
let config = serde_json::json!({}); // 注释说"QQ支付配置暂从 u_app 扩展字段读取，当前传空使用插件默认"
let plugin = match create_pay_plugin("qq", &config) { ... };

// PayPal
let config = serde_json::json!({});
let plugin = match create_pay_plugin("paypal", &config) { ... };
```

`AppInfo` 结构体（`src/app/middleware/app_context.rs:20-86`）仅定义了 `alipay_config`/`alipay_state`/`alipay_type` 和 `wechat_pay_config`/`wechat_pay_state`/`wechat_pay_type`，没有 QQ/PayPal 的对应字段。

QQ 和 PayPal 支付插件 `init(json!({}))` 后，`qq_appid`/`qq_mchid`/`qq_key`/`client_id`/`client_secret` 等均为 `None`。后续 `create()` 调用会因"支付参数未配置"错误而失败。

而 ali/wx 分支有完整的 `app_info.xxx_config` 读取和解析逻辑（pay.rs:285-316, 337-368）。

**影响**: 用户端 QQ/PayPal 支付通道完全不可用。

**修复方案**: 在 `AppInfo` 中增加 `qqpay_state`/`qqpay_type`/`qqpay_config` 和 `paypal_state`/`paypal_type`/`paypal_config` 字段，并在 pay.rs 中像 ali/wx 一样读取。

**风险等级**: 🔴 **CRITICAL** (用户端支付不可用)

---

## [HIGH] 通知事务中 agent 更新/插入错误被静默吞没

**文件**: `src/app/handlers/api/index/notify.rs`
**行号**: 195 (更新), 206 (插入)

```rust
// 行195 — 更新已有代理
let _ = sqlx::query("UPDATE u_agent SET pay_divide = GREATEST(pay_divide, ?), km_discount = LEAST(km_discount, ?) WHERE id = ?")
    .bind(...)
    .execute(&mut *tx)
    .await;  // 错误被忽略！

// 行206 — 插入新代理
let _ = sqlx::query("INSERT INTO u_agent (aggid, uid, pay_divide, km_discount, time, appid) VALUES (?, ?, ?, ?, ?, ?)")
    .bind(...)
    .execute(&mut *tx)
    .await;  // 错误被忽略！
```

两个操作的错误都被 `let _` 静默吞没。如果数据库写入失败（死锁、约束冲突等），订单状态已经更新为 `state=2`（已支付），但代理数据未正确写入。事务没有回滚，导致资金数据不一致。

**影响**: 用户付款成功并收到权益，但代理分成数据丢失。`Divide_money` 代理分账部分（行111-124）有正确错误检查和回滚，但代理等级更新/开通部分没有。

**修复方案**: 改为 `if sqlx::query(...).execute(&mut *tx).await.is_err() { tx.rollback().await; return false; }`。

**风险等级**: 🔴 **CRITICAL** (资金数据一致性问题)

---

## [HIGH] 通知事务中多处 rollback/commit 错误被静默吞没

**文件**: `src/app/handlers/api/index/notify.rs`
**行号**: 93, 100, 106, 122, 152, 165, 219, 232

所有这8处使用 `let _ = tx.rollback().await;` 或 `let _ = tx.commit().await;`。

其中第238行的最终 `tx.commit().await.is_ok()` 检查了结果，但其他8处均未检查。

**影响**: 如果 rollback 失败（MySQL 连接断开等），程序不会记录日志，调试困难。第100行甚至在没有错误的情况下默默忽略 commit 失败——这可能导致事务悬空。

**修复方案**: 添加 `.map_err(|e| tracing::error!(...))` 日志记录。

**风险等级**: 🟠 **HIGH**

---

## [MEDIUM] 皆网支付金额单位不明确——create 发送分，verify_notify 按元解析

**文件**: `src/app/plugins/pay/jie.rs`
**行号**: 163 (create), 234-237 (verify_notify)

**create 发往网关**:
```rust
data.insert("money".to_string(), format!("{}", order.money));
```
`order.money` 是 `f64`，来自 DB 的 `i64`（单位分）。例如 100 分 → `"100"`。

**verify_notify 解析返回**:
```rust
.get("money")
.and_then(|s| s.parse::<f64>().ok())
.map(|v| (v * 100.0).round() as i64)
```
将收到的 `money` 视作元，乘以 100 转回分。

如果皆网平台返回金额单位为元（`"1.00"`），则 verify 正确但 create 发送了错误的值（`"100"` 元而非 `"1.00"` 元）。如果皆网返回单位为分（`"100"`），则 create 正确但 verify 计算出 `100*100=10000` 分，金额验证失败。

**影响**: 取决于皆网平台实际行为，可能多收或少收用户金额，或金额验证失败。

**风险等级**: 🟡 **MEDIUM** (需确认皆网平台接口规范)

---

## [MEDIUM] 支付网关下单早于本地订单创建

**文件**: `src/app/handlers/api/user/trade/pay.rs`
**行号**: 319-330 (调用 plugin.create), 467-486 (INSERT 订单)

代码先调用支付网关的 `create()` 成功创建外部订单，然后才在本地数据库 INSERT `u_order`。如果 INSERT 失败（主键冲突、连接断开等），支付网关侧已存在一笔待支付订单，但本地无对应记录。用户付款后，异步通知时查询订单不存在，返回 `"fail"`，资金无法到账。

**影响**: 极端情况下的资金损失风险（用户付款了却未收到权益）。

**修复方案**: 先将订单写入 DB，再调用网关创建。或增加补偿机制（定时任务扫描支付网关的未处理订单）。

**风险等级**: 🟡 **MEDIUM**

---

## [MEDIUM] 订单号可预测性

**文件**: `src/app/handlers/api/user/trade/pay.rs`
**行号**: 191-197

```rust
let mut rng = rand::rngs::StdRng::from_entropy();
let order_no = format!(
    "{}{:05}",
    Utc::now().format("%Y%m%d%H%M%S"),
    rng.gen_range(10000..99999)
);
```

订单号 = 时间戳(14位) + 5位随机数。使用 `StdRng::from_entropy()` 种子尚可，但随机空间仅 89999 种可能，同一秒内可枚举。虽然不像纯自增 ID 那样易预测，但攻击者仍可能枚举同一秒内的订单号。

**影响**: 潜在的信息泄露风险（可枚举用户订单）。

**风险等级**: 🟢 **LOW**

---

## [INFO] QQ/PayPal 通知路由未注册

**文件**: `src/app/handlers/api/index/mod.rs`
**行号**: 14-23

当前注册的通知路由：
- `/notify/ali/<order_no>` ✓
- `/notify/wx/<order_no>` ✓
- `/notify/qq/<order_no>` ✓ (存在，与 CRITICAL bug 2 的列名问题不同)
- `/notify/paypal/<order_no>` ✓ (存在)

路由已注册，但 `qq_notify` 和 `paypal_notify` handler 因上述 CRITICAL bug 无法正常工作。

**风险等级**: 🟢 **INFO**

---

## [INFO] admin pay.rs generate_plugins_json 未包含 QQ/PayPal 插件列表

**文件**: `src/app/handlers/api/admin/app/pay.rs`
**行号**: 372-398

`generate_plugins_json()` 只实例化了 `JiePayPlugin`、`AliPayPlugin`、`WxPayPlugin`，未包含 `QqPayPlugin`、`PayPalPayPlugin`。虽然 CHANNEL_DEFS 包含了 qqpay 和 paypal 通道定义，但管理后台不会展示 QQ/PayPal 作为可选支付引擎。

**影响**: 管理员无法在后台为 QQ/PayPal 通道选择引擎类型（数据库默认值为 `'jie'`）。

**风险等级**: 🟢 **LOW**

---

## 汇总

| # | 等级 | 描述 | 文件:行 |
|---|------|------|---------|
| 1 | 🔴 CRITICAL | 通知验签分支硬编码只支持 ali/wx，qq/paypal 误读微信配置 | `notify.rs:383-391` |
| 2 | 🔴 CRITICAL | QQ 通知 SQL 查询 `pay_qq_*` 但数据库列为 `pay_qqpay_*` | `notify.rs:465` |
| 3 | 🔴 CRITICAL | 用户下单 QQ/PayPal 传空配置，插件初始化失败 | `pay.rs:384-385, 408` |
| 4 | 🔴 CRITICAL | 通知中 agent 更新/插入错误被 `let _` 吞没 | `notify.rs:195, 206` |
| 5 | 🟠 HIGH | 8处事务 rollback/commit 错误被静默吞没 | `notify.rs:93,100,106,122,152,165,219,232` |
| 6 | 🟡 MEDIUM | 皆网 create 发分、verify 按元解析，单位可能不一致 | `jie.rs:163, 234-237` |
| 7 | 🟡 MEDIUM | 支付网关下单早于本地订单 INSERT | `pay.rs:319-330 vs 467-486` |
| 8 | 🟢 LOW | 订单号随机空间仅 5 位 | `pay.rs:193-197` |
| 9 | 🟢 LOW | admin 后台未列出 QQ/PayPal 支付引擎 | `admin/pay.rs:372-398` |

**前 4 项为关键/高危，建议优先修复。**