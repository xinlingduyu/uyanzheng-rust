mod server;
mod mysql;
mod redis; // Redis配置
mod i18n; // 国际化配置
mod debug;
mod app_config; // App配置

use std::sync::LazyLock;
use anyhow::Context;
use config::{Config, FileFormat};
use serde::Deserialize;

pub use server::ServerConfig;
pub use mysql::MysqlConfig;
pub use redis::RedisConfig; // 导出Redis配置
pub use i18n::I18nConfig; // 导出国际化配置
pub use debug::DebugConfig;
pub use app_config::AppConfig as AppConfigDetails;

static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| AppConfig::load().expect("Failed to initialize config"));

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    server: ServerConfig,
    mysql: MysqlConfig,
    redis: RedisConfig, // Redis配置
    i18n: I18nConfig,  // 国际化配置
    debug: DebugConfig,  // 国际化配置
    app: AppConfigDetails, // App配置
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        Config::builder()
            .add_source(
                config::File::with_name("config")
                    .format(FileFormat::Yaml)
                    .required(true)
            )
            .add_source(
                config::Environment::with_prefix("APP")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(",")
            )
            .build()
            .with_context(|| anyhow::anyhow!("Failed to load config"))?
            .try_deserialize()
            .with_context(|| anyhow::anyhow!("Failed to deserialize config"))
    }
    
    pub fn server(&self) -> &ServerConfig {
        &self.server
    }
    
    pub fn database(&self) -> &MysqlConfig {
        &self.mysql
    }
    
    pub fn redis(&self) -> &RedisConfig {
        &self.redis
    }
    
    pub fn i18n(&self) -> &I18nConfig {
        &self.i18n
    }
    
    pub fn debug(&self) -> &DebugConfig {
        &self.debug
    }

    pub fn app(&self) -> &AppConfigDetails {
        &self.app
    }
}

pub fn get() -> &'static AppConfig {
    &CONFIG
}
