//! 数据库迁移模块
//! 
//! 负责检查并执行数据库版本迁移，支持跨版本更新。
//! 版本号使用语义化版本（如 0.1.0, 0.1.1 等）
//! 
//! 架构说明：
//! - APP_VERSION：程序版本（从 Cargo.toml 读取）
//! - config.yaml 中的 app.ver：配置文件版本
//! - u_migration 表：记录已执行的迁移

use crate::core::AppState;
use sqlx::MySqlPool;
use std::fs;
use std::path::Path;

/// 程序版本号（与 Cargo.toml 同步）
/// 当程序版本高于配置版本时，执行迁移
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 迁移类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MigrationType {
    Database,  // 只更新数据库
    Config,    // 只更新配置文件
    Both,      // 两者都更新
}

/// 单个迁移定义
struct Migration {
    version: &'static str,          // 目标版本号
    description: &'static str,      // 迁移描述
    migration_type: MigrationType,   // 迁移类型
    sql: Option<&'static str>,      // SQL 语句（如有）
}

/// 获取所有可用的迁移（按版本号排序）
/// 
/// 添加新迁移时，在这里添加新的 Migration 记录
fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: "1.0.1",
            description: "添加 AI 配置字段到 u_app 表",
            migration_type: MigrationType::Database,
            sql: Some(
                "ALTER TABLE u_app 
                 ADD COLUMN ai_state VARCHAR(10) DEFAULT 'off' COMMENT 'AI功能状态 on/off',
                 ADD COLUMN ai_provider VARCHAR(50) DEFAULT NULL COMMENT 'AI提供商',
                 ADD COLUMN ai_api_base TEXT DEFAULT NULL COMMENT 'AI API地址',
                 ADD COLUMN ai_api_key TEXT DEFAULT NULL COMMENT 'AI API密钥',
                 ADD COLUMN ai_model VARCHAR(100) DEFAULT NULL COMMENT 'AI模型名称',
                 ADD COLUMN ai_temperature DECIMAL(3,2) DEFAULT NULL COMMENT 'AI温度参数',
                 ADD COLUMN ai_max_tokens INT(10) UNSIGNED DEFAULT NULL COMMENT 'AI最大token数';"
            ),
        },
        // 添加更多迁移...
    ]
}

/// 检查并执行迁移（供外部调用）
/// 
/// 简化版本，自动使用 config.yaml 作为配置文件
pub async fn check_and_run_migration(pool: &MySqlPool) -> Result<(), anyhow::Error> {
    let config_path = "config.yaml";
    check_and_run(pool, config_path).await
        .map_err(|e| anyhow::anyhow!("迁移失败: {}", e))
}

/// 检查并执行迁移
/// 
/// 逻辑：
/// 1. 读取 config.yaml 中的 app.ver
/// 2. 与 APP_VERSION 比较
/// 3. 如果 APP_VERSION > config_version，执行待迁移的脚本
/// 4. 更新 config.yaml 中的 ver
/// 5. 记录迁移到 u_migration 表
pub async fn check_and_run(pool: &MySqlPool, config_path: &str) -> Result<(), String> {
    // 1. 读取当前配置版本
    let config_version = read_config_version(config_path)?;
    
    // 2. 比较版本
    if config_version == APP_VERSION {
        tracing::info!("数据库版本已是最新: {}", APP_VERSION);
        return Ok(());
    }
    
    if compare_versions(APP_VERSION, &config_version) <= 0 {
        tracing::info!("配置版本 {} 不低于程序版本 {}", config_version, APP_VERSION);
        return Ok(());
    }
    
    tracing::info!("检测到版本更新: {} -> {}", config_version, APP_VERSION);
    
    // 3. 获取需要执行的迁移
    let migrations = get_migrations();
    let pending: Vec<&Migration> = migrations
        .iter()
        .filter(|m| {
            compare_versions(m.version, &config_version) > 0 
            && compare_versions(m.version, APP_VERSION) <= 0
        })
        .collect();
    
    if pending.is_empty() {
        tracing::info!("没有待执行的迁移");
        // 仍然更新配置版本
        update_config_version(config_path, APP_VERSION)?;
        return Ok(());
    }
    
    tracing::info!("发现 {} 个待执行迁移", pending.len());
    
    // 4. 执行迁移
    for migration in pending {
        execute_migration(pool, migration).await?;
    }
    
    // 5. 更新配置版本号
    update_config_version(config_path, APP_VERSION)?;
    
    tracing::info!("数据库迁移完成，当前版本: {}", APP_VERSION);
    Ok(())
}

