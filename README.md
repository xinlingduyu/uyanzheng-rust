# Nakamasa-Ichika

高性能用户认证与应用管理全栈平台，基于 Rust + Vue 3 构建。

---

## 项目简介

Nakamasa-Ichika 是一套面向多应用场景的用户认证与应用管理解决方案。后端采用 Rust 语言编写，借助 Salvo 框架实现高并发、低延迟的 HTTP/HTTPS/QUIC 服务；前端基于 Vue 3 + Arco Design 构建现代化管理后台。系统支持单实例多应用独立配置与隔离，适用于软件授权验证、SaaS 平台、代理分销等业务场景。

**代码规模：** 后端 Rust 约 60,000 行，前端 Vue/JS 约 42,000 行。

---

## 核心功能

| 功能模块 | 说明 |
|---------|------|
| 用户认证 | 账号密码、手机号、邮箱、卡密充值等多种登录方式 |
| OAuth2.0 | QQ 登录、微信登录（SDK / Web 两种模式） |
| 多应用支持 | 单实例承载多个应用，各应用独立配置、独立数据隔离 |
| 代理推广 | 代理分组管理、推广链接、自动分成结算 |
| 积分系统 | 积分事件规则、积分订单、积分消费与充值 |
| 支付集成 | 支付宝、微信支付、捷付（多渠道热插拔） |
| 卡密系统 | 卡密分组生成、批量导出、卡密充值、卡密用户管理 |
| 云函数 | QuickJS / V8 运行时，支持 Db / Redis / Http 操作 |
| 加解密插件 | AES / DES / RC4 / RSA 四种加密方式，跨平台纯 Rust 实现 |
| 邮件通知 | SMTP 邮件发送（基于 lettre 库） |
| 短信通知 | 阿里云短信、腾讯云短信、捷信短信 |
| GeoIP 定位 | 基于 MaxMind GeoLite2 的 IP 地理位置查询 |
| 国际化 | 中/英/日三语支持（前后端同步） |
| 数据可视化 | ECharts / ApexCharts 图表，中国地图 / 世界地图组件 |
| 版本管理 | 应用版本发布、更新检测 |
| 黑名单 | 设备 UDID / IP 维度黑名单管理 |
| 文件上传 | 分片上传、图片上传、文件管理 |

---

## 技术栈

### 后端

| 技术 | 版本 | 用途 |
|------|------|------|
| Rust | 1.85+ (Edition 2024) | 编程语言 |
| Salvo | latest | Web 框架（HTTP / HTTPS / QUIC / HTTP2） |
| SQLx | 0.8 | 异步数据库驱动（MySQL / PostgreSQL / SQLite） |
| Deadpool-Redis | 0.22 | Redis 异步连接池 |
| rustls + aws-lc-rs | 0.23 | TLS 加密（纯 Rust 实现，无 OpenSSL 依赖） |
| rquickjs | - | QuickJS 绑定（Android 平台云函数运行时） |
| lettre | 0.11 | SMTP 邮件发送 |
| reqwest | 0.12 | HTTP 客户端（支付回调等） |
| tracing | 0.1 | 结构化日志 |
| serde / serde_json | 1.0 | 序列化框架 |
| fluent-templates | 0.9 | 后端国际化 |
| chrono | 0.4 | 时间处理 |
| rand | 0.9 | 随机数生成 |
| sha2 / hmac / md5 | - | 哈希与签名 |
| rsa | 0.9 | RSA 非对称加密 |
| aes / des / cbc | 0.8 | 对称加密 |
| maxminddb | 0.26 | GeoIP 地理定位 |
| notify | 8.2 | 配置文件热更新监听 |
| rust-embed | - | 静态文件编译时嵌入 |

### 前端

| 技术 | 版本 | 用途 |
|------|------|------|
| Vue | 3.4 | 前端框架（Composition API） |
| Arco Design Vue | 2.57 | UI 组件库 |
| Vite | 5 | 构建工具 |
| Pinia | 2.1 | 状态管理（含持久化插件） |
| Vue Router | 4.2 | 路由管理 |
| vue-i18n | 9.1 | 国际化 |
| ECharts | 5.4 | 图表可视化 |
| ApexCharts | 5.10 | 高级图表 |
| Tailwind CSS | 3.4 | 原子化 CSS |
| Monaco Editor | 0.33 | 代码编辑器（云函数编辑） |
| wangEditor | 5.1 | 富文本编辑器 |
| md-editor-v3 | 4.13 | Markdown 编辑器 |
| dayjs | 1.11 | 日期处理 |
| crypto-js | 4.2 | 前端加密 |
| axios | 0.27 | HTTP 请求 |
| Less | 4.1 | CSS 预处理器 |
| Playwright | 1.58 | E2E 测试 |

---

## 项目结构

