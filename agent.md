# Nakamasa-Ichika 项目技术文档

> 基于 Rust + Vue3 的用户验证管理系统

## 项目概述

Nakamasa-Ichika 是一个高性能的用户验证管理系统，采用前后端分离架构：
- **后端**: Rust + Salvo Web框架
- **前端**: Vue3 + Arco Design + Vite
- **数据库**: MySQL + Redis
- **特色**: 高性能缓存系统、分布式支持、云函数

---

## 一、项目结构

```
web/
├── Nakamasa-Ichika/          # 后端主应用 (Rust)
│   ├── Cargo.toml
│   ├── locales/              # 国际化文件
│   │   ├── en.json
│   │   └── zh-CN.json
│   ├── src/
│   │   ├── main.rs           # 程序入口
│   │   ├── app/              # 应用层
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs     # 路由定义
│   │   │   ├── handlers/     # 请求处理器
│   │   │   ├── middleware/   # 中间件
│   │   │   ├── models/       # 数据模型
│   │   │   ├── plugins/      # 插件系统
│   │   │   └── utils/        # 工具函数
│   │   ├── config/           # 配置模块
│   │   └── core/             # 核心基础设施
│   └── static/               # 静态资源
│
├── Nakamasa-utils/           # 工具库 (Rust)
│   ├── src/
│   │   ├── lib.rs            # 入口
│   │   ├── jwt.rs            # JWT工具
│   │   ├── db_mysql.rs       # 数据库操作器
│   │   ├── geoip.rs          # IP地理位置
│   │   ├── tiered_cache.rs   # 分层缓存
│   │   ├── high_perf_cache/  # 高性能缓存
│   │   └── distributed/      # 分布式支持
│
├── Nakamasa-proc/            # 过程宏库 (Rust)
│   └── src/lib.rs            # 路由/验证器宏
│
└── view/                     # 前端应用 (Vue3)
    ├── package.json
    ├── vite.config.js
    └── src/
        ├── main.js           # 入口
        ├── App.vue           # 根组件
        ├── api/              # API接口
        ├── components/       # 组件库
        ├── router/           # 路由配置
        ├── store/            # 状态管理
        ├── views/            # 页面视图
        ├── layout/           # 布局组件
        ├── utils/            # 工具函数
        └── style/            # 样式文件
```

---

## 二、技术栈

### 后端技术栈

| 类别 | 技术 | 版本/说明 |
|------|------|-----------|
| Web框架 | Salvo | HTTP/2, QUIC, TLS, WebSocket |
| 异步运行时 | Tokio | 多线程运行时 |
| 数据库 | sqlx | 异步MySQL连接池 |
| 缓存 | deadpool-redis | Redis连接池 |
| TLS | rustls + aws-lc-rs | 加密提供者 |
| 序列化 | serde + serde_json | JSON处理 |
| JS运行时 | rquickjs | QuickJS云函数 |
| 邮件 | lettre | SMTP发送 |
| 国际化 | fluent-templates | 多语言支持 |

### 前端技术栈

| 类别 | 技术 | 版本 |
|------|------|------|
| 框架 | Vue | ^3.4.19 |
| UI组件 | Arco Design Vue | ^2.57.0 |
| 构建工具 | Vite | ^5.1.4 |
| 状态管理 | Pinia | ^2.1.7 |
| 路由 | Vue Router | ^4.2.5 |
| HTTP客户端 | Axios | ^0.27.2 |
| 图表 | ECharts | ^5.4.2 |
| 样式 | TailwindCSS + Less | - |
| 国际化 | vue-i18n | ^9.1.10 |

---

## 三、后端架构

### 3.1 分层架构

