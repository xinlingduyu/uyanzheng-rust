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
- 使用 `#![allow(...)]` 在开发阶段抑制警告
- 模块使用文档注释说明架构
- Handler 使用 `#[route]` 宏定义路由
- 错误处理使用 `anyhow::Result`

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
cargo run                    # 开发运行
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
4. **代理配置**: 前端通过 Vite proxy 代理到后端 API
5. **疑问**: 如果有不了解的调用提问工具进行提问