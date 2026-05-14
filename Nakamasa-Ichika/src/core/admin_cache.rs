//! 管理员缓存服务
//!
//! 封装管理员信息的缓存逻辑，提供简洁的 API：
//! - 自动处理缓存命中/未命中
//! - 自动同步数据库和缓存
//! - 支持密码变更检测和缓存失效

use Nakamasa_utils::{CacheConfig, EvictionPolicy, ShardedCacheV2};
use sqlx::MySqlPool;
use std::sync::Arc;
use std::time::Duration;

/// 管理员缓存条目
#[derive(Clone, Debug)]
pub struct AdminData {
    pub id: u64,
    pub user: String,
    pub password: String,
    pub notes: Option<String>,
    pub state: String,
    pub avatars: Option<String>,
    pub auth: Option<String>,
    pub lockin: bool,
    pub appid: Option<u64>,
}

impl AdminData {
    /// 检查账号是否正常
    #[inline]
    pub fn is_active(&self) -> bool {
        self.state == "y"
    }

    /// 获取权限列表
    #[inline]
    pub fn auth_list(&self) -> serde_json::Value {
        match &self.auth {
            Some(v) => serde_json::from_str(v).unwrap_or_else(|_| serde_json::json!(["all"])),
            None => serde_json::json!(["all"]),
        }
    }
}

/// 缓存查询结果
#[derive(Debug)]
pub enum CacheResult<T> {
    /// 缓存命中
    Hit(T),
    /// 缓存未命中，已从数据库加载
    Miss(T),
    /// 数据不存在
    NotFound,
    /// 数据库错误
    Error(String),
}

impl<T> CacheResult<T> {
    #[inline]
    pub fn is_hit(&self) -> bool {
        matches!(self, CacheResult::Hit(_))
    }

    #[inline]
    pub fn is_miss(&self) -> bool {
        matches!(self, CacheResult::Miss(_))
    }

    #[inline]
    pub fn data(&self) -> Option<&T> {
        match self {
            CacheResult::Hit(data) | CacheResult::Miss(data) => Some(data),
            _ => None,
        }
    }

    #[inline]
    pub fn into_data(self) -> Option<T> {
        match self {
            CacheResult::Hit(data) | CacheResult::Miss(data) => Some(data),
            _ => None,
        }
    }
}

/// 管理员缓存服务
pub struct AdminCacheService {
    /// 管理员ID -> 管理员数据
    cache: ShardedCacheV2<u64, AdminData>,
    /// 用户名 -> 管理员ID
    name_index: ShardedCacheV2<String, u64>,
    /// 数据库连接池（安装模式下为 Some，安装引导时为 None）
    db: Option<MySqlPool>,
}

impl AdminCacheService {
    /// 创建管理员缓存服务
    pub fn new(db: MySqlPool, capacity: usize) -> Self {
        let config = CacheConfig {
            max_entries: capacity,
            shard_count: 8,
            default_ttl: Duration::from_secs(300), // 5分钟
            eviction_policy: EvictionPolicy::Hybrid {
                lfu_weight: 0.7,
                lru_weight: 0.3,
            },
            ..Default::default()
        };

        let name_config = CacheConfig {
            max_entries: capacity,
            shard_count: 4,
            default_ttl: Duration::from_secs(300),
            eviction_policy: EvictionPolicy::LRU,
            ..Default::default()
        };

        Self {
            cache: ShardedCacheV2::new(config),
            name_index: ShardedCacheV2::new(name_config),
            db: Some(db),
        }
    }

    /// 创建空的管理员缓存服务（用于安装模式，无数据库时）
    pub fn new_empty() -> Self {
        let config = CacheConfig {
            max_entries: 100,
            shard_count: 4,
            default_ttl: Duration::from_secs(60),
            eviction_policy: EvictionPolicy::LRU,
            ..Default::default()
        };

        let name_config = CacheConfig {
            max_entries: 100,
            shard_count: 2,
            default_ttl: Duration::from_secs(60),
            eviction_policy: EvictionPolicy::LRU,
            ..Default::default()
        };

        Self {
            cache: ShardedCacheV2::new(config),
            name_index: ShardedCacheV2::new(name_config),
            db: None,
        }
    }