/// 新安装时的初始化
/// 
/// 在首次安装成功后调用，初始化迁移记录和配置版本
pub fn init_on_install(config_path: Option<&str>) {
    // 初始化迁移表（如果不存在）
    // 注意：这个方法在首次安装时调用，此时数据库已创建
    
    // 写入初始迁移记录（版本 0.1.0）
    // 实际SQL执行在 install.rs 中完成
    
    // 更新 config.yaml 版本号
    if let Some(path) = config_path {
        if let Err(e) = update_config_version(path, APP_VERSION) {
            tracing::error!("更新配置版本失败: {}", e);
        }
    }
}

/// 执行单个迁移
async fn execute_migration(pool: &MySqlPool, migration: &Migration) -> Result<(), String> {
    tracing::info!("执行迁移 {}: {}", migration.version, migration.description);
    
    // 记录开始
    let start_time = chrono::Utc::now().timestamp();
    
    // 执行 SQL（如果有）
    if let Some(sql) = migration.sql {
        match sqlx::query(sql).execute(pool).await {
            Ok(_) => {
                tracing::info!("迁移 {} SQL 执行成功", migration.version);
            }
            Err(e) => {
                let error_msg = format!("迁移 {} 失败: {}", migration.version, e);
                tracing::error!("{}", error_msg);
                
                // 记录失败
                let _ = record_migration(pool, migration.version, &migration.description, false, &error_msg).await;
                return Err(error_msg);
            }
        }
    }
    
    // 如果是 Config 或 Both 类型，更新配置文件
    if matches!(migration.migration_type, MigrationType::Config | MigrationType::Both) {
        // TODO: 实现配置文件更新逻辑
        tracing::info!("迁移 {} 需要更新配置文件", migration.version);
    }
    
    // 记录成功
    record_migration(pool, migration.version, &migration.description, true, "")
        .await?;
    
    Ok(())
}

/// 记录迁移历史
async fn record_migration(
    pool: &MySqlPool,
    version: &str,
    description: &str,
    success: bool,
    error_message: &str,
) -> Result<(), String> {
    let executed_at = chrono::Utc::now().timestamp();
    let success_int = if success { 1 } else { 0 };
    
    let query = "
        INSERT INTO u_migration (version, description, executed_at, success, error_message)
        VALUES (?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            success = VALUES(success),
            error_message = VALUES(error_message),
            executed_at = VALUES(executed_at)
    ";
    
    sqlx::query(query)
        .bind(version)
        .bind(description)
        .bind(executed_at)
        .bind(success_int)
        .bind(error_message)
        .execute(pool)
        .await
        .map_err(|e| format!("记录迁移失败: {}", e))?;
    
    Ok(())
}

/// 读取配置文件的版本号
fn read_config_version(config_path: &str) -> Result<String, String> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    // 简单解析 YAML 获取 app.ver
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("ver:") {
            let ver = trimmed.trim_start_matches("ver:")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            return Ok(ver.to_string());
        }
    }
    
    // 如果没找到，返回默认版本
    Ok("0.0.0".to_string())
}

/// 更新配置文件版本号
fn update_config_version(config_path: &str, new_version: &str) -> Result<(), String> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    let mut found = false;
    
    for line in &mut lines {
        if line.trim_start().starts_with("ver:") {
            *line = format!("    ver: {}", new_version);
            found = true;
            break;
        }
    }
    
    if !found {
        // 如果没找到 ver 字段，在 app: 部分添加
        for (i, line) in lines.iter_mut().enumerate() {
            if line.trim() == "app:" {
                lines.insert(i + 1, format!("    ver: {}", new_version));
                break;
            }
        }
    }
    
    let new_content = lines.join("\n");
    fs::write(config_path, new_content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    
    tracing::info!("配置文件版本已更新: {}", new_version);
    Ok(())
}

/// 比较版本号
/// 
/// 返回：
/// - 负数：v1 < v2
/// - 0：v1 == v2
/// - 正数：v1 > v2
fn compare_versions(v1: &str, v2: &str) -> i32 {
    let v1_parts: Vec<u32> = v1.split('.')
        .map(|p| p.parse().unwrap_or(0))
        .collect();
    let v2_parts: Vec<u32> = v2.split('.')
        .map(|p| p.parse().unwrap_or(0))
        .collect();
    
    for i in 0..3 {
        let p1 = v1_parts.get(i).copied().unwrap_or(0);
        let p2 = v2_parts.get(i).copied().unwrap_or(0);
        if p1 != p2 {
            return (p1 as i32) - (p2 as i32);
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compare_versions() {
        assert!(compare_versions("0.1.0", "0.1.0") == 0);
        assert!(compare_versions("0.1.1", "0.1.0") > 0);
        assert!(compare_versions("0.1.0", "0.1.1") < 0);
        assert!(compare_versions("1.0.0", "0.9.9") > 0);
    }
}
