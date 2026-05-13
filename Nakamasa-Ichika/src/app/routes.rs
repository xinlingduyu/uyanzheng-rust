//! # 路由模块 (Routes Module)
//!
//! 定义应用程序的 URL 路由规则和路由分组。
//!
//! ## 路由结构
//!
//! ```text
//! /                           # 根路径 (GET: 欢迎页)
//! ├── /admin/*                # 管理后台静态文件
//! ├── /static/*               # 公共静态资源
//! ├── /upload/*               # 上传文件访问
//! │
//! └── /api/
//!     ├── /health             # 健康检查
//!     │
//!     ├── /install            # 安装 API
//!     │
//!     ├── /admin/             # 管理员 API
//!     │   ├── /login          # 登录
//!     │   ├── /user           # 用户管理
//!     │   ├── /app            # 应用管理
//!     │   └── ...
//!     │
//!     ├── /user/              # 用户 API
//!     │   ├── /login          # 登录
//!     │   ├── /register       # 注册
//!     │   ├── /info           # 用户信息
//!     │   └── ...
//!     │
//!     ├── /oauth2.0/          # OAuth2 回调
//!     │   ├── /qqlogon/callback
//!     │   └── /wxlogon/callback
//!     │
//!     └── /index/             # 首页 API
//!         ├── /appinfo
//!         └── /config
//! ```
//!
//! ## 安装模式
//!
//! 当系统未安装时（config.yaml 不存在），只注册以下路由：
//! - `/admin/*` - 安装页面静态文件
//! - `/api/install/*` - 安装 API
//! - `/api/health` - 健康检查

use salvo::Router;

use super::handlers;
use super::handlers::api::{admin_routes, user_routes, index_routes, install_routes};
use super::middleware::cors::cors;
use super::middleware::connect::connect_handler;
use crate::core::is_installed;

/// 构建应用路由
///
/// 根据安装状态返回不同的路由配置。
///
/// # 安装前模式
///
/// 只提供安装页面和安装 API，引导用户完成安装流程。
///
/// # 安装后模式
///
/// 注册所有业务路由，包括管理后台、用户 API 等。
///
/// # Example
///
/// ```rust,ignore
/// use crate::app::routes::routes;
///
/// let router = routes();
/// server.start(state, router).await?;
/// ```
pub fn routes() -> Router {
    // 检查是否已安装
    if !is_installed() {
        return build_installation_routes();
    }
    
    build_production_routes()
}

/// 构建安装模式路由
///
/// 仅提供安装所需的最小路由集合。
fn build_installation_routes() -> Router {
    Router::new()
        .hoop(cors)
        // 静态资源路由 - 只提供 admin 静态文件（安装页面）
        .push(static_files::admin_static_route())
        .push(static_files::root_static_route())
        // 健康检查
        .push(health::health_check::route())
        // 安装 API 路由
        .push(install_routes())
        // 欢迎页
        .get(handlers::hello::hello).hoop(connect_handler)
}

/// 构建生产模式路由
///
/// 注册完整的业务路由。
fn build_production_routes() -> Router {
    Router::new()
        .hoop(cors)
        // ========================================
        // 静态资源路由
        // ========================================
        
        // 上传文件访问路由 - 安全访问上传的图片/文件
        .push(static_files::upload_files_route())
        
        // 静态资源路由 - 优先级较高
        .push(static_files::static_files_route())
        .push(static_files::admin_static_route())
        .push(static_files::root_static_route())
        
        // ========================================
        // API 路由
        // ========================================
        
        // 健康检查
        .push(health::health_check::route())
        
        // 管理后台 API 路由 - /api/admin/*
        .push(admin_routes())
        
        // 用户 API 路由
        .push(user_routes())
        
        // OAuth2.0 回调路由
        .push(oauth2_routes())
        
        // 索引 API 路由
        .push(index_routes())
        
        // 欢迎页
        .get(handlers::hello::hello).hoop(connect_handler)
}

/// OAuth2.0 回调路由
///
/// 处理第三方登录平台的回调请求。
///
/// # 路由
///
/// - `GET /api/oauth2.0/qqlogon/callback` - QQ 登录回调
/// - `GET /api/oauth2.0/wxlogon/callback` - 微信登录回调
fn oauth2_routes() -> Router {
    Router::with_path("/api/oauth2.0")
        // QQ 登录回调
        .push(Router::with_path("/qqlogon/callback")
            .get(handlers::api::user::oauth::qqlogonCallback::qq_logon_callback))
        // 微信登录回调
        .push(Router::with_path("/wxlogon/callback")
            .get(handlers::api::user::oauth::wxlogonCallback::wx_logon_callback))
}

// ============================================================================
// 模块导入
// ============================================================================

use handlers::static_files;
use handlers::health;
use handlers::api;