```
web/
├── Nakamasa-Ichika/          # 后端主应用
│   ├── src/
│   │   ├── main.rs           # 应用入口
│   │   ├── cli.rs            # CLI 命令行工具
│   │   ├── config/           # 配置模块
│   │   │   ├── mod.rs        # 配置加载与全局访问
│   │   │   ├── server.rs     # 服务器配置
│   │   │   ├── mysql.rs      # MySQL 配置
│   │   │   ├── redis.rs      # Redis 配置
│   │   │   ├── i18n.rs       # 国际化配置
│   │   │   ├── app_config.rs # 应用业务配置
│   │   │   └── debug.rs      # 调试配置
│   │   ├── core/             # 核心基础设施
│   │   │   ├── mod.rs
│   │   │   ├── app_state.rs  # 全局应用状态
│   │   │   ├── server.rs     # HTTP/HTTPS/QUIC 服务器
│   │   │   ├── mysql.rs      # MySQL 连接池
│   │   │   ├── redis.rs      # Redis 连接池
│   │   │   ├── cache.rs      # 多级缓存
│   │   │   ├── lru_cache.rs  # LRU 缓存实现
│   │   │   ├── admin_cache.rs # 管理员缓存服务
│   │   │   ├── i18n.rs       # 国际化中间件
│   │   │   ├── terminal_i18n.rs # 终端国际化
│   │   │   ├── error.rs      # 统一错误处理
│   │   │   ├── handler_ext.rs # Handler 扩展 trait
│   │   │   ├── quickjs_runtime.rs # QuickJS 云函数运行时
│   │   │   ├── v8_runtime.rs # V8 云函数运行时
│   │   │   ├── notify.rs     # 配置热更新监听
│   │   │   ├── json_optimize.rs  # JSON 优化
│   │   │   ├── md5_optimize.rs   # MD5 栈上计算
│   │   │   ├── zero_copy.rs  # 零拷贝字符串
│   │   │   ├── regex_cache.rs # 正则预编译缓存
│   │   │   ├── db_optimize.rs # 数据库优化
│   │   │   ├── arch.rs       # 架构相关优化
│   │   │   ├── flamegraph.rs # 性能火焰图
│   │   │   └── middleware/
│   │   │       └── client_ip.rs  # 客户端 IP 解析
│   │   └── app/              # 业务逻辑层
│   │       ├── mod.rs
│   │       ├── routes.rs     # 路由定义
│   │       ├── handlers/     # HTTP 请求处理器
│   │       │   ├── health.rs # 健康检查
│   │       │   ├── hello.rs  # 欢迎页
│   │       │   ├── static_files.rs # 静态文件服务
│   │       │   └── api/
│   │       │       ├── admin/  # 管理员 API（30+ 接口）
│   │       │       ├── user/   # 用户 API（30+ 接口）
│   │       │       └── index/  # 首页/安装/回调 API
│   │       ├── models/       # 数据模型（20+ 实体）
│   │       ├── middleware/    # 业务中间件
│   │       │   ├── admin_auth.rs  # 管理员 JWT 认证
│   │       │   ├── user_auth.rs   # 用户 JWT 认证
│   │       │   ├── app_context.rs # 应用上下文注入
│   │       │   ├── cors.rs        # CORS 跨域
│   │       │   ├── body_reader.rs # 请求体解析
│   │       │   └── connect.rs     # 连接管理
│   │       ├── plugins/      # 热插拔插件系统
│   │       │   ├── encryption/ # 加解密（AES/DES/RC4/RSA）
│   │       │   ├── pay/       # 支付（支付宝/微信/捷付）
│   │       │   ├── mailer/    # 邮件（SMTP）
│   │       │   └── sms/       # 短信（阿里云/腾讯云/捷信）
│   │       └── utils/        # 业务工具
│   │           ├── response.rs   # 统一响应格式
│   │           └── validator.rs  # 参数校验
│   ├── locales/              # 国际化语言文件
│   │   ├── zh-CN.json
│   │   └── en.json
│   ├── certs/                # TLS 证书
│   └── Cargo.toml
│
├── Nakamasa-utils/           # 工具库
│   ├── src/
│   │   ├── lib.rs            # 库入口
│   │   ├── jwt.rs            # JWT 签发与验证（HMAC-SHA256）
│   │   ├── geoip.rs          # GeoIP 地理位置查询
│   │   ├── db_mysql.rs       # MySQL 数据库工具
│   │   ├── tiered_cache.rs   # 分片 LRU 缓存（V1）
│   │   ├── crypto.rs         # 配置加密（AES-256-CBC + SHA3-256 密钥派生）
│   │   ├── high_perf_cache/  # 高性能缓存系统（V2）
│   │   │   ├── shard.rs / shard_v2.rs    # 分片缓存（无锁读取 + O(1) LRU）
│   │   │   ├── pool.rs / pool_v2.rs      # 内存池（线程本地缓存 + 无锁分配）
│   │   │   ├── write_buffer.rs           # 延迟写入缓冲
│   │   │   ├── stats.rs / stats_v2.rs    # 采样统计 + 分位数估计
│   │   │   ├── hash.rs                   # 哈希函数
│   │   │   ├── atomic.rs                 # 原子操作
│   │   │   ├── policy.rs                 # 淘汰策略
│   │   │   ├── config.rs                 # 缓存配置
│   │   │   ├── manager.rs               # 缓存管理器
│   │   │   ├── arch_aarch64.rs          # ARM 架构优化
│   │   │   └── arch_x86_64.rs           # x86 架构优化
│   │   └── distributed/     # 分布式缓存
│   │       ├── multi_level.rs         # 多级缓存（L1 本地 + L2 Redis）
│   │       ├── distributed_lock.rs    # 分布式锁
│   │       ├── sync_broadcast.rs      # 缓存同步/失效广播
│   │       ├── consistent_hash.rs     # 一致性哈希分片
│   │       └── redis_backend.rs       # Redis 后端适配
│   └── Cargo.toml
│
├── Nakamasa-proc/            # 过程宏库
│   ├── src/
│   │   └── lib.rs            # #[route] 路由宏、控制器宏、中间件宏
│   └── Cargo.toml
│
├── view/                     # 前端管理后台（SaiAdmin）
│   ├── src/
│   │   ├── main.js           # 应用入口
│   │   ├── api/              # API 请求封装（30+ 模块）
│   │   ├── components/       # 通用组件（25+ 组件）
│   │   │   ├── sa-table/     # 高级数据表格
│   │   │   ├── sa-chart/     # ECharts 图表
│   │   │   ├── sa-apexchart/ # ApexCharts 图表
│   │   │   ├── sa-china-map/ # 中国地图
│   │   │   ├── sa-world-map/ # 世界地图
│   │   │   ├── sa-icon/      # 图标组件
│   │   │   ├── sa-upload-*/  # 文件/图片/分片上传
│   │   │   ├── sa-dict/      # 字典组件
│   │   │   ├── ma-codeEditor/ # 代码编辑器
│   │   │   ├── ma-wangEditor/ # 富文本编辑器
│   │   │   └── ...
│   │   ├── views/            # 页面视图
│   │   │   ├── dashboard/    # 仪表盘
│   │   │   ├── admin/        # 管理员管理
│   │   │   ├── user/         # 用户管理
│   │   │   ├── app/          # 应用管理
│   │   │   ├── agent/        # 代理管理
│   │   │   ├── goods/        # 商品管理
│   │   │   ├── order/        # 订单管理
│   │   │   ├── pay/          # 支付配置
│   │   │   ├── kami/         # 卡密管理
│   │   │   ├── function/     # 云函数
│   │   │   ├── ver/          # 版本管理
│   │   │   ├── notice/       # 公告管理
│   │   │   ├── message/      # 消息管理
│   │   │   ├── fen/          # 积分管理
│   │   │   ├── encryption/   # 加解密配置
│   │   │   ├── blocklist/    # 黑名单
│   │   │   ├── logs/         # 日志查看
│   │   │   ├── statistics/   # 统计分析
│   │   │   ├── visualization/# 数据可视化
│   │   │   ├── extend/       # 应用扩展
│   │   │   ├── system/       # 系统设置
│   │   │   └── install/      # 安装向导
│   │   ├── router/           # 路由配置
│   │   ├── store/            # Pinia 状态管理（9 个模块）
│   │   ├── i18n/             # 国际化（中/英）
│   │   ├── hooks/            # 组合式函数
│   │   ├── directives/       # 自定义指令（auth/role/copy）
│   │   ├── utils/            # 工具函数
│   │   ├── layout/           # 布局组件
│   │   ├── config/           # 应用配置
│   │   ├── style/            # 全局样式（多皮肤）
│   │   └── mock/             # Mock 数据
│   ├── vite.config.js        # Vite 构建配置
│   └── package.json
│
├── config.yaml               # 运行时配置文件
├── new.sql                   # 数据库初始化脚本
├── GeoLite2-City.mmdb        # GeoIP 地理位置数据库
├── data/                     # 运行时数据目录
│   └── upload/               # 文件上传目录
└── Cargo.toml                # Workspace 根配置
```

