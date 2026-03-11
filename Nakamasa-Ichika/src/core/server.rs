//! HTTP 服务器模块
//! 
//! 提供高性能、安全的 HTTP/HTTPS/QUIC 服务器配置

use salvo::Router;
use salvo::prelude::*;
use salvo::logging::Logger;
use salvo::conn::rustls::{RustlsConfig, Keycert};
use salvo::affix_state;
use std::sync::Arc;
use std::collections::HashMap;
use crate::core::{AppState, I18nMiddleware, terminal_t, t_with_args};
use crate::config::ServerConfig;
use crate::config;

/// 服务器配置
pub struct Server {
    config: &'static ServerConfig,
}

impl Server {
    pub fn new(config: &'static ServerConfig) -> Self {
        Self { config }
    }

    /// 启动服务器
    pub async fn start(&self, state: Arc<AppState>, router: Router) -> anyhow::Result<()> {
        let router = self.build_router(state, router);
        let port = self.config.port();

        tracing::info!("{}", terminal_t("tls.loading_certs"));
        
        // 加载 TLS 证书
        let cert = include_bytes!("../../certs/ssl.crt").to_vec();
        let key = include_bytes!("../../certs/ssl.key").to_vec();

        tracing::info!("{}", terminal_t("tls.loaded"));

        // 创建服务
        let mut service = Service::new(router);

        // 检查 debug 标志，如果为 true 则添加 Logger 中间件
        if config::get().debug().debug() {
            service = service.hoop(Logger::new());
            tracing::info!("{}", terminal_t("server.debug_mode"));
        }

        // 配置 TLS
        tracing::info!("{}", terminal_t("tls.configuring"));
        let rustls_config = RustlsConfig::new(
            Keycert::new()
                .cert(cert.as_slice())
                .key(key.as_slice())
        );
    
        // 创建监听器
        tracing::info!("{}", terminal_t("server.creating_listener"));
        
        // 根据平台选择监听方式
        // 移动设备：仅 TCP + TLS，禁用 QUIC 减少资源消耗
        // 服务器：TCP + TLS + QUIC
        #[cfg(target_os = "android")]
        let acceptor = {
            tracing::info!("Mobile mode: TCP + TLS only");
            TcpListener::new(("0.0.0.0", port))
                .rustls(rustls_config.clone())
                .bind()
                .await
        };
        
        #[cfg(not(target_os = "android"))]
        let acceptor = {
            tracing::info!("Server mode: TCP + TLS + QUIC");
            let tcp_listener = TcpListener::new(("0.0.0.0", port))
                .rustls(rustls_config.clone());
            
            // QUIC 协议监听器
            tracing::info!("{}", terminal_t("quic.enabling"));
            let quic_config = rustls_config.build_quinn_config()
                .map_err(|e| anyhow::anyhow!("Failed to build QUIC config: {}", e))?;
            
            QuinnListener::new(quic_config, ("0.0.0.0", port))
                .join(tcp_listener)
                .bind()
                .await
        };
        
        // 输出监听端口信息
        let port_str = port.to_string();
        let mut args = HashMap::new();
        args.insert("port", port_str.as_str());
        tracing::info!("{}", t_with_args("server.listening", &args));
        tracing::info!("{}", terminal_t("server.ready"));
        
        // 启动服务器
        salvo::prelude::Server::new(acceptor)
            .serve(service)
            .await;

        Ok(())
    }

    /// 构建路由
    fn build_router(&self, state: Arc<AppState>, router: Router) -> Router {
        Router::new()
            .push(router)
            .hoop(I18nMiddleware) // 添加国际化中间件
            .hoop(affix_state::inject(state))
    }
}

// ============================================================================
// 服务器状态
// ============================================================================

/// 服务器运行时状态
pub struct ServerState {
    /// 启动时间
    pub start_time: std::time::Instant,
    /// 请求数量
    pub request_count: std::sync::atomic::AtomicU64,
    /// 活跃连接数
    pub active_connections: std::sync::atomic::AtomicU64,
}

impl ServerState {
    /// 创建新的服务器状态
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            request_count: std::sync::atomic::AtomicU64::new(0),
            active_connections: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    /// 记录请求
    #[inline]
    pub fn record_request(&self) {
        self.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// 增加活跃连接
    #[inline]
    pub fn add_connection(&self) {
        self.active_connections.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// 减少活跃连接
    #[inline]
    pub fn remove_connection(&self) {
        self.active_connections.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    /// 获取运行时间（秒）
    #[inline]
    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
    
    /// 获取请求总数
    #[inline]
    pub fn total_requests(&self) -> u64 {
        self.request_count.load(std::sync::atomic::Ordering::Relaxed)
    }
    
    /// 获取活跃连接数
    #[inline]
    pub fn current_connections(&self) -> u64 {
        self.active_connections.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}