# Nakamasa-Ichika — Full-Stack Project Context

## Project Overview

**Nakamasa-Ichika** is a high-performance user authentication and application management platform built with Rust (backend) and Vue 3 (frontend). It provides a complete admin system with multi-tenant application support, payment integration, agent/affiliate system, and cloud function execution.

### Architecture

This is a **Rust workspace** containing three crates:
- **`Nakamasa-Ichika`** — Main backend application (Salvo web framework)
- **`Nakamasa-utils`** — Shared utility library (JWT, caching, GeoIP, etc.)
- **`Nakamasa-proc`** — Procedural macros for routes/controllers/middleware

Plus a standalone **Vue 3 admin frontend** in `view/`.

### Core Features

- **User Authentication**: Username/password, phone, email, card-key, OAuth2.0 (QQ, WeChat)
- **Multi-App Support**: Single instance, multiple apps with isolated configurations
- **Agent System**: Affiliate promotion and revenue sharing
- **Points System**: Flexible point rules and transactions
- **Payment Integration**: Alipay, WeChat Pay
- **Cloud Functions**: QuickJS runtime for serverless JavaScript execution
- **Tiered Caching**: L1 (LRU local) → L2 (Redis) → MySQL

---

## Directory Structure

```
web/
├── Nakamasa-Ichika/       # Backend main application
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── app/           # Business logic (handlers, models, middleware, plugins, routes)
│   │   ├── core/          # Infrastructure (DB, Redis, cache, server)
│   │   └── config/        # Configuration module
│   ├── locales/           # i18n files (zh-CN, en)
│   ├── certs/             # TLS certificates
│   └── static/            # Static assets
│
├── Nakamasa-utils/        # Shared utility library
│   └── src/               # JWT, GeoIP, caching implementations
│
├── Nakamasa-proc/         # Procedural macros
│   └── src/               # route/controller/middleware macros
│
├── view/                  # Vue 3 Admin frontend
│   ├── src/
│   │   ├── api/           # API request wrappers
│   │   ├── components/    # Reusable components
│   │   ├── views/         # Page views
│   │   ├── router/        # Route config
│   │   ├── store/         # Pinia state
│   │   ├── i18n/          # Internationalization
│   │   └── utils/         # Utility functions
│   └── vite.config.js
│
├── arco-design-pro/       # Arco Design Pro template (reference)
├── data/                  # Runtime data
├── target/                # Build artifacts
├── Cargo.toml             # Workspace root
├── Cargo.lock
└── new.sql                # Database schema
```

---

## Technology Stack

### Backend (Rust)

| Technology | Purpose |
|------------|---------|
| Rust 1.85+ (Edition 2024) | Programming language |
| Salvo | Web framework (HTTP/2, QUIC support) |
| SQLx 0.8 | Database driver (MySQL, PostgreSQL, SQLite) |
| deadpool-redis 0.22 | Redis connection pool |
| rquickjs 0.8 | Cloud function JS runtime (QuickJS) |
| tokio | Async runtime |
| fluent-templates 0.9 | i18n |
| rustls + aws-lc-rs | TLS encryption |
| serde/serde_json | Serialization |
| tracing + tracing-subscriber | Logging/tracing |
| pprof 0.14 | Profiling (flamegraph support) |

### Frontend (Vue 3)

| Technology | Purpose |
|------------|---------|
| Vue 3.4 | Framework |
| Arco Design Vue 2.57 | UI component library |
| Vite 5 | Build tool |
| Pinia | State management |
| Vue Router 4 | Routing |
| Vue i18n 9 | Internationalization |
| ECharts 5 + ApexCharts | Data visualization |
| Monaco Editor | Code editor (cloud functions) |
| wangEditor / md-editor-v3 | Rich text / Markdown editors |
| Tailwind CSS 3 | Utility CSS |
| Axios | HTTP client |
| dayjs | Date handling |

---

## Building & Running

### Prerequisites

- Rust 1.85+
- Node.js 18+
- MySQL 5.7+ / 8.0+
- Redis 6.0+

### Backend

```bash
# From workspace root (/data/data/com.termux/files/home/rust/web)

# Development mode
cargo run

# Build for production
cargo build --release

# Run tests
cargo test

# Lint with Clippy
cargo clippy
```

**First-time setup**: If no `config.yaml` exists, the app will start in installation mode. Visit `/admin/install` to complete database and base configuration. The system will auto-generate `config.yaml`.

### Frontend

```bash
# Navigate to frontend directory
cd view

# Install dependencies
npm install

# Development server (with HTTPS enabled, self-signed cert)
npm run dev

# Production build
npm run build

# Preview production build
npm run preview
```

### Database

Import the schema from `new.sql` into your MySQL database:

```bash
mysql -u root -p < new.sql
```

---

## API Route Structure

