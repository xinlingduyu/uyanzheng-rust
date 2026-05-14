# Nakamasa-Ichika 第一批安全审查报告

审查时间：2026-05-13
范围：QuickJS 云函数、支付通知/回调、Token/IP 验证开关、CORS 配置化。
说明：本报告按用户要求忽略 admin token 校验与 user token 机制不一致的问题；admin 与 user 两套认证系统仍按当前项目边界分别处理。

## 一、建议修复顺序

1. QuickJS 云函数运行时安全边界
   - 先修复运行时资源限制、阻塞/死循环、HTTP SSRF、SQL 条件/排序拼接、列名空值问题。
   - 原因：云函数执行入口一旦暴露给低权限用户或被后台配置滥用，影响范围最大，可能导致数据库越权、内网探测、线程阻塞。

2. 支付异步通知幂等与金额校验
   - 修复 notify 重复通知/并发重复入账，验证回调金额、订单号、支付渠道一致性。
   - 原因：直接影响资金和用户资产；当前已把同步 return 只读化，这是正确方向，但异步 notify 仍需更强事务条件。

3. Token 验证开关与 IP 绑定开关配置化
   - 在 config.yaml 中新增明确开关，默认保持安全开启。
   - 只控制是否启用验证，不改变 admin JWT 核心签发/验签/续期流程。

4. CORS 白名单配置化
   - 从 config.yaml 读取 allowed_origins、allowed_headers、allowed_methods、allow_credentials、max_age。
   - 按请求 Origin 精确匹配后回显，不再硬编码 127.0.0.1:8888。

## 二、P0 / 高危问题

### P0-1 QuickJS 未设置 CPU/内存/执行时间硬限制

位置：Nakamasa-Ichika/src/core/quickjs_runtime.rs
证据：QuickJsRuntime::new 仅 Runtime::new()；execute 中直接 ctx.eval(code)。未看到 set_memory_limit、set_max_stack_size、interrupt handler 或外层超时。

风险：
- 恶意或错误 JS 可以 while(true) 阻塞当前 worker。
- 大数组/递归可导致内存压力或栈溢出。
- HTTP/DB/Redis 桥接函数内部使用 block_on，叠加 JS 死循环会放大线程阻塞风险。

建议：
- Runtime 创建后设置内存限制、栈限制。
- 给 execute 增加执行超时机制；如果 rquickjs 版本支持 interrupt handler，按 deadline 中断。
- 限制云函数代码体积与返回体大小。
- HTTP timeout 做上限钳制，例如 1..=10 秒。

### P0-2 QuickJS SQL 拼接仍有绕过面：Where/Order 直接拼接

位置：Nakamasa-Ichika/src/core/quickjs_runtime.rs
证据：
- Db.prototype.Where(condition) 直接 this._where = condition。
- Find/FindAll/Updates/Delete/IncOrDec 将 this._where 拼入 SQL。
- Db.prototype.Order(order) 直接拼入 ORDER BY。
- validate_sql_security 是字符串黑名单，不能替代参数化和标识符白名单。

风险：
- WhereParam 是安全方向，但旧 Where 仍允许条件字符串拼接。
- Order 可注入表达式或函数，造成绕过、慢查询或信息探测。

建议：
- 第一批至少把 Where 标记为只允许简单安全表达式，或内部调用 validateCondition。
- 推荐强制使用 WhereParam，保留 Where 仅兼容简单 `field = value` 且字段名白名单/正则校验。
- Order 只允许 `字段名 ASC|DESC`，字段名必须通过非空 sanitizeColumnName 校验。

### P0-3 QuickJS HTTP 能访问任意 URL，存在 SSRF 风险

位置：Nakamasa-Ichika/src/core/quickjs_runtime.rs execute_http / Http 类
证据：execute_http 直接 reqwest::Client 请求 JS 传入 url。

风险：
- 云函数可访问 127.0.0.1、内网服务、云元数据地址、Redis/管理面板等。
- 可作为内网探测和数据 exfiltration 通道。

建议：
- 默认禁止访问 localhost、127.0.0.0/8、10.0.0.0/8、172.16.0.0/12、192.168.0.0/16、169.254.0.0/16、::1、fc00::/7、fe80::/10。
- 仅允许 http/https。
- 可在 config.yaml 增加 cloud_function.http_allowlist，默认空则只允许公网域名。

### P0-4 支付 notify 非原子幂等，可能重复入账

