# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nakamasa-Ichika is a high-performance full-stack SaaS platform with user authentication, multi-tenant application support, payment integration, agent/affiliate systems, and cloud function execution. Built with Rust (backend) and Vue 3 (frontend).

## Workspace Structure

This is a Rust workspace with three crates:
- `Nakamasa-Ichika/` - Main backend application (Salvo framework)
- `Nakamasa-utils/` - Shared utilities (JWT, caching, GeoIP)
- `Nakamasa-proc/` - Procedural macros for routes/controllers/middleware

Plus a Vue 3 admin frontend in `view/`.

## Common Commands

### Backend (from workspace root)
```bash
cargo run                    # Development mode
cargo build --release        # Production build (LTO + max optimization)
cargo build --profile dev-fast # Optimized dev build
cargo test                   # Run tests
cargo clippy                 # Lint with Clippy
```

### Frontend (from `view/` directory)
```bash
npm install                  # Install dependencies
npm run dev                  # Start dev server (HTTPS with self-signed cert)
npm run build                # Production build
npm run preview              # Preview production build
npm run tailwind             # Tailwind config viewer
```

### Database
```bash
mysql -u root -p < new.sql   # Import schema (788 lines)
```

## Architecture

### Backend Layering
```
HTTP Request → Middleware (CORS → I18n → AppContext → Auth) → Handler → Service/Model → DB/Cache
```

### Multi-Tenant Design
- Single instance, multiple applications with isolated configurations
- Database separation by `appid` field
- Cache isolation per application
- API authentication via header-based app identification

### Caching Strategy
Three-tier cache: L1 (LRU local) → L2 (Redis) → MySQL

## Key Configuration

### First-Time Setup
1. Run application without `config.yaml` - it enters installation mode
2. Visit `/admin/install` to complete database and base configuration
3. System auto-generates `config.yaml`

### Environment Variables (Frontend)
- `VITE_APP_BASE_URL` - Backend API URL for proxy
- `VITE_APP_PORT` - Frontend dev server port
- `VITE_APP_PROXY_PREFIX` - API path prefix for proxy
- `VITE_APP_BASE` - Base path for deployment

### Special Files
- `config.yaml` - Auto-generated main configuration
- `GeoLite2-City.mmdb` - GeoIP database (place in project root)
- `new.sql` - Complete database schema
- `.sqlx/` - SQLx compile-time query cache

## Development Conventions

### Rust
- Edition 2024, MSRV 1.85
- Handlers use `#[route]` macro from `Nakamasa-proc`
- Error handling with `anyhow::Result`
- SQLx with compile-time query checking
- Cloud functions use **QuickJS** (not V8) for Android/Termux compatibility

### Frontend
- Vue 3 Composition API
- Component naming: `sa-` prefix (e.g., `sa-icon`)
- API requests in `src/api/`
- State via Pinia with persistence
- i18n via `vue-i18n` (zh_CN, en)

### Testing
- No test files discovered - test framework not yet set up

## Important Notes

1. **Android/Termux**: Currently running in Termux environment
2. **HTTPS Dev Server**: Frontend uses HTTPS with self-signed certificate
3. **Vite Proxy**: Frontend proxies API requests to backend via Vite config
4. **SQLx Queries**: Queries are checked at compile time - `.sqlx` directory caches plans
5. **Release Build**: Uses aggressive optimization (LTO fat, opt-level 3, single codegen unit)

## Key File Locations

- `Nakamasa-Ichika/src/main.rs` - Backend entry point
- `Nakamasa-Ichika/src/app/routes.rs` - Route definitions
- `Nakamasa-Ichika/src/core/run.rs` - Server startup
- `view/vite.config.js` - Vite configuration with HTTPS and proxy
- `Cargo.toml` - Workspace manifest with optimized release profile
