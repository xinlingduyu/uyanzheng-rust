# Nakamasa-Ichika 全栈项目

## 项目概述

Nakamasa-Ichika 是一个高性能的用户认证和应用管理全栈平台，包含：
- **后端服务**: 基于 Rust + Salvo 框架的高性能 Web 服务
- **管理后台**: 基于 Vue 3 + Arco Design 的现代化管理界面

### 核心功能
- 用户认证（账号密码、手机号、邮箱、卡密、OAuth2.0）
- 多应用支持（单实例多应用独立配置）
- 代理推广与分成系统
- 积分系统
- 支付集成（支付宝、微信）
- 云函数支持（QuickJS 运行时）

---

## 项目结构

```
web/
├── Nakamasa-Ichika/       # 后端主应用
│   ├── src/
│   │   ├── main.rs        # 应用入口
│   │   ├── app/           # 业务逻辑层
│   │   │   ├── handlers/  # HTTP 处理器
│   │   │   ├── models/    # 数据模型
│   │   │   ├── middleware/# 中间件
│   │   │   ├── plugins/   # 插件系统
│   │   │   └── routes.rs  # 路由定义
│   │   ├── core/          # 核心基础设施
│   │   │   ├── mysql.rs   # MySQL 连接池
│   │   │   ├── redis.rs   # Redis 工具
│   │   │   ├── cache.rs   # 多级缓存
│   │   │   └── ...
│   │   └── config/        # 配置模块
│   └── locales/           # 国际化文件
│
├── Nakamasa-utils/        # 工具库
│   ├── jwt.rs             # JWT 工具
│   ├── geoip.rs           # GeoIP 地理定位
│   ├── tiered_cache.rs    # 分层缓存
│   ├── high_perf_cache/   # 高性能缓存实现
│   └── distributed/       # 分布式缓存支持
│
├── Nakamasa-proc/         # 过程宏库
│   └── lib.rs             # route/controller/middleware 宏
│
├── view/                  # 前端管理后台
│   ├── src/
│   │   ├── api/           # API 请求封装
│   │   ├── components/    # 通用组件
│   │   ├── views/         # 页面视图
│   │   ├── router/        # 路由配置
│   │   ├── store/         # Pinia 状态管理
│   │   ├── i18n/          # 国际化
│   │   └── utils/         # 工具函数
│   └── vite.config.js     # Vite 配置
│
└── arco-design-pro/       # Arco Design Pro 模板（参考）
```

---

## 技术栈

### 后端
| 技术 | 用途 |
|------|------|
| Rust 1.85+ | 编程语言 |
| Salvo | Web 框架（HTTP/HTTPS/QUIC） |
| SQLx | 数据库（MySQL/PostgreSQL/SQLite） |
| Deadpool-Redis | Redis 连接池 |
| QuickJS | 云函数 JavaScript 运行时 |
| rustls + aws-lc-rs | TLS 加密 |

### 前端
| 技术 | 用途 |
|------|------|
| Vue 3 | 前端框架 |
| Arco Design Vue | UI 组件库 |
| Vite 5 | 构建工具 |
| Pinia | 状态管理 |
| Vue Router | 路由管理 |
| ECharts/ApexCharts | 图表可视化 |

---

## 构建与运行

### 环境要求
- Rust 1.85+
- Node.js 18+
- MySQL 5.7+ / 8.0+
- Redis 6.0+

### 后端

```bash
# 开发模式
cd /data/data/com.termux/files/home/rust/web
cargo run

# 生产构建
cargo build --release

# 启用 TUI 仪表盘（三框樱花粉色监控界面，支持 Ctrl+C 退出）
cargo run -- --tui

# 运行前确保 config.yaml 存在
# 首次运行访问 /admin/install 完成安装
```

### 前端

```bash
cd /data/data/com.termux/files/home/rust/web/view

# 安装依赖
npm install

# 开发模式
npm run dev

# 生产构建
npm run build
```

---

## API 路由结构