位置：Nakamasa-Ichika/src/app/handlers/api/index/notify.rs
证据：
- ali_notify/wx_notify 先 SELECT 订单 state，如果 state != 0 返回 success。
- update_order 内部无 `WHERE state = 0` 条件；并发 notify 可同时读取 state=0 后重复执行用户资产更新。

风险：
- 支付平台重复通知或攻击者并发回放合法 notify 时，可能造成 vip/fen/agent/balance 重复增加。

建议：
- 在事务内先执行：`UPDATE u_order SET state = 2, trade_no = ?, end_time = ? WHERE id = ? AND state = 0`。
- 检查 rows_affected == 1 才继续发放资产；为 0 则说明已处理，直接 commit/返回 success，不再发放。
- 可加唯一索引/约束辅助 trade_no 或 order_no 幂等。

### P0-5 支付 notify 缺少金额一致性校验

位置：Nakamasa-Ichika/src/app/handlers/api/index/notify.rs 与 pay 插件 verify_notify 接口
证据：verify_notify 当前只返回 trade_no；notify.rs 未从回调中校验 total_amount/cash_fee 与订单 money 是否一致。

风险：
- 如果签名验证正确但订单金额未校验，低金额订单或篡改金额通知可能错误完成高金额订单。

建议：
- 修改 PayPlugin::verify_notify 返回结构体，例如 NotifyVerified { trade_no, out_trade_no, amount_cent, status }。
- notify 中比较：URL order_no == 回调 out_trade_no、amount_cent == u_order.money、支付渠道一致、交易状态成功。
- 支付回调失败仍返回 fail，已处理重复通知返回 success。

## 三、P1 / 中高风险问题

### P1-1 admin_auth 中 skip_token_verify 字段只影响返回 tokenVerify 结果，不是真正跳过验证

位置：Nakamasa-Ichika/src/app/middleware/admin_auth.rs
证据：handle 开头仍然读取 token 并完整 verify；skip_token_verify 只在后面决定是否返回 TokenVerifyResult。

风险：
- 字段名容易误导后续维护者。
- 用户提出“token验证开启开关”时，需要明确新增全局配置开关，而不是复用该字段。

建议：
- 新增 config.yaml：security.admin_token_verify_enabled，默认 true。
- 在 admin_auth 中仅在该开关为 false 时跳过 token 验证，并且必须只用于开发/内网场景。
- 注意：按项目安全边界，不改 JWT 签名/验证核心流程，只加开关分支。

### P1-2 user token 校验缺少全局配置开关

位置：Nakamasa-Ichika/src/app/middleware/user_auth.rs
证据：UserAuth 有 check_token 字段和 skip_token() 路由级开关，但没有从 config.yaml 读取的全局启停配置。

建议：
- 新增 config.yaml：security.user_token_verify_enabled，默认 true。
- 实际判断：`if self.check_token && app_conf.security().user_token_verify_enabled()`。
- 默认不改变现有路由行为。

### P1-3 IP 绑定验证应配置化，且代理头可信边界不明确

位置：
- Nakamasa-Ichika/src/app/middleware/admin_auth.rs claim_ip != ip_str
- Nakamasa-Ichika/src/app/middleware/admin_auth.rs get_client_ip
- Nakamasa-Ichika/src/app/middleware/user_auth.rs get_client_ip
- Nakamasa-Ichika/src/core/middleware/client_ip.rs

风险：
- admin 当前 JWT 内 claim_ip 强绑定请求 IP；移动网络/NAT/反代场景可能误杀。
- get_client_ip 信任 X-Real-IP / X-Forwarded-For，但没有可信代理列表；直连公网时客户端可伪造这些头。

建议：
- config.yaml 新增：security.admin_ip_bind_enabled，默认 true。
- 可选新增 trusted_proxies；只有 remote_addr 属于 trusted_proxies 时才采信 X-Forwarded-For/X-Real-IP。
- user 侧如果存在 IP 冻结/限流，也应统一使用同一可信 IP 提取函数，避免多个 get_client_ip 逻辑不一致。

### P1-4 CORS 硬编码单一 Origin，不从配置读取

位置：Nakamasa-Ichika/src/app/middleware/cors.rs
证据：CORS_ORIGIN 静态写死 http://127.0.0.1:8888；allow_credentials=true。

风险：
- 部署到真实域名后跨域异常。
- 若未来改为 * 且 credentials=true 会形成危险组合。

建议：
- config.yaml 新增 cors 配置：allowed_origins、allowed_headers、allowed_methods、allow_credentials、max_age。
- 请求有 Origin 时，只有命中 allowed_origins 才设置 Access-Control-Allow-Origin 为该 Origin。
- 不允许 credentials=true 与 wildcard origin 同时使用。