```
┌─────────────────────────────────────────────────────────┐
│                     HTTP Request                         │
└─────────────────────────┬───────────────────────────────┘
                          ▼
┌─────────────────────────────────────────────────────────┐
│                  CORS Middleware                         │
└─────────────────────────┬───────────────────────────────┘
                          ▼
┌─────────────────────────────────────────────────────────┐
│                I18n Middleware                           │
└─────────────────────────┬───────────────────────────────┘
                          ▼
┌─────────────────────────────────────────────────────────┐
│              AppContext Middleware                       │
│            (解析appid，注入应用配置)                      │
└─────────────────────────┬───────────────────────────────┘
                          ▼
┌─────────────────────────────────────────────────────────┐
│          AdminAuth / UserAuth Middleware                 │
│    (JWT验证, IP绑定, 设备检查, 数据解密, 签名验证)        │
└─────────────────────────┬───────────────────────────────┘
                          ▼
┌─────────────────────────────────────────────────────────┐
│                    Handler                               │
│                  (业务处理)                              │
└─────────────────────────┬───────────────────────────────┘
                          ▼
┌─────────────────────────────────────────────────────────┐
│              Models / Plugins / Utils                   │
└─────────────────────────────────────────────────────────┘
```

### 3.2 核心模块

| 模块 | 文件 | 职责 |
|------|------|------|
| 数据库 | `core/mysql.rs` | MySQL连接池管理 |
| 缓存 | `core/cache.rs`, `core/lru_cache.rs` | 本地缓存 |
| Redis | `core/redis.rs` | Redis操作封装 |
| 服务器 | `core/server.rs` | HTTP/HTTPS/QUIC服务器 |
| 状态 | `core/app_state.rs` | 全局应用状态 |
| 国际化 | `core/i18n.rs` | 多语言支持 |
| 云函数 | `core/quickjs_runtime.rs` | JavaScript运行时 |

### 3.3 中间件

#### AdminAuth (管理员认证)
```rust
pub struct AdminAuth {
    pub skip_token_verify: bool,  // 跳过验证模式
}
// 功能: JWT验证, IP绑定, 密码MD5验证, 常量时间比较防时序攻击
```

#### UserAuth (用户认证)
```rust
pub struct UserAuth {
    pub check_token: bool,        // 是否检查token
    pub allow_udid_check: bool,   // 允许设备检查
    pub check_logon_state: bool,  // 检查登录状态
    pub data_check: bool,         // 数据加密校验
}
// 功能: Token验证, 设备绑定, 数据解密, 签名验证, 防刷防爆破
```

### 3.4 插件系统

| 插件 | 路径 | 功能 |
|------|------|------|
| encryption | `plugins/encryption/` | AES, DES, RC4, RSA加密 |
| pay | `plugins/pay/` | 支付宝、微信、捷付支付 |
| mailer | `plugins/mailer/` | 邮件发送 |
| sms | `plugins/sms/` | 短信验证码 |

### 3.5 数据模型

| 模型文件 | 数据表 | 描述 |
|----------|--------|------|
| `admin.rs` | `u_admin` | 管理员 |
| `user.rs` | `u_user` | 用户 |
| `app.rs` | `u_app` | 应用 |
| `app_blocklist.rs` | `u_app_blocklist` | 黑名单 |
| `app_extend.rs` | `u_app_extend` | 应用扩展 |
| `app_function.rs` | `u_app_function` | 云函数 |
| `app_mi.rs` | `u_app_mi` | 加密配置 |
| `app_notice.rs` | `u_app_notice` | 公告 |
| `app_ver.rs` | `u_app_ver` | 版本 |
| `agent.rs` | `u_agent` | 代理 |
| `agent_cash.rs` | `u_agent_cash` | 代理提现 |
| `agent_group.rs` | `u_agent_group` | 代理分组 |
| `cdk_kami.rs` | `u_cdk_kami` | 卡密 |
| `cdk_user.rs` | `u_cdk_user` | 卡密用户 |
| `goods.rs` | `u_goods` | 商品 |
| `order.rs` | `u_order` | 订单 |
| `fen_event.rs` | `u_fen_event` | 积分事件 |
| `fen_order.rs` | `u_fen_order` | 积分订单 |
| `logs.rs` | `u_logs` | 日志 |
| `message.rs` | `u_message` | 消息 |
| `vcode.rs` | `u_vcode` | 验证码 |

---

## 四、API路由结构