```
/                              # Welcome page
├── /admin/*                   # Admin panel static files
├── /static/*                  # Public static resources
├── /upload/*                  # Uploaded file access
│
└── /api/
    ├── /health                # Health check
    ├── /install               # Installation API
    │
    ├── /admin/                # Admin endpoints
    │   ├── /login             # Admin login
    │   ├── /user              # User management
    │   ├── /app               # App management
    │   ├── /function          # Cloud functions
    │   ├── /ver               # Version management
    │   └── ...
    │
    ├── /user/                 # User endpoints
    │   ├── /login             # User login
    │   ├── /register          # User registration
    │   ├── /info              # User info
    │   └── ...
    │
    ├── /oauth2.0/             # OAuth2 callbacks
    │   ├── /qqlogon/callback
    │   └── /wxlogon/callback
    │
    └── /index/                # Public index endpoints
        ├── /appinfo
        └── /config
```

---

## Frontend View Modules

| Path | Purpose |
|------|---------|
| `/views/dashboard/` | Dashboard / statistics overview |
| `/views/user/` | User management |
| `/views/admin/` | Admin management |
| `/views/app/` | Application management |
| `/views/goods/` | Product/goods management |
| `/views/order/` | Order management |
| `/views/pay/` | Payment configuration |
| `/views/kami/` | Card-key management |
| `/views/agent/` | Agent/affiliate management |
| `/views/function/` | Cloud function editor |
| `/views/ver/` | Version management |
| `/views/notice/` | Announcements |
| `/views/message/` | Message management |
| `/views/statistics/` | Statistical analysis |
| `/views/visualization/` | Data visualization |

---

## Architecture

### Backend Layering

```
HTTP Request
    │
    ▼
Middleware Layer (CORS → I18n → AppContext → Auth)
    │
    ▼
Handler Layer (Admin API | User API | Index API)
    │
    ▼
Service/Model Layer (Models | Plugins | Utils | Core)
    │
    ▼
Database (MySQL) / Cache (Redis + LRU)
```

### Caching Strategy

```
L1 (LRU local cache) → L2 (Redis distributed cache) → MySQL (persistent)
```

### Configuration

Main config file: `config.yaml` (auto-generated on first install)

Contains:
- Server settings (HTTP/HTTPS/QUIC ports)
- Database connection (MySQL)
- Redis connection
- Application base info

---

## Development Conventions

### Rust

- Use `#![allow(...)]` during development to suppress warnings; clean up for production
- Modules should have doc comments explaining architecture
- Handlers use `#[route]` macro for route definition
- Error handling uses `anyhow::Result`
- Edition 2024, MSRV 1.85
- Release profile: `lto = "fat"`, `opt-level = 3`, `codegen-units = 1`, `strip = true`, `panic = "unwind"`

### Frontend (Vue 3)

- Composition API throughout
- Component naming: `sa-` prefix (e.g., `sa-icon`)
- API requests encapsulated in `src/api/`
- State management via Pinia
- i18n via `vue-i18n` (zh_CN, en)

### i18n

- **Backend**: `Nakamasa-Ichika/locales/`, uses `fluent-templates`
- **Frontend**: `view/src/i18n/`, uses `vue-i18n`
- Supported languages: Chinese (zh-CN/zh_CN), English (en)

---

## Key Files

| File | Description |
|------|-------------|
| `Cargo.toml` | Workspace manifest with dependency versions and build profiles |
| `Nakamasa-Ichika/src/main.rs` | Backend entry point |
| `Nakamasa-Ichika/src/app/routes.rs` | Route definitions |
| `Nakamasa-Ichika/src/core/run.rs` | Server startup logic |
| `view/package.json` | Frontend dependencies and scripts |
| `view/vite.config.js` | Vite configuration (proxy to backend) |
| `new.sql` | Complete database schema (788 lines) |

---

## Important Notes

1. **Android/Termux Environment**: Currently running on Termux. Cloud functions use **QuickJS** (not V8).
2. **GeoIP**: Requires `GeoLite2-City.mmdb` file in project root directory.
3. **HTTPS Dev Server**: Frontend dev server uses HTTPS with self-signed certificate.
4. **Vite Proxy**: Frontend proxies API requests to backend via Vite proxy configuration.
5. **SQLx**: Uses compile-time query checking — `.sqlx` directory contains cached query plans.

---

## Common Commands

```bash
# ── Backend ──
cargo run                    # Run in development mode
cargo build --release        # Production build
cargo test                   # Run tests
cargo clippy                 # Lint with Clippy

# ── Frontend ──
cd view && npm install       # Install dependencies
cd view && npm run dev       # Start dev server
cd view && npm run build     # Production build
cd view && npm run preview   # Preview built files

# ── Database ──
mysql -u root -p < new.sql   # Import schema
```