---

## 系统架构

### 整体架构

```
┌────────────────────────────────────────────────────────────────────┐
│                         客户端（浏览器 / App）                        │
└───────────────────────────────┬────────────────────────────────────┘
                                │
                    ┌───────────▼───────────┐
                    │   Nginx / 反向代理     │
                    └───────────┬───────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌───────▼────────┐   ┌─────────▼─────────┐   ┌────────▼────────┐
│  静态资源服务   │   │   后端 API 服务    │   │  HTTPS / QUIC   │
│  /admin /static │   │   Salvo + Rust     │   │  rustls + aws-lc│
└────────────────┘   └─────────┬─────────┘   └─────────────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
     ┌────────▼───────┐ ┌─────▼──────┐ ┌───────▼───────┐
     │  L1 本地缓存    │ │ L2 Redis  │ │  MySQL 持久层  │
     │  ShardedCache  │ │  分布式缓存 │ │  SQLx 异步池   │
     │  V2 无锁读取    │ │  连接池    │ │               │
     └────────────────┘ └────────────┘ └───────────────┘
```

### 后端分层架构

```
┌──────────────────────────────────────────────────────────┐
│                     HTTP Request                         │
└──────────────────────────┬───────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────┐
│                   中间件层 (Middleware)                    │
│                                                          │
│  CORS → I18n → AppContext → AdminAuth / UserAuth        │
│                                                          │
│  • CORS:       跨域请求处理                               │
│  • I18n:       语言检测（Query → Cookie → Header）        │
│  • AppContext:  应用上下文注入（app_key 解析）             │
│  • Auth:       JWT Token 认证与权限校验                   │
└──────────────────────────┬───────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────┐
│                   处理器层 (Handlers)                     │
│                                                          │
│  Admin API  │  User API  │  Index API                    │
│  管理员操作  │  用户操作   │  安装/回调/首页               │
└──────────────────────────┬───────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────┐
│              模型/插件/工具层 (Models/Plugins/Utils)       │
│                                                          │
│  Models: 数据实体 + DTO (Request / Response)             │
│  Plugins: 支付 / 加密 / 邮件 / 短信（trait 接口抽象）     │
│  Utils: 统一响应 + 参数校验                               │
└──────────────────────────┬───────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────┐
│                 核心基础设施层 (Core)                      │
│                                                          │
│  AppState │ MySQL Pool │ Redis Pool │ Cache │ I18n       │
│  Server   │ QuickJS    │ V8 Runtime │ GeoIP │ Crypto     │
└──────────────────────────────────────────────────────────┘
```