    /// 通过ID获取管理员（优先缓存）
    pub async fn get_by_id(&self, id: u64) -> CacheResult<AdminData> {
        // 尝试从缓存获取
        if let Some(data) = self.cache.get(&id) {
            return CacheResult::Hit(data);
        }

        // 缓存未命中，从数据库加载
        match self.load_from_db_by_id(id).await {
            Ok(Some(data)) => {
                self.name_index.set(data.user.clone(), id);
                let data = self.cache.set_and_get(id, data);
                CacheResult::Miss(data)
            }
            Ok(None) => CacheResult::NotFound,
            Err(e) => CacheResult::Error(e),
        }
    }

    /// 通过用户名获取管理员（优先缓存）
    pub async fn get_by_name(&self, username: &str) -> CacheResult<AdminData> {
        // 尝试从用户名索引获取ID
        let username_key = username.to_string();
        if let Some(id) = self.name_index.get(&username_key) {
            // 再从主缓存获取数据
            if let Some(data) = self.cache.get(&id) {
                return CacheResult::Hit(data);
            }
        }

        // 缓存未命中，从数据库加载
        match self.load_from_db_by_name(username).await {
            Ok(Some(data)) => {
                let id = data.id;
                self.name_index.set(data.user.clone(), id);
                let data = self.cache.set_and_get(id, data);
                CacheResult::Miss(data)
            }
            Ok(None) => CacheResult::NotFound,
            Err(e) => CacheResult::Error(e),
        }
    }

    /// 验证登录（用户名 + 密码）
    /// 返回 (缓存结果, 管理员数据)
    pub async fn verify_login(
        &self,
        username: &str,
        password_hash: &str,
    ) -> CacheResult<AdminData> {
        // 先尝试从缓存验证
        let username_key = username.to_string();
        if let Some(id) = self.name_index.get(&username_key)
            && let Some(data) = self.cache.get(&id)
            && data.password == password_hash
            && data.is_active()
        {
            return CacheResult::Hit(data);
        }
        // 密码不匹配或账号已禁用，移除过期缓存
        // 但不立即删除，让数据库验证后决定

        // 缓存验证失败，查询数据库
        match self.verify_from_db(username, password_hash).await {
            Ok(Some(data)) => {
                let id = data.id;
                self.name_index.set(data.user.clone(), id);
                let data = self.cache.set_and_get(id, data);
                CacheResult::Miss(data)
            }
            Ok(None) => CacheResult::NotFound,
            Err(e) => CacheResult::Error(e),
        }
    }

    /// 验证Token（ID + 密码MD5）
    pub async fn verify_token(&self, id: u64, password_md5: &str) -> CacheResult<AdminData> {
        // 尝试从缓存验证
        if let Some(data) = self.cache.get(&id) {
            if data.is_active() {
                // 计算密码的MD5进行比较
                let stored_md5 = Self::password_md5(&data.password);
                if stored_md5 == password_md5 {
                    return CacheResult::Hit(data);
                }
            }
            // 缓存数据无效，移除
            self.cache.remove(&id);
        }

        // 缓存验证失败，查询数据库
        match self.verify_token_from_db(id, password_md5).await {
            Ok(Some(data)) => {
                self.name_index.set(data.user.clone(), id);
                let data = self.cache.set_and_get(id, data);
                CacheResult::Miss(data)
            }
            Ok(None) => CacheResult::NotFound,
            Err(e) => CacheResult::Error(e),
        }
    }

    /// 更新缓存中的管理员数据
    #[inline]
    pub fn update(&self, data: AdminData) {
        let id = data.id;
        let user = data.user.clone();
        self.name_index.set(user, id);
        self.cache.set(id, data);
    }

    /// 使缓存失效
    #[inline]
    pub fn invalidate(&self, id: u64) {
        if let Some(data) = self.cache.get(&id) {
            self.name_index.remove(&data.user);
        }
        self.cache.remove(&id);
    }

    /// 使指定用户名的缓存失效
    #[inline]
    pub fn invalidate_by_name(&self, username: &str) {
        let username_key = username.to_string();
        if let Some(id) = self.name_index.get(&username_key) {
            self.cache.remove(&id);
        }
        self.name_index.remove(&username_key);
    }

    /// 清空所有缓存
    #[inline]
    pub fn clear(&self) {
        self.cache.clear();
        self.name_index.clear();
    }

    /// 获取缓存统计
    pub fn stats(&self) -> AdminCacheStats {
        AdminCacheStats {
            entries: self.cache.len(),
            name_index_entries: self.name_index.len(),
        }
    }