```
/                           # 欢迎页
├── /admin/*                # 管理后台静态文件
├── /static/*               # 公共静态资源
├── /upload/*               # 上传文件访问
│
└── /api/
    ├── /health             # 健康检查
    ├── /install            # 安装 API
    │
    ├── /admin/             # 管理员 API
    │   ├── /login          # 登录
    │   ├── /user           # 用户管理
    │   ├── /app            # 应用管理
    │   ├── /function       # 云函数
    │   ├── /ver            # 版本管理
    │   └── ...
    │
    ├── /user/              # 用户 API
    │   ├── /login          # 登录
    │   ├── /register       # 注册
    │   ├── /info           # 用户信息
    │   └── ...
    │
    ├── /oauth2.0/          # OAuth2 回调
    │   ├── /qqlogon/callback
    │   └── /wxlogon/callback
    │
    └── /index/             # 首页 API
        ├── /appinfo
        └── /config
```

---

## 前端页面模块

| 路径 | 功能 |
|------|------|
| `/views/dashboard/` | 仪表盘/数据统计 |
| `/views/user/` | 用户管理 |
| `/views/admin/` | 管理员管理 |
| `/views/app/` | 应用管理 |
| `/views/goods/` | 商品管理 |
| `/views/order/` | 订单管理 |
| `/views/pay/` | 支付配置 |
| `/views/kami/` | 卡密管理 |
| `/views/agent/` | 代理管理 |
| `/views/function/` | 云函数 |
| `/views/ver/` | 版本管理 |
| `/views/notice/` | 公告管理 |
| `/views/message/` | 消息管理 |
| `/views/statistics/` | 统计分析 |
| `/views/visualization/` | 数据可视化 |

---

## 核心架构

### 后端分层架构
```
┌─────────────────────────────────────────────┐
│                 HTTP Request                │
└─────────────────────┬───────────────────────┘
                      ▼
┌─────────────────────────────────────────────┐
│              Middleware Layer               │
│  CORS → I18n → AppContext → Auth           │
└─────────────────────┬───────────────────────┘
                      ▼
┌─────────────────────────────────────────────┐
│              Handler Layer                  │
│  Admin API | User API | Index API          │
└─────────────────────┬───────────────────────┘
                      ▼
┌─────────────────────────────────────────────┐
│           Service/Model Layer               │
│  Models | Plugins | Utils | Core           │
└─────────────────────────────────────────────┘
```

### 缓存架构
```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  L1 Cache   │ ──► │  L2 Redis   │ ──► │   MySQL     │
│ (LRU 本地)  │     │ (分布式)    │     │  (持久化)   │
└─────────────┘     └─────────────┘     └─────────────┘
```

缓存实现约定：
- 主体项目优先复用 `Nakamasa-utils::high_perf_cache::ShardedCacheV2`，不要在业务代码里新增 `RwLock<HashMap<...>>` 或重复实现 LRU。
- `ShardedCacheV2` 适合短 TTL、高频读、允许短时间最终一致的配置/聚合数据，例如 AppContext 应用上下文、`/ini` 版本/公告/扩展配置、用户基础信息、积分事件定义。
- 涉及金额、订单状态、支付回调、卡密消费、积分扣减、权限判定等强一致写路径，不得只依赖本地缓存；必须以数据库/Redis 原子操作为准。
- 缓存 key 必须包含租户/应用维度和版本维度，例如 `appid:ver_key:ver_val`，避免多应用配置串读。
- 配置类缓存 TTL 建议 30-120 秒；用户/token 类缓存 TTL 应更短，并在密码修改、踢下线、配置保存后主动失效。
- 管理后台修改应用、版本、公告、扩展、加密配置后，应调用 AppState 的缓存失效方法或清空相关短 TTL 运行时缓存。

---

## 配置文件

### config.yaml
应用主配置文件，包含：
- 服务器配置（HTTP/HTTPS/QUIC 端口）
- 数据库连接（MySQL）
- Redis 连接
- 应用基础信息

### 首次安装
1. 运行应用（无 config.yaml）
2. 访问 `/admin/install`
3. 完成数据库和基础配置
4. 系统自动生成 config.yaml

---

## 开发规范