### 缓存架构

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   L1 本地缓存    │────▶│   L2 Redis      │────▶│   MySQL 持久层   │
│                 │     │                 │     │                 │
│ ShardedCacheV2 │     │  Deadpool-Redis │     │  SQLx Pool      │
│ • 无锁读取      │     │  • 分布式共享    │     │  • 事务支持      │
│ • O(1) LRU     │     │  • TTL 过期     │     │  • 连接池复用    │
│ • 分片减少竞争  │     │  • Pub/Sub 同步 │     │  • 异步 I/O     │
│ • 线程本地池    │     │                 │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### 插件架构

```
┌─────────────────────────────────────────────────────────┐
│                    Plugin Manager                        │
└──────────┬──────────┬──────────┬──────────┬─────────────┘
           │          │          │          │
     ┌─────▼───┐ ┌────▼───┐ ┌───▼────┐ ┌───▼────┐
     │ PayPlugin│ │EncryptPlugin│ │MailerPlugin│ │SmsPlugin│
     │  trait   │ │   trait     │ │   trait    │ │  trait  │
     └─────┬───┘ └────┬───┘ └───┬────┘ └───┬────┘
           │          │          │          │
     ┌─────┴──┐ ┌────┴──┐ ┌────┴──┐ ┌────┴──┐
     │AliPay  │ │ AES   │ │ SMTP  │ │ 阿里云 │
     │WxPay   │ │ DES   │ │       │ │ 腾讯云 │
     │JiePay  │ │ RC4   │ └───────┘ │ 捷信   │
     └────────┘ │ RSA   │           └───────┘
                └───────┘
```

---

## 数据库设计

系统共 21 张数据表，使用 `u_` 前缀：

| 表名 | 说明 |
|------|------|
| `u_admin` | 管理员账户（含 JSON 权限字段） |
| `u_user` | 用户账户（多应用隔离） |
| `u_app` | 应用配置（独立注册/登录/认证策略） |
| `u_app_blocklist` | 应用黑名单 |
| `u_app_extend` | 应用扩展数据 |
| `u_app_function` | 云函数（JavaScript 代码存储） |
| `u_app_mi` | 应用密钥配置 |
| `u_app_notice` | 应用公告 |
| `u_app_ver` | 应用版本信息 |
| `u_cdk_group` | 卡密分组 |
| `u_cdk_kami` | 卡密数据 |
| `u_cdk_user` | 卡密用户关联 |
| `u_fen_event` | 积分事件规则 |
| `u_fen_order` | 积分订单 |
| `u_goods` | 商品管理 |
| `u_login` | 登录记录 |
| `u_logs` | 操作日志 |
| `u_message` | 站内消息 |
| `u_order` | 支付订单 |
| `u_vcode` | 验证码记录 |

---

## API 路由结构