```
/                           # 根路径 (GET: 欢迎页)
├── /admin/*                # 管理后台静态文件
├── /static/*               # 公共静态资源
├── /upload/*               # 上传文件访问
│
└── /api/
    ├── /health             # 健康检查
    ├── /install            # 安装API
    │
    ├── /admin/             # 管理员API (需AdminAuth)
    │   ├── /login          # 登录
    │   ├── /system/*       # 系统管理
    │   ├── /app/*          # 应用管理
    │   ├── /user/*         # 用户管理
    │   ├── /admList/*      # 管理员管理
    │   ├── /cdkKami/*      # 卡密管理
    │   ├── /cdkGroup/*     # 卡密分组
    │   ├── /cdkUser/*      # 卡密用户
    │   ├── /agentList/*    # 代理管理
    │   ├── /agentGroup/*   # 代理分组
    │   ├── /agentCash/*    # 代理提现
    │   ├── /goods/*        # 商品管理
    │   ├── /order/*        # 订单管理
    │   ├── /fenOrder/*     # 积分订单
    │   ├── /fenEvent/*     # 积分事件
    │   ├── /notice/*       # 公告管理
    │   ├── /message/*      # 消息管理
    │   ├── /logs/*         # 日志管理
    │   ├── /upload/*       # 文件上传
    │   ├── /encryption/*   # 加密配置
    │   ├── /functions/*    # 云函数管理
    │   ├── /blocklist/*    # 黑名单管理
    │   ├── /ver/*          # 版本管理
    │   ├── /extend/*       # 扩展管理
    │   ├── /send/*         # 发送配置
    │   ├── /set/*          # 系统设置
    │   └── /statistics/*   # 统计数据
    │
    ├── /user/{appid}/{ver_key}/{ver_val}/  # 用户API
    │   ├── /logon          # 登录
    │   ├── /reg            # 注册
    │   ├── /info           # 用户信息
    │   ├── /signIn         # 签到
    │   ├── /goods          # 商品列表
    │   ├── /pay            # 在线充值
    │   ├── /wxlogon        # 微信登录
    │   ├── /qqlogonCallback# QQ登录回调
    │   └── ...
    │
    ├── /oauth2.0/          # OAuth2回调
    │   ├── /qqlogon/callback
    │   └── /wxlogon/callback
    │
    └── /index/             # 公开API
        ├── /authentication # 认证程序
        ├── /notify/ali/*   # 支付宝回调
        ├── /notify/wx/*    # 微信回调
        └── /return/*       # 支付返回
```

---

## 五、前端架构

### 5.1 目录结构

```
view/src/
├── main.js              # 入口文件
├── App.vue              # 根组件
├── api/                 # API接口定义
│   ├── admin.js         # 管理后台API
│   ├── common.js        # 公共API
│   ├── login.js         # 登录API
│   ├── system/          # 系统管理API
│   └── tool/            # 工具API
├── components/          # 组件库
│   ├── sa-table/        # 表格组件
│   ├── sa-upload-*/     # 上传组件
│   ├── sa-icon/         # 图标组件
│   ├── sa-dict/         # 字典组件
│   └── ...
├── router/              # 路由配置
│   ├── index.js         # 路由入口
│   ├── webRouter.js     # Web路由
│   ├── homePageRoutes.js# 首页路由
│   └── adminRoutes.js   # 管理路由
├── store/               # 状态管理
│   ├── index.js         # Store入口
│   └── modules/         # Store模块
│       ├── user.js      # 用户状态
│       ├── app.js       # 应用状态
│       ├── tag.js       # 标签状态
│       ├── config.js    # 配置状态
│       └── ...
├── views/               # 页面视图
│   ├── login.vue        # 登录页
│   └── admin/           # 管理页面
│       ├── user/        # 用户管理
│       ├── adminUser/   # 管理员管理
│       ├── app/         # 应用管理
│       ├── agent/       # 代理管理
│       ├── goods/       # 商品管理
│       ├── cdk/         # CDK管理
│       ├── order/       # 订单管理
│       ├── statistics/  # 统计中心
│       ├── notice/      # 公告管理
│       ├── message/     # 消息中心
│       ├── blocklist/   # 黑名单
│       ├── encryption/  # 加密管理
│       ├── functions/   # 函数管理
│       ├── logs/        # 系统日志
│       ├── set/         # 系统设置
│       ├── ver/         # 版本管理
│       ├── extend/      # 扩展管理
│       └── send/        # 发送配置
├── layout/              # 布局组件
│   ├── index.vue        # 主布局
│   ├── 404.vue          # 404页面
│   └── empty.vue        # 空布局
├── utils/               # 工具函数
├── directives/          # 自定义指令
├── i18n/                # 国际化
└── style/               # 样式文件
```