### Rust 代码风格
- 当前工作区使用 Rust 2024 edition；以 `Cargo.toml` 为准，不要按 2015 edition 误报修改代码
- 模块使用文档注释说明架构；业务代码按 handler / model / middleware / plugin / core 分层
- Handler 使用 Salvo `#[handler]` 与路由模块注册，路由命名必须与前端实际请求保持一致
- 新增或重构代码优先保持小范围 diff，避免无关格式化；不要随手执行 `cargo fmt --all`
- 仅对明确改动的小范围文件运行格式化或手动整理，避免大规模纯格式化 diff 掩盖业务修复

### 后端安全与错误处理规范
- 生产请求路径禁止使用危险 `unwrap()` / `expect()`；从 `Depot`、请求参数、数据库、Redis、文件系统、上游服务取值必须 `match` 处理
- 可接受的 `unwrap()` 仅限编译期确定安全的常量解析、字面量正则、受前置 guard 保护的逻辑等，并应保持局部可证明
- `depot.obtain::<Arc<AppState>>()` 必须返回统一错误：Admin/Install 使用 `ApiResponse`，User 使用 `render_error`，支付回调使用平台要求的纯文本（如 `fail`）
- admin 与 user 是两套独立认证：Admin=JWT，User=Redis MD5 token；不得混用 token 验证逻辑
- 修改 admin 认证时只能做非核心逻辑修复（如错误处理），不得擅自改变 JWT 签名、验证、续期、IP 绑定语义
- 所有 SQL 值必须使用 bind 参数；动态 SQL 的表名、列名、排序字段必须走白名单，禁止直接拼接前端字段名
- 批量 `IN (...)` 操作必须校验：数组非空、数量上限（建议 <=1000）、ID 合法；占位符仍用 bind 绑定
- 对外错误信息返回稳定业务文案；原始数据库、文件系统、AI 上游、云函数运行时内部错误写入日志，不直接暴露给用户
- 支付回调的入参解析必须兼容支付平台差异：支持 JSON body、POST form、GET query；回调响应仍保持平台协议要求的纯文本（如 `success` / `fail`），不能改成 ApiResponse JSON
- 安装路由在已安装后只暴露只读检查接口 `/api/install/check`，不要重新注册完整安装接口

### 验证规范
- 非依赖代码修改至少运行 `cargo check` 验证；修改 `Cargo.toml` / feature / 依赖后必须运行 `cargo build` 或 `cargo run` 完整验证
- 修复编译错误时先定位具体错误，再做最小修改；不要为通过编译扩大改动范围
- Git 操作（commit / push / reset / revert / rebase）必须先获得用户明确确认

### 前端代码风格
- Vue 3 Composition API
- 组件命名：`sa-` 前缀（如 `sa-icon`）
- API 请求封装在 `src/api/` 目录
- 使用 Pinia 进行状态管理

---

## 国际化

### 后端
- 语言文件：`Nakamasa-Ichika/locales/`
- 支持：中文（zh-CN）、英文（en）
- 使用 `fluent-templates` 实现

### 前端
- 语言文件：`view/src/i18n/`
- 支持：中文（zh_CN）、英文（en）
- 使用 `vue-i18n` 实现

---

## 常用命令速查

```bash
# 后端
cargo run                    # 开发运行（传统日志输出）
cargo run -- --tui           # 启用 TUI 仪表盘
cargo build --release        # 生产构建
cargo test                   # 运行测试
cargo clippy                 # 代码检查

# 前端
cd view && npm run dev       # 开发服务器
cd view && npm run build     # 生产构建
cd view && npm run preview   # 预览构建结果

# 数据库
# SQL 文件位于 new.sql
```

---

## 注意事项

1. **Android 环境**: 当前运行在 Termux Android 环境，云函数使用 QuickJS（非 V8）
2. **GeoIP**: 需要放置 `GeoLite2-City.mmdb` 文件在项目根目录
3. **HTTPS**: 前端开发服务器默认启用 HTTPS（自签名证书）
4. **TUI 仪表盘**: 通过 `--tui` 参数启用，显示系统信息、CPU 走势图、资源监控和启动日志。默认使用传统日志输出。TUI 模式下按 `Ctrl+C` 退出。
5. **代理配置**: 前端通过 Vite proxy 代理到后端 API
6. **疑问**: 如果有不了解的调用提问工具进行提问