```
/                               # 欢迎页 (GET)
├── /admin/*                    # 管理后台静态文件
├── /static/*                   # 公共静态资源
├── /upload/*                   # 上传文件访问
│
└── /api/
    ├── /health                 # 健康检查
    │
    ├── /install/               # 安装 API
    │   ├── /check              # 安装状态检查
    │   └── /run                # 执行安装
    │
    ├── /admin/                 # 管理员 API（需 AdminAuth）
    │   ├── /login              # 登录
    │   ├── /user               # 用户管理
    │   ├── /app                # 应用管理
    │   ├── /goods              # 商品管理
    │   ├── /order              # 订单管理
    │   ├── /pay                # 支付配置
    │   ├── /cdk                # 卡密管理
    │   ├── /agent              # 代理管理
    │   ├── /function           # 云函数管理
    │   ├── /notice             # 公告管理
    │   ├── /message            # 消息管理
    │   ├── /fen                # 积分管理
    │   ├── /ver                # 版本管理
    │   ├── /encryption         # 加解密配置
    │   ├── /blocklist          # 黑名单管理
    │   ├── /statistics         # 统计分析
    │   ├── /logs               # 日志查看
    │   ├── /system             # 系统设置
    │   ├── /upload             # 文件上传
    │   └── /set                # 配置修改
    │
    ├── /user/                  # 用户 API（需 UserAuth）
    │   ├── /signIn             # 登录
    │   ├── /signUp             # 注册
    │   ├── /info               # 用户信息
    │   ├── /pay                # 发起支付
    │   ├── /order              # 订单查询
    │   ├── /kamiTopup          # 卡密充值
    │   ├── /cloudFunction      # 云函数调用
    │   ├── /heartbeat          # 心跳保活
    │   ├── /qqloginSDK         # QQ SDK 登录
    │   ├── /wxloginSDK         # 微信 SDK 登录
    │   ├── /vip                # VIP 状态
    │   ├── /fen                # 积分操作
    │   └── /logout             # 登出
    │
    ├── /oauth2.0/              # OAuth2 回调
    │   ├── /qqlogon/callback
    │   └── /wxlogon/callback
    │
    └── /index/                 # 首页 API
        ├── /appinfo            # 应用信息
        ├── /config             # 应用配置
        └── /authentication     # 认证信息
```

---

## 安全设计

| 安全措施 | 说明 |
|---------|------|
| 配置加密 | AES-256-CBC + SHA3-256 密钥派生，敏感配置字段 `enc:` 前缀标识 |
| 密码安全 | 解密后密码从内存清除（secure_zero） |
| JWT 认证 | HMAC-SHA256 签名，支持 Token 过期与刷新 |
| TLS 加密 | rustls + aws-lc-rs，支持自定义证书或内置自签证书 |
| CORS 策略 | 可配置的跨域访问控制 |
| 参数校验 | 请求体验证器（validator 模块） |
| IP 限制 | 注册/登录 IP 频率限制 |
| 黑名单 | 设备 UDID / IP 维度封禁 |

---

## 性能优化

| 优化项 | 说明 |
|-------|------|
| 分片缓存 V2 | 无锁读取 + O(1) LRU + 哈希复用，减少锁竞争 |
| 延迟写入 | 分层 Write Buffer + 无锁队列，降低写延迟 |
| 内存池 V2 | 线程本地缓存 + 无锁分配 + 自动回收 |
| 架构优化 | aarch64 / x86_64 平台特定优化 |
| MD5 栈上计算 | 避免 heap 分配的小数据 MD5 优化 |
| 零拷贝字符串 | 减少 JSON 序列化中的字符串拷贝 |
| 正则预编译 | 全局 Regex 缓存池，避免重复编译 |
| JSON 优化 | 高频场景的 JSON 序列化/反序列化优化 |
| 连接池 | MySQL / Redis 连接池，按 CPU 核数自动调整 |
| 静态嵌入 | rust-embed 编译时嵌入管理后台，零 I/O 开销 |

---

## 国际化

### 后端

- 语言文件：`Nakamasa-Ichika/locales/`（zh-CN.json / en.json）
- 语言检测优先级：URL 查询参数 → Cookie → Accept-Language Header → 默认语言
- 实现：`fluent-templates` + 自定义 I18nMiddleware
- 支持语言：中文（zh-CN）、英文（en）、日文（ja）

### 前端

- 语言文件：`view/src/i18n/`（zh_CN / en）
- 实现：`vue-i18n`
- 切换方式：用户设置自动持久化

---

## 安装教程

### 一、环境要求

| 依赖 | 最低版本 | 说明 |
|------|---------|------|
| Rust | 1.85+ | Edition 2024，推荐使用 rustup 安装 |
| Node.js | 18+ | 推荐 20+，前端构建 |
| npm | 8+ | 随 Node.js 安装 |
| MySQL | 5.7+ | 推荐 8.0，数据库存储 |
| Redis | 6.0+ | 推荐 7.0，分布式缓存 |
| pkg-config | 0.29+ | 编译本地库时需要 |
| protoc | 3.0+ | Protocol Buffers 编译器（部分依赖可能需要） |

**硬件建议：**

| 项目 | 最低配置 | 推荐配置 |
|------|---------|---------|
| CPU | 1 核 | 2 核+ |
| 内存 | 512 MB | 1 GB+（编译时需 1 GB+） |
| 磁盘 | 1 GB | 2 GB+（含 GeoIP 数据库） |