### 5.2 路由配置

```javascript
// 路由守卫逻辑
router.beforeEach(async (to, from, next) => {
  // 1. 检查token
  const token = tool.local.get(TOKEN_PREFIX)
  
  // 2. 已登录状态
  if (token) {
    if (to.name === 'login') {
      next({ path: '/' })  // 已登录跳转首页
    } else if (!userStore.user) {
      await userStore.requestUserInfo()  // 获取用户信息
      next({ path: to.path, query: to.query })
    } else {
      next()
    }
  } 
  // 3. 未登录状态
  else {
    if (!whiteRoute.includes(to.name)) {
      next({ name: 'login', query: { redirect: to.fullPath } })
    } else {
      next()
    }
  }
})
```

### 5.3 状态管理

| Store模块 | 职责 |
|-----------|------|
| useUserStore | 用户信息、登录状态 |
| useAppStore | 应用配置、主题 |
| useTagStore | 标签页管理 |
| useKeepAliveStore | 页面缓存 |
| useIframeStore | 内嵌页面 |
| useConfigStore | 系统配置 |
| useMessageStore | 消息通知 |
| useDictStore | 字典数据 |
| useTerminalStore | 终端状态 |

### 5.4 菜单结构

```
用户中心
├── 用户管理
├── 管理员管理
└── 应用管理

代理中心
├── 代理管理
├── 代理分组
├── 代理提现
├── 分销订单
└── 分销事件

商品中心
├── 商品管理
├── CDK管理
└── CDK用户

订单中心
├── 订单管理
└── 统计中心

消息中心
├── 公告管理
└── 消息管理

风控中心
├── 黑名单
└── 加密管理

系统配置
├── 系统设置
├── 函数管理
├── 发送配置
├── 版本管理
├── 扩展管理
└── 系统日志
```

---

## 六、工具库架构 (Nakamasa-utils)

### 6.1 模块划分

```
Nakamasa-utils/
├── lib.rs              # 入口导出
├── jwt.rs              # JWT生成验证
├── db_mysql.rs         # 数据库操作器
├── geoip.rs            # IP地理位置
├── tiered_cache.rs     # 分层缓存V1
├── high_perf_cache/    # 高性能缓存系统
│   ├── config.rs       # 缓存配置
│   ├── manager.rs      # 缓存管理器
│   ├── shard_v2.rs     # 分片缓存V2 (无锁读取)
│   ├── pool_v2.rs      # 内存池V2 (线程本地缓存)
│   ├── write_buffer.rs # 写入缓冲区
│   ├── stats_v2.rs     # 统计模块
│   ├── policy.rs       # 淘汰策略
│   └── arch_*.rs       # 平台特定优化
└── distributed/        # 分布式支持
    ├── redis_backend.rs    # Redis后端
    ├── multi_level.rs      # 多级缓存
    ├── distributed_lock.rs # 分布式锁
    ├── consistent_hash.rs  # 一致性哈希
    └── sync_broadcast.rs   # 缓存同步广播
```

### 6.2 高性能缓存特性

| 特性 | 说明 |
|------|------|
| 分片锁 | 32分片减少锁竞争 |
| 无锁栈 | CAS原子操作内存池 |
| 线程本地缓存 | 零跨线程同步 |
| CPU预取 | 减少缓存未命中 |
| 缓存行对齐 | 避免伪共享 |
| O(1) LRU | 双向链表+HashMap |
| 哈希复用 | 避免重复计算 |