## 四、P2 / 中风险与逻辑问题

### P2-1 QuickJS sanitizeColumnName 可能返回空字符串

位置：Nakamasa-Ichika/src/core/quickjs_runtime.rs helpers_code
证据：sanitizeColumnName 只 replace 非法字符为空；Add/Updates/IncOrDec 未检查空字段名。

风险：
- 构造非法 key 可能生成畸形 SQL。

建议：
- sanitize 后如果为空，直接抛错或返回 {OK:false}。
- 对 data keys 做 every 校验后再生成 SQL。

### P2-2 QuickJS IncOrDec 未强制数值化

位置：Nakamasa-Ichika/src/core/quickjs_runtime.rs Db.prototype.IncOrDec
证据：v 直接拼入 SQL 表达式，依赖 JS 比较和字符串拼接。

风险：
- 字符串/NaN/Infinity 等边界值造成 SQL 异常或绕过。

建议：
- `var num = Number(v); if (!Number.isFinite(num)) return {OK:false}`。
- 更好：改成参数化 `safeKey = safeKey + ?`，不要拼接数值字面量。

### P2-3 notify XML 解析用正则，健壮性不足

位置：Nakamasa-Ichika/src/app/handlers/api/index/notify.rs parse_xml_to_json
风险：
- 对复杂 XML、实体、重复字段等处理不可控。

建议：
- 微信通知使用成熟 XML 解析库或微信 SDK 规范解析。
- 限制 payload 大小，拒绝过大的通知体。

## 五、第一批落地文件清单建议

1. QuickJS
   - Nakamasa-Ichika/src/core/quickjs_runtime.rs
   - 可选新增：Nakamasa-Ichika/src/config/security.rs 或 app_config.rs 中 cloud_function 配置

2. 支付通知
   - Nakamasa-Ichika/src/app/handlers/api/index/notify.rs
   - Nakamasa-Ichika/src/app/plugins/pay/mod.rs / trait 定义文件
   - Nakamasa-Ichika/src/app/plugins/pay/ali.rs
   - Nakamasa-Ichika/src/app/plugins/pay/wx.rs
   - Nakamasa-Ichika/src/app/plugins/pay/jie.rs

3. 配置开关
   - Nakamasa-Ichika/src/config/mod.rs
   - Nakamasa-Ichika/src/config/app_config.rs 或新增 security.rs / cors.rs
   - Nakamasa-Ichika/src/app/handlers/api/index/install.rs（生成 config.yaml 模板）
   - config.yaml（如果当前环境存在配置文件，需同步新增默认项）

4. CORS
   - Nakamasa-Ichika/src/app/middleware/cors.rs

5. IP 提取统一化
   - Nakamasa-Ichika/src/core/middleware/client_ip.rs
   - Nakamasa-Ichika/src/app/middleware/admin_auth.rs
   - Nakamasa-Ichika/src/app/middleware/user_auth.rs

## 六、第一批修复验收标准

1. cargo check -p Nakamasa-Ichika 通过。
2. QuickJS：死循环脚本能被超时中断；超大内存分配被限制；Http 访问 127.0.0.1/内网被拒绝。
3. 支付 notify：同一订单并发两次通知，只入账一次；金额不一致返回 fail；重复已处理通知返回 success。
4. 配置：无 config.yaml 时默认安全开启；安装生成的 config.yaml 包含 security 和 cors 默认配置。
5. CORS：白名单命中才回显 Origin；未命中不设置允许跨域；credentials 与 wildcard 不形成危险组合。
6. admin token：除新增开关外，不改 JWT 签发/验签/续期核心逻辑。

## 七、建议配置样例

```yaml
security:
  admin_token_verify_enabled: true
  user_token_verify_enabled: true
  admin_ip_bind_enabled: true
  trust_proxy_headers: false
  trusted_proxies:
    - "127.0.0.1"

cors:
  allowed_origins:
    - "http://127.0.0.1:8888"
  allowed_headers:
    - "content-type"
    - "authorization"
    - "accept-language"
    - "token"
  allowed_methods:
    - "GET"
    - "POST"
    - "PUT"
    - "DELETE"
    - "OPTIONS"
  allow_credentials: true
  max_age: 86400

cloud_function:
  timeout_ms: 3000
  memory_limit_bytes: 33554432
  max_stack_size_bytes: 1048576
  http_timeout_secs: 10
  http_block_private_network: true
  http_allowed_hosts: []
```