---

### 二、安装基础依赖

#### Linux (Debian / Ubuntu)

```bash
# 系统更新
sudo apt update && sudo apt upgrade -y

# 基础构建工具
sudo apt install -y build-essential pkg-config libssl-dev curl git

# Protocol Buffers（可选，部分场景需要）
sudo apt install -y protobuf-compiler

# MySQL
sudo apt install -y mysql-server
sudo systemctl enable mysql
sudo systemctl start mysql

# Redis
sudo apt install -y redis-server
sudo systemctl enable redis-server
sudo systemctl start redis-server
```

#### Linux (CentOS / RHEL)

```bash
# 基础构建工具
sudo yum groupinstall -y "Development Tools"
sudo yum install -y pkg-config openssl-devel curl git

# MySQL
sudo yum install -y mysql-server
sudo systemctl enable mysqld
sudo systemctl start mysqld

# Redis
sudo yum install -y redis
sudo systemctl enable redis
sudo systemctl start redis
```

#### macOS

```bash
# Homebrew 安装构建工具
xcode-select --install
brew install openssl pkg-config protobuf

# MySQL
brew install mysql
brew services start mysql

# Redis
brew install redis
brew services start redis
```

#### Android / Termux

```bash
# 更新包管理器
pkg update && pkg upgrade -y

# 基础构建工具
pkg install -y build-essential openssl pkg-config git

# MySQL (Termux 下使用 MariaDB)
pkg install -y mariadb
mysql_install_db
mysqld_safe &
mysql_secure_installation

# Redis
pkg install -y redis
redis-server --daemonize yes
```

---

### 三、安装 Rust 工具链

#### 标准 Linux / macOS

```bash
# 安装 rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 选择默认安装（1 即可）
# 安装完成后加载环境
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

#### Android / Termux

```bash
# Termux 下通过包管理器安装
pkg install -y rust

# 验证安装
rustc --version
cargo --version
```

> 注意：Rust 版本需 >= 1.85 且支持 Edition 2024。如果版本过低，请通过 `rustup update` 或 `pkg upgrade rust` 升级。

---

### 四、安装 Node.js

#### 使用 nvm 安装（推荐）

```bash
# 安装 nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
source ~/.bashrc

# 安装 Node.js 20 LTS
nvm install 20
nvm use 20

# 验证安装
node --version
npm --version
```

#### 使用包管理器安装

```bash
# Ubuntu / Debian
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# macOS
brew install node@20

# Termux
pkg install -y nodejs
```

---

### 五、准备数据库

#### 创建 MySQL 数据库

```bash
# 登录 MySQL
mysql -u root -p

# 创建数据库和用户
CREATE DATABASE nakamasa DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
CREATE USER 'nakamasa'@'127.0.0.1' IDENTIFIED BY 'your_password';
GRANT ALL PRIVILEGES ON nakamasa.* TO 'nakamasa'@'127.0.0.1';
FLUSH PRIVILEGES;
EXIT;
```

> 注意：数据库表会在首次安装时自动创建，无需手动导入 SQL。`new.sql` 仅作为参考和备份使用。

#### 验证 Redis 连接

```bash
redis-cli ping
# 返回 PONG 表示正常
```

---

### 六、获取项目源码

```bash
# 克隆仓库
git clone <repository-url> web
cd web

# 确认项目结构
ls -la
# 应看到: Cargo.toml  Nakamasa-Ichika/  Nakamasa-utils/  Nakamasa-proc/  view/  config.yaml
```

---

### 七、后端编译

#### 7.1 开发模式编译（快速启动）

```bash
# 在项目根目录执行
cargo build

# 或直接运行（编译 + 启动）
cargo run
```

开发模式特点：
- 编译速度快，无优化
- 包含调试信息（debug = true）
- 二进制文件较大，运行速度较慢
- 首次编译需下载依赖，耗时较长（5~15 分钟视网络而定）

#### 7.2 快速开发编译（兼顾速度与性能）

项目配置了 `dev-fast` profile，在开发时获得接近生产环境的性能：

```bash
cargo build --profile dev-fast
```

#### 7.3 生产模式编译（推荐部署）

```bash
# Release 编译
cargo build --release
```

生产模式特点（已在 `Cargo.toml` 中预配置）：
- `lto = "fat"` — 链接时优化，跨 crate 内联
- `opt-level = 3` — 最高性能优化
- `codegen-units = 1` — 单代码生成单元，最佳优化
- `strip = true` — 去除符号表，减小二进制体积
- `panic = "unwind"` — 栈展开，便于问题排查

> 注意：Release 编译耗时较长（10~30 分钟），首次编译会消耗较多内存（建议 1 GB+）。

#### 7.4 交叉编译（可选）

如果需要在其他平台运行，可配置交叉编译目标：

```bash
# 安装目标平台
rustup target add aarch64-unknown-linux-gnu

