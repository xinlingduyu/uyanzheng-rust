//! Nakamasa-utils 工具库

// 全局警告抑制
#![allow(unused)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unsafe_op_in_unsafe_fn)]
// Clippy 警告抑制
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::type_complexity)]
#![allow(clippy::borrow_interior_mutable_const)]
#![allow(clippy::await_holding_lock)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::inherent_to_string)]

pub mod jwt;
pub mod geoip;
pub mod db_mysql;
pub mod tiered_cache;
pub mod crypto;

/// 高性能缓存模块
/// 
/// 提供极致优化的缓存实现，包括：
/// - 分片架构减少锁竞争
/// - SIMD 加速哈希计算
/// - 多种淘汰策略
/// - 内存池优化
/// - 完善的监控统计
pub mod high_perf_cache;

/// 分布式缓存模块
/// 
/// 提供分布式缓存支持，包括：
/// - 多级缓存 (L1 本地 + L2 Redis)
/// - 分布式锁
/// - 缓存同步/失效广播
/// - 一致性哈希分片
pub mod distributed;

pub use jwt::*;
pub use geoip::*;
pub use db_mysql::*;
pub use tiered_cache::*;
pub use crypto::{
    encrypt,
    decrypt,
    is_encrypted,
    decrypt_if_needed,
    secure_zero,
    ENCRYPTED_PREFIX,
    CryptoError,
};

// 重新导出高性能缓存的核心类型
pub use high_perf_cache::{
    // 核心缓存 (V1)
    ShardedCache,
    CacheEntry,
    CacheStats,
    CacheMetrics,
    
    // 核心缓存 (V2 - 高性能版本)
    ShardedCacheV2,
    CacheStatsV2,
    
    // 配置
    CacheConfig,
    CacheBuilder,
    EvictionPolicy,
    
    // 管理器
    CacheManager,
    MultiCacheManager,
    CacheWarmer,
    create_cache,
    create_cache_with_capacity,
    
    // 工具
    CACHE_LINE_SIZE,
    prefetch_l1,
    prefetch_l2,
    prefetch_l3,
    memory_barrier,
    cpu_pause,
    
    // 哈希
    hash_bytes,
    hash_and_mod,
    HashAlgorithm,
    FastHashBuilder,
    fast_eq,
    
    // 内存池
    FixedSizePool,
    MultiSizePool,
    ObjectPool,
    BufferPool,
    StringPool,
    global_alloc,
    global_free,
};

// 重新导出分布式缓存的核心类型
pub use distributed::{
    // Redis 后端
    RedisBackend,
    RedisConfig,
    RedisError,
    MockRedisBackend,
    TypedCache,
    Pipeline,
    
    // 多级缓存
    MultiLevelCache,
    MultiLevelCacheConfig,
    WritePolicy,
    DistributedCacheManager,
    
    // 分布式锁
    DistributedLock,
    DistributedLockConfig,
    DistributedRwLock,
    FairLock,
    LockError,
    
    // 缓存同步
    CacheSyncBroadcaster,
    CacheSyncSubscriber,
    CacheEvent,
    CacheEventType,
    SyncCacheWrapper,
    
    // 一致性哈希
    ConsistentHash,
    ConsistentHashConfig,
    WeightedConsistentHash,
    ShardMapper,
};

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
