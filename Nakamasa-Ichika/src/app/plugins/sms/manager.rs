//! 短信插件管理器
//! 负责插件的注册、加载和调用

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use super::trait_def::{SmsPlugin, SmsPluginMeta};

/// 短信插件管理器
pub struct SmsPluginManager {
    plugins: RwLock<HashMap<String, Box<dyn SmsPlugin>>>,
}

impl SmsPluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    /// 注册插件
    pub fn register(&self, plugin: Box<dyn SmsPlugin>) -> Result<(), String> {
        let plugin_type = plugin.plugin_type().to_string();
        let mut plugins = self.plugins.write()
            .map_err(|e| format!("获取写锁失败: {}", e))?;
        
        plugins.insert(plugin_type.clone(), plugin);
        tracing::info!("短信插件 {} 已注册", plugin_type);
        Ok(())
    }

    /// 获取插件
    pub fn get_plugin(&self, plugin_type: &str) -> Result<Arc<dyn SmsPlugin>, String> {
        let plugins = self.plugins.read()
            .map_err(|e| format!("获取读锁失败: {}", e))?;
        
        plugins.get(plugin_type)
            .map(|p| {
                unsafe { Arc::from_raw(&**p as *const dyn SmsPlugin) }
            })
            .ok_or_else(|| format!("插件 {} 不存在", plugin_type))
    }

    /// 获取所有插件元数据
    pub fn get_all_meta(&self) -> Vec<SmsPluginMeta> {
        let plugins = self.plugins.read()
            .ok()
            .unwrap();
        
        plugins.values()
            .map(|p| SmsPluginMeta {
                name: p.name().to_string(),
                plugin_type: p.plugin_type().to_string(),
                form: p.config_form(),
            })
            .collect()
    }

    /// 初始化插件
    pub fn init_plugin(&self, plugin_type: &str, config: serde_json::Value) -> Result<(), String> {
        let mut plugins = self.plugins.write()
            .map_err(|e| format!("获取写锁失败: {}", e))?;
        
        if let Some(plugin) = plugins.get_mut(plugin_type) {
            plugin.init(config)
        } else {
            Err(format!("插件 {} 不存在", plugin_type))
        }
    }
}

impl Default for SmsPluginManager {
    fn default() -> Self {
        Self::new()
    }
}