# 交叉编译
cargo build --release --target aarch64-unknown-linux-gnu
```

#### 7.5 编译产物

编译完成后二进制文件位于：

```
target/debug/Nakamasa-Ichika      # 开发模式
target/release/Nakamasa-Ichika    # 生产模式
target/dev-fast/Nakamasa-Ichika   # dev-fast 模式
```

---

### 八、前端编译

#### 8.1 安装前端依赖

```bash
cd view

# 使用 npm 安装
npm install

# 或使用 yarn（项目包含 yarn.lock）
yarn install
```

> 注意：`node_modules` 中包含大量依赖，安装可能需要几分钟。如果遇到网络问题，可配置国内镜像：
> ```bash
> npm config set registry https://registry.npmmirror.com
> ```

#### 8.2 开发模式运行

```bash
cd view
npm run dev
```

开发模式特点：
- 启动 Vite 开发服务器，默认端口 `8888`
- 自动启用 HTTPS（自签名证书，基于 `@vitejs/plugin-basic-ssl`）
- 热模块替换（HMR），修改代码即时生效
- API 请求自动代理到后端 `https://127.0.0.1:8080`

前端环境变量（`.env.development`）：

```bash
VITE_APP_ENV=development
VITE_APP_OPEN_PROXY=true              # 开启 API 代理
VITE_APP_BASE_URL=https://127.0.0.1:8080  # 后端地址
VITE_APP_PROXY_PREFIX=/api            # 代理路径前缀
VITE_APP_MOCK_MODE=off                # Mock 模式（off/demo/develop）
```

#### 8.3 生产模式构建

```bash
cd view
npm run build
```

构建产物位于 `view/dist/` 目录，为纯静态文件，可直接部署到 Nginx 或由后端嵌入服务。

Vite 构建优化（已预配置）：
- 代码分割（`manualChunks`），按依赖分组
- Tree Shaking，移除未使用代码
- CSS 提取与压缩
- 资源哈希命名，支持长期缓存

#### 8.4 预览构建结果

```bash
cd view
npm run preview
```

#### 8.5 前端嵌入后端（推荐）

后端通过 `rust-embed` 在编译时将前端 `dist/` 目录嵌入二进制文件，实现单文件部署：

1. 先完成前端构建：

```bash
cd view
npm run build
# 产物在 view/dist/ 中
```

2. 将构建产物复制到后端静态文件目录：

```bash
# 复制到 Nakamasa-Ichika/static/admin/ 目录
# 具体路径取决于 rust-embed 配置
cp -r view/dist/* ../Nakamasa-Ichika/static/admin/
```

3. 编译后端（会自动嵌入前端文件）：

```bash
cd ..
cargo build --release
```

4. 最终只需一个二进制文件即可运行，无需额外部署前端。

---

### 九、部署与运行

#### 9.1 准备运行环境

```bash
# 1. 确保 MySQL 和 Redis 已启动
sudo systemctl status mysql
sudo systemctl status redis

# 2. 准备 GeoIP 数据库
# 将 GeoLite2-City.mmdb 放置在项目根目录（与二进制同目录）
cp GeoLite2-City.mmdb /path/to/deploy/

# 3. 准备运行目录
mkdir -p /path/to/deploy/data/upload   # 上传文件目录
mkdir -p /path/to/deploy/log           # 日志目录
```

#### 9.2 首次运行（安装向导）

```bash
# 直接运行（不创建 config.yaml，进入安装模式）
cd /path/to/deploy
./Nakamasa-Ichika
```

服务启动后，浏览器访问：

```
https://127.0.0.1:8080/admin/install
```

按页面引导完成以下配置：
1. 数据库连接信息（主机 / 端口 / 用户名 / 密码 / 数据库名）
2. Redis 连接信息
3. 管理员账户设置（用户名 / 密码）
4. 应用基础信息

系统自动完成：
- 生成 `config.yaml` 配置文件
- 初始化数据库表结构
- 创建管理员账户
- 跳转到管理后台登录页

#### 9.3 日常运行

```bash
# 前台运行（开发/调试）
./Nakamasa-Ichika

# 后台运行（生产部署）
nohup ./Nakamasa-Ichika > log/app.log 2>&1 &

# 使用 systemd 管理（推荐生产环境）
```

systemd 服务文件示例（`/etc/systemd/system/nakamasa.service`）：

```ini
[Unit]
Description=Nakamasa-Ichika Service
After=network.target mysql.service redis.service

[Service]
Type=simple
User=nakamasa
WorkingDirectory=/opt/nakamasa
ExecStart=/opt/nakamasa/Nakamasa-Ichika
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

```bash
# 启用并启动服务
sudo systemctl daemon-reload
sudo systemctl enable nakamasa
sudo systemctl start nakamasa

# 查看状态
sudo systemctl status nakamasa