    // ==================== 内部数据库操作 ====================

    /// 从数据库加载管理员（通过ID）
    async fn load_from_db_by_id(&self, id: u64) -> Result<Option<AdminData>, String> {
        let db = self.db.as_ref().ok_or("Database not available")?;

        let result = sqlx::query_as::<_, (
            u64, String, String, Option<String>, String, Option<String>, Option<String>, bool, Option<u64>
        )>(
            "SELECT id, user, password, notes, state, avatars, auth, lockin, appid FROM u_admin WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(db)
        .await;

        match result {
            Ok(Some(row)) => Ok(Some(AdminData {
                id: row.0,
                user: row.1,
                password: row.2,
                notes: row.3,
                state: row.4,
                avatars: row.5,
                auth: row.6,
                lockin: row.7,
                appid: row.8,
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// 从数据库加载管理员（通过用户名）
    async fn load_from_db_by_name(&self, username: &str) -> Result<Option<AdminData>, String> {
        let db = self.db.as_ref().ok_or("Database not available")?;

        let result = sqlx::query_as::<_, (
            u64, String, String, Option<String>, String, Option<String>, Option<String>, bool, Option<u64>
        )>(
            "SELECT id, user, password, notes, state, avatars, auth, lockin, appid FROM u_admin WHERE user = ?"
        )
        .bind(username)
        .fetch_optional(db)
        .await;

        match result {
            Ok(Some(row)) => Ok(Some(AdminData {
                id: row.0,
                user: row.1,
                password: row.2,
                notes: row.3,
                state: row.4,
                avatars: row.5,
                auth: row.6,
                lockin: row.7,
                appid: row.8,
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// 从数据库验证登录
    async fn verify_from_db(
        &self,
        username: &str,
        password_hash: &str,
    ) -> Result<Option<AdminData>, String> {
        let db = self.db.as_ref().ok_or("Database not available")?;

        let result = sqlx::query_as::<_, (
            u64, String, String, Option<String>, String, Option<String>, Option<String>, bool, Option<u64>
        )>(
            "SELECT id, user, password, notes, state, avatars, auth, lockin, appid FROM u_admin WHERE user = ? AND password = ? AND state = 'y'"
        )
        .bind(username)
        .bind(password_hash)
        .fetch_optional(db)
        .await;

        match result {
            Ok(Some(row)) => Ok(Some(AdminData {
                id: row.0,
                user: row.1,
                password: row.2,
                notes: row.3,
                state: row.4,
                avatars: row.5,
                auth: row.6,
                lockin: row.7,
                appid: row.8,
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// 从数据库验证Token
    async fn verify_token_from_db(
        &self,
        id: u64,
        password_md5: &str,
    ) -> Result<Option<AdminData>, String> {
        let db = self.db.as_ref().ok_or("Database not available")?;

        let result = sqlx::query_as::<_, (
            u64, String, String, Option<String>, String, Option<String>, Option<String>, bool, Option<u64>
        )>(
            "SELECT id, user, password, notes, state, avatars, auth, lockin, appid FROM u_admin WHERE id = ? AND state = 'y'"
        )
        .bind(id)
        .fetch_optional(db)
        .await;

        match result {
            Ok(Some(row)) => {
                // 验证密码MD5
                let stored_md5 = Self::password_md5(&row.2);
                if stored_md5 == password_md5 {
                    Ok(Some(AdminData {
                        id: row.0,
                        user: row.1,
                        password: row.2,
                        notes: row.3,
                        state: row.4,
                        avatars: row.5,
                        auth: row.6,
                        lockin: row.7,
                        appid: row.8,
                    }))
                } else {
                    Ok(None)
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// 计算密码的MD5
    #[inline]
    fn password_md5(password: &str) -> String {
        use crate::core::md5_optimize::{md5_hex, md5_to_str};
        let bytes = md5_hex(password.as_bytes());
        md5_to_str(&bytes).to_string()
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct AdminCacheStats {
    pub entries: usize,
    pub name_index_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_result() {
        let hit: CacheResult<i32> = CacheResult::Hit(42);
        assert!(hit.is_hit());
        assert_eq!(hit.data(), Some(&42));

        let miss: CacheResult<i32> = CacheResult::Miss(100);
        assert!(miss.is_miss());
        assert_eq!(miss.into_data(), Some(100));
    }
}