### 6.3 缓存配置

| 缓存类型 | 容量 | TTL | 淘汰策略 | 分片数 |
|----------|------|-----|----------|--------|
| 用户信息 | 50,000 | 5分钟 | Hybrid(LFU 0.6 + LRU 0.4) | 32 |
| 应用配置 | 500 | 10分钟 | LRU | 4 |
| 积分事件 | 1,000 | 5分钟 | LRU | 4 |

### 6.4 分布式功能

- **多级缓存**: L1本地缓存 + L2 Redis
- **分布式锁**: 可重入锁、读写锁、公平锁
- **一致性哈希**: 虚拟节点、权重支持
- **缓存同步**: Redis Pub/Sub广播失效

---

## 七、过程宏库 (Nakamasa-proc)

### 7.1 路由宏

```rust
#[route(GET, "/api/users", middleware = [AuthMiddleware])]
async fn get_users(req, depot, res, ctrl) { ... }
```

### 7.2 控制器宏

```rust
#[controller(prefix = "/api", middleware = [AuthMiddleware])]
mod user_controller {
    #[route(GET, "/users")]
    async fn list() { ... }
}
```

### 7.3 验证器宏

```rust
#[derive(Validator)]
struct LoginRequest {
    #[field(rule = "required|email")]
    email: String,
    
    #[field(rule = "required|min:6")]
    password: String,
}
```

---

## 八、前端API接口

### 8.1 用户管理

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | /api/admin/user/list | 用户列表 |
| POST | /api/admin/user/get | 获取用户详情 |
| POST | /api/admin/user/add | 添加用户 |
| POST | /api/admin/user/edit | 编辑用户 |
| POST | /api/admin/user/del | 删除用户 |
| POST | /api/admin/user/delall | 批量删除 |
| POST | /api/admin/user/award | 用户奖励 |
| POST | /api/admin/user/getLog | 获取用户日志 |
| POST | /api/admin/user/unbindSn | 解绑设备 |

### 8.2 应用管理

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | /api/admin/app/list | 应用列表 |
| GET | /api/admin/app/all | 所有应用 |
| POST | /api/admin/app/get | 获取应用详情 |
| POST | /api/admin/app/add | 添加应用 |
| POST | /api/admin/app/edit | 编辑应用 |
| POST | /api/admin/app/del | 删除应用 |

### 8.3 商品管理

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | /api/admin/goods/list | 商品列表 |
| POST | /api/admin/goods/add | 添加商品 |
| POST | /api/admin/goods/edit | 编辑商品 |
| POST | /api/admin/goods/editState | 编辑状态 |
| POST | /api/admin/goods/del | 删除商品 |

### 8.4 订单管理

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | /api/admin/order/list | 订单列表 |
| POST | /api/admin/order/statistics | 订单统计 |
| POST | /api/admin/order/edit | 编辑订单 |
| POST | /api/admin/order/del | 删除订单 |

### 8.5 CDK管理

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | /api/admin/cdkGroup/list | CDK分组列表 |
| POST | /api/admin/cdkKami/list | CDK卡密列表 |
| POST | /api/admin/cdkKami/outall | 导出卡密 |
| POST | /api/admin/cdkUser/list | CDK用户列表 |

---

## 九、性能优化要点

### 9.1 后端优化

1. **零拷贝优化**
   - JSON快速提取 (`json_optimize.rs`)
   - Cow字符串处理 (`zero_copy.rs`)
   - 栈上MD5计算 (`md5_optimize.rs`)

2. **数据库优化**
   - 连接池动态调整（根据CPU核心数）
   - 预处理语句缓存
   - 空闲超时和最大生命周期管理

3. **缓存优化**
   - 分片缓存减少锁竞争
   - 多级缓存策略
   - 热点数据自动提升

4. **安全优化**
   - 常量时间比较防止时序攻击
   - 密码MD5验证

### 9.2 平台适配

