# Nakamasa-Ichika

高性能用户认证与应用管理全栈平台 —— Rust + Salvo 后端，Vue 3 + Arco Design 前端。

---

## 快速导航

| 文档 | 说明 |
|------|------|
| [📖 项目架构](docs/agent.md) | 项目结构、模块划分、开发规范 |
| [⚡ 构建与运行](docs/CLI_USAGE.md) | 后端/前端启动、配置、CLI 命令 |
| [☁️ 云函数开发](docs/CLOUD_FUNCTION.md) | QuickJS 运行时 API、Db/Redis/Http 操作 |
| [🤖 AI 对话接口](docs/AI_CHAT.md) | 多 Provider 集成、流式响应、错误处理 |
| [📦 高性能缓存](./Nakamasa-utils/HIGH_PERF_CACHE.md) | 分层缓存、ShardedCacheV2 实现 |

---

## 技术栈

| 层 | 技术 |
|----|------|
| **后端** | Rust 2024 · Salvo · SQLx · QuickJS · Redis |
| **前端** | Vue 3 · Arco Design · Vite 5 · ECharts |
| **数据库** | MySQL 5.7+ / 8.0 · Redis 6.0+ |
| **部署** | 支持 HTTP/HTTPS/QUIC，跨平台编译 |

---

## 核心功能

```
用户认证 ── 账号/手机/邮箱/卡密/OAuth2.0
多应用   ── 单实例多应用，独立配置隔离
代理系统 ── 分组管理、推广分成、自动结算
支付集成 ── 支付宝/微信/捷付，多渠道热插拔
云函数   ── QuickJS 运行时，Db/Redis/Http 内建 API
加解密   ── AES/DES/RC4/RSA 跨平台纯 Rust 实现
```

---

## 项目结构

```
web/
├── Nakamasa-Ichika/    # 后端主应用
├── Nakamasa-utils/     # 工具库（JWT / GeoIP / 高性能缓存）
├── Nakamasa-Ai/        # AI 多 Provider 客户端
├── Nakamasa-proc/      # 过程宏库
├── view/               # 前端管理后台
└── docs/               # 开发文档
```

---

## 快速启动

```bash
# 后端
cd Nakamasa-Ichika && cargo run

# 前端
cd view && npm install && npm run dev
```

首次运行访问 `/admin/install` 完成安装配置。

---

## 相关资源

| 文件 | 内容 |
|------|------|
| [docs/CLI_USAGE.md](./docs/CLI_USAGE.md) | 命令行使用详情 |
| [docs/CLOUD_FUNCTION.md](./docs/CLOUD_FUNCTION.md) | 云函数开发手册 |

---

> 代码规模：后端 Rust ≈ 60,000 行 · 前端 Vue/JS ≈ 42,000 行