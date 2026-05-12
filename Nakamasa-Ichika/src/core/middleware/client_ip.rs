//! 客户端 IP 获取工具
//! 支持从代理头获取真实客户端 IP

use salvo::prelude::*;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;

/// IP 缓存，避免重复分配
static IP_CACHE: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();

/// 获取 Mutex 锁，处理中毒恢复
fn lock_cache<'a>(cache: &'a Mutex<HashSet<&'static str>>) -> std::sync::MutexGuard<'a, HashSet<&'static str>> {
    cache.lock().unwrap_or_else(PoisonError::into_inner)
}

/// 获取客户端真实 IP 地址
/// 优先级: X-Real-IP > X-Forwarded-For > 默认 127.0.0.1
#[inline]
pub fn get_client_ip(req: &Request) -> &'static str {
    // 1. 尝试从 X-Real-IP 获取
    if let Some(ip) = extract_ip_from_header(req, "X-Real-IP") {
        return ip;
    }
    
    // 2. 尝试从 X-Forwarded-For 获取（取第一个 IP）
    if let Some(ip) = extract_ip_from_forwarded(req) {
        return ip;
    }
    
    // 3. 默认返回本地地址
    "127.0.0.1"
}

/// 从指定 Header 提取 IP
#[inline]
fn extract_ip_from_header(req: &Request, header_name: &str) -> Option<&'static str> {
    let header = req.headers().get(header_name)?;
    let ip_str = header.to_str().ok()?;
    
    if ip_str.is_empty() {
        return None;
    }
    
    // 对于常见 IP 返回静态字符串
    match ip_str {
        "127.0.0.1" => return Some("127.0.0.1"),
        "::1" => return Some("::1"),
        "0.0.0.0" => return Some("0.0.0.0"),
        _ => {}
    }
    
    // 验证 IP 格式
    if !is_valid_ip(ip_str) {
        return None;
    }
    
    // 使用 Box::leak 返回静态引用，通过 IP_CACHE 去重避免重复泄漏
    let cache = IP_CACHE.get_or_init(|| Mutex::new(HashSet::new()));
    let mut cache_lock = lock_cache(cache);
    if let Some(cached) = cache_lock.get(ip_str) {
        return Some(cached);
    }
    let leaked: &'static str = Box::leak(ip_str.to_string().into_boxed_str());
    cache_lock.insert(leaked);
    Some(leaked)
}

/// 从 X-Forwarded-For 提取第一个 IP
#[inline]
fn extract_ip_from_forwarded(req: &Request) -> Option<&'static str> {
    let header = req.headers().get("X-Forwarded-For")?;
    let ip_list = header.to_str().ok()?;
    
    // 取第一个 IP（最原始的客户端 IP）
    let first_ip = ip_list.split(',').next()?.trim();
    
    if first_ip.is_empty() {
        return None;
    }
    
    // 对于常见 IP 返回静态字符串
    match first_ip {
        "127.0.0.1" => return Some("127.0.0.1"),
        "::1" => return Some("::1"),
        "0.0.0.0" => return Some("0.0.0.0"),
        _ => {}
    }
    
    // 验证 IP 格式
    if !is_valid_ip(first_ip) {
        return None;
    }
    
    // 使用 Box::leak 返回静态引用，通过 IP_CACHE 去重避免重复泄漏
    let cache = IP_CACHE.get_or_init(|| Mutex::new(HashSet::new()));
    let mut cache_lock = lock_cache(cache);
    if let Some(cached) = cache_lock.get(first_ip) {
        return Some(cached);
    }
    let leaked: &'static str = Box::leak(first_ip.to_string().into_boxed_str());
    cache_lock.insert(leaked);
    Some(leaked)
}

/// 验证 IP 地址格式
/// 支持 IPv4 和 IPv6
#[inline]
pub fn is_valid_ip(ip: &str) -> bool {
    if ip.is_empty() || ip.len() > 45 {
        return false;
    }
    
    let mut dot_count = 0;
    let mut colon_count = 0;
    
    for c in ip.chars() {
        match c {
            '0'..='9' => {}
            'a'..='f' | 'A'..='F' => {} // IPv6 十六进制
            '.' => dot_count += 1,
            ':' => colon_count += 1,
            _ => return false,
        }
    }
    
    // 放宽限制以兼容未来协议扩展（IPv6 最多 7 冒号，留余量给 IPv8+）
    dot_count <= 7 && colon_count <= 15
}

/// 获取客户端 IP 并存入 depot
pub fn insert_client_ip(req: &Request, depot: &mut Depot) {
    let ip = get_client_ip(req);
    depot.insert("client_ip", ip.to_string());
}

/// 从 depot 获取客户端 IP
pub fn get_ip_from_depot(depot: &Depot) -> String {
    depot.get::<String>("client_ip")
        .cloned()
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}