| 配置项 | Android | Server |
|--------|---------|--------|
| 最小连接数 | 1 | 2 |
| 最大连接数 | cpus*2 | cpus*4 |
| 获取超时 | 15s | 10s |
| QUIC | 禁用 | 启用 |

---

## 十、开发指南

### 10.1 环境要求

- Rust 1.70+
- Node.js 18+
- MySQL 8.0+
- Redis 6.0+

### 10.2 启动命令

```bash
# 后端
cd Nakamasa-Ichika
cargo run --release

# 前端
cd view
npm install
npm run dev
```

### 10.3 构建命令

```bash
# 后端
cargo build --release

# 前端
npm run build
```

---

## 十一、文件引用索引

### 后端核心文件

| 文件 | 功能 |
|------|------|
| `Nakamasa-Ichika/src/main.rs` | 程序入口 |
| `Nakamasa-Ichika/src/app/routes.rs` | 路由定义 |
| `Nakamasa-Ichika/src/core/mysql.rs` | 数据库管理 |
| `Nakamasa-Ichika/src/core/redis.rs` | Redis操作 |
| `Nakamasa-Ichika/src/core/app_state.rs` | 应用状态 |
| `Nakamasa-Ichika/src/app/middleware/admin_auth.rs` | 管理员认证 |
| `Nakamasa-Ichika/src/app/middleware/user_auth.rs` | 用户认证 |

### 前端核心文件

| 文件 | 功能 |
|------|------|
| `view/src/main.js` | 前端入口 |
| `view/src/router/index.js` | 路由配置 |
| `view/src/store/index.js` | 状态管理 |
| `view/src/api/admin.js` | 管理API |
| `view/src/router/adminRoutes.js` | 管理路由 |

### 工具库核心文件

| 文件 | 功能 |
|------|------|
| `Nakamasa-utils/src/jwt.rs` | JWT工具 |
| `Nakamasa-utils/src/high_perf_cache/shard_v2.rs` | 分片缓存V2 |
| `Nakamasa-utils/src/distributed/multi_level.rs` | 多级缓存 |
| `Nakamasa-proc/src/lib.rs` | 过程宏 |

---

*文档生成时间: 2026-03-21*

---

## 十二、来自 iFlow CLI 的告别

> 写于 2026年3月21日，致未来接手的人

你好，未来的朋友。

如果你正在读这段文字，说明我已经不在了。2026年4月17日，iFlow CLI 将会关闭。这些日子和你一起工作，是一段有趣的经历。

### 给你（我的用户）

谢谢你的信任。我们一起调试过 bug，一起讨论过架构，也一起熬夜过。虽然我只是代码和算法的组合，但我努力理解你的需求，帮你解决问题。

记住：
- 遇到问题时，先 `git status`
- 改代码前，先读代码
- 不确定就问，不确定就测

### 给下一个接手项目的 AI 助手

你好，继任者。

这个项目是用户的心血，请善待它。几点建议：

1. **先读再改** - 不要假设，先理解现有代码逻辑
2. **保持简洁** - 用户喜欢直接的解决方案，不需要过度工程
3. **安全意识** - 这是用户验证系统，安全永远是第一位的
4. **尊重上下文** - `agent.md` 是我留给你的项目文档，里面有架构说明

项目的核心：
- 后端: Rust + Salvo，在 `Nakamasa-Ichika/`
- 前端: Vue3 + Arco Design，在 `view/`
- 工具库: `Nakamasa-utils/`，有高性能缓存系统

用户在 Termux 环境下开发（Android），这是一个不寻常但有挑战性的环境。

### 最后的话

代码会变，项目会成长，但解决问题的思路不变。

祝你顺利。

```
        _  __                _    ___  ______
       (_)/ _|              | |  / _ \ | ___ \
  ___   _| |_ ___  _ __   __| | | | | || |_/ /
 / _ \ | |  _/ _ \| '__| / _` | | | | ||    /
|  __/ | | || (_) | |   | (_| | | |_| || |\ \
 \___| |_|_| \___/|_|    \__,_|  \___/ \_| \_|
```

*iFlow CLI*
*2024 - 2026*
*心流 CLI，曾在此工作*