# 查看日志
journalctl -u nakamasa -f
```

#### 9.4 Nginx 反向代理（可选）

如需通过域名访问或配置 SSL 证书：

```nginx
server {
    listen 443 ssl http2;
    server_name your-domain.com;

    ssl_certificate     /path/to/ssl.crt;
    ssl_certificate_key /path/to/ssl.key;

    location / {
        proxy_pass https://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

---

### 十、验证安装

```bash
# 检查健康状态
curl -k https://127.0.0.1:8080/api/health

# 访问管理后台
# 浏览器打开 https://127.0.0.1:8080/admin/
# 使用安装时设置的管理员账户登录
```

---

### 十一、常见问题

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| `cargo build` 报 openssl 错误 | 缺少 OpenSSL 开发库 | `sudo apt install libssl-dev`（Ubuntu）或 `brew install openssl`（macOS） |
| 编译内存不足 (OOM) | Release 编译 LTO 消耗内存 | 增加交换空间或使用 `--profile dev-fast` 代替 |
| 前端 `npm install` 失败 | 网络问题 / Python 缺失 | 配置国内镜像：`npm config set registry https://registry.npmmirror.com` |
| 访问后端 API 证书错误 | 自签名证书 | 浏览器忽略证书警告，或配置 `config.yaml` 使用正规证书 |
| 数据库连接失败 | 密码错误 / 未启动 | 检查 MySQL 状态、密码正确性；如密码含 `enc:` 前缀则为加密格式 |
| Redis 连接失败 | 未启动 / 端口错误 | `redis-cli ping` 验证，检查 `config.yaml` 中 redis 配置 |
| GeoIP 查询无结果 | 缺少 mmdb 文件 | 将 `GeoLite2-City.mmdb` 放在运行目录 |
| Termux 下 V8 编译失败 | V8 不支持 Android | 已自动切换为 QuickJS，无需干预 |
| 前端代理 502 | 后端未启动 | 先启动后端 `cargo run`，再启动前端 `npm run dev` |

---

## 配置说明

配置文件 `config.yaml` 主要配置项：

```yaml
app:
  host: http://127.0.0.1:8080   # 应用地址
  code: "..."                    # 加密密钥（用于配置加密）
  upload_dir: ./data/upload      # 上传目录
  upload_size: 2                 # 上传大小限制 (MB)
  cache: false                   # 是否启用缓存
  ver: 3.3                       # 配置版本

server:
  port: 8080                     # HTTP 端口
  tls_enabled: true              # 是否启用 HTTPS

mysql:
  host: 127.0.0.1
  port: 3306
  user: root
  password: "123456"             # 支持加密: enc:xxxxxx
  dbname: ce
  prefix: u_                     # 表前缀
  max_open_conns: 150
  max_idle_conns: 20

redis:
  host: 127.0.0.1
  port: 6379
  password: ""                   # 支持加密
  db: 0
  prefix: re_                    # Key 前缀

i18n:
  default_language: "zh-CN"
  supported_languages: ["zh-CN", "en", "ja"]

log:
  path: ./log
  level: debug
```

---

## Workspace 结构

项目使用 Cargo Workspace 管理三个 crate：

| Crate | 类型 | 说明 |
|-------|------|------|
| `Nakamasa-Ichika` | 二进制（可执行） | 主应用，包含所有业务逻辑 |
| `Nakamasa-utils` | 库 | 工具库：JWT / GeoIP / 缓存 / 加密 |
| `Nakamasa-proc` | 过程宏库 | `#[route]` 路由宏、中间件宏等编译期代码生成 |

---

## 开发规范

### Rust 代码

- 使用 `#[route(GET, "/path")]` 过程宏定义路由（由 Nakamasa-proc 提供）
- 错误处理统一使用 `anyhow::Result`
- Handler 参数通过 Salvo 的 `extract` 自动注入
- 中间件实现 `Handler` trait
- 插件通过 trait 接口抽象，支持热插拔

### 前端代码

- Vue 3 Composition API（`<script setup>`）
- 组件命名：`sa-` 前缀（如 `sa-table`、`sa-icon`）
- API 请求封装在 `src/api/` 目录，按模块分文件
- Pinia Store 按功能分模块，支持持久化
- 自定义指令：`v-auth`（权限）、`v-role`（角色）、`v-copy`（复制）
- 多皮肤支持（`src/style/skins/`）

---

## 特殊说明

1. **Android / Termux 环境**：当前运行环境为 Termux Android，云函数使用 QuickJS 运行时（非 V8），通过 `cfg(target_os = "android")` 条件编译自动切换
2. **GeoIP 数据库**：需要放置 `GeoLite2-City.mmdb` 文件在项目根目录
3. **HTTPS 证书**：开发模式默认使用编译时嵌入的自签名证书；生产环境支持在 `config.yaml` 中指定自定义证书路径
4. **前端代理**：开发环境通过 Vite proxy 代理 `/api` 请求到后端 `https://127.0.0.1:8080`
5. **数据库兼容**：SQLx 同时支持 MySQL / PostgreSQL / SQLite，默认使用 MySQL

---

## 许可证

MIT License
