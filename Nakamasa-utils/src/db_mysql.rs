use sqlx::{
    any::{AnyPoolOptions, AnyArguments, AnyRow},
    AnyPool,
    Row, Column, ValueRef,
    types::chrono::Utc,
};
use serde_json::{Value as JsonValue, Number};
use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::Mutex as AsyncMutex;
use once_cell::sync::Lazy;
use thiserror::Error;

// --- 全局驱动注册 ---
static DRIVER_INIT: Lazy<()> = Lazy::new(|| {
    let _ = sqlx::any::install_default_drivers();
});

// --- 全局连接池缓存 ---
static POOL_CACHE: Lazy<AsyncMutex<HashMap<String, Arc<AnyPool>>>> = Lazy::new(|| {
    Lazy::force(&DRIVER_INIT);
    AsyncMutex::new(HashMap::new())
});

// --- 类型定义 ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbType {
    MySql,
    Postgres,
    Sqlite,
}

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub db_type: DbType,
    pub host: String,
    pub port: u16,
    pub dbname: String,
    pub user: String,
    pub pwd: String,
    pub pre: String,
    pub charset: String,
    pub options: HashMap<String, String>,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            db_type: DbType::MySql,
            host: "localhost".to_string(),
            port: 3306,
            dbname: "test".to_string(),
            user: "root".to_string(),
            pwd: "".to_string(),
            pre: "".to_string(),
            charset: "utf8mb4".to_string(),
            options: HashMap::new(),
            max_connections: 20,
            min_connections: 1,
        }
    }
}

impl DbConfig {
    // 修复: 改为 pub，允许外部调用
    pub fn get_url(&self) -> String {
        let mut base_url = match self.db_type {
            DbType::MySql => format!("mysql://{}:{}@{}:{}/{}?charset={}", self.user, self.pwd, self.host, self.port, self.dbname, self.charset),
            DbType::Postgres => format!("postgres://{}:{}@{}:{}/{}", self.user, self.pwd, self.host, self.port, self.dbname),
            DbType::Sqlite => format!("sqlite:{}?mode=rwc", self.dbname),
        };

        let extra_opts: Vec<String> = self.options.iter()
            .filter(|(k, _)| !k.starts_with("charset"))
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        
        if !extra_opts.is_empty() {
            let separator = if base_url.contains('?') { "&" } else { "?" };
            base_url.push_str(&format!("{}{}", separator, extra_opts.join("&")));
        }
        
        base_url
    }
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("数据库连接错误: {0}")]
    Connection(String),
    #[error("SQL执行错误: {0}\nSQL: {1}")]
    Query(String, String),
    #[error("参数错误: {0}")]
    InvalidArgument(String),
    #[error("事务错误: {0}")]
    Transaction(String),
    #[error("不支持的数据库类型")]
    UnsupportedDatabase,
}

#[derive(Debug, Clone)]
pub enum DbEvent {
    BeforeQuery, AfterQuery,
    BeforeInsert, AfterInsert,
    BeforeUpdate, AfterUpdate,
    BeforeDelete, AfterDelete,
    BeforeSelect, AfterSelect,
    BeforeBatchInsert, AfterBatchInsert,
}

#[async_trait]
pub trait EventListener: Send + Sync {
    async fn on_event(&self, event: DbEvent, data: Option<HashMap<String, String>>);
}

// --- Operator 主体 ---
pub struct Operator {
    // 修复: 移除未使用的 db_type 字段
    table_name: String,
    pool: Arc<AnyPool>,
    
    where_clause: Option<(String, Vec<JsonValue>)>,
    group_by: Option<String>,
    join: Option<String>,
    order_by: Option<String>,
    limit: Option<(u32, u32)>,
    
    listener: Option<Arc<dyn EventListener>>,
}

impl Operator {
    pub async fn get(config: DbConfig, table_name: &str, config_name: &str) -> Result<Self, DbError> {
        let pool = Self::get_pool(&config, config_name).await?;
        let full_table_name = format!("{}{}", config.pre, table_name);

        Ok(Self {
            // db_type: config.db_type, // 已移除
            table_name: full_table_name,
            pool,
            where_clause: None,
            group_by: None,
            join: None,
            order_by: None,
            limit: None,
            listener: None,
        })
    }

    async fn get_pool(config: &DbConfig, config_name: &str) -> Result<Arc<AnyPool>, DbError> {
        let mut cache: tokio::sync::MutexGuard<'_, HashMap<String, Arc<AnyPool>>> = POOL_CACHE.lock().await;

        if let Some(pool) = cache.get(config_name) {
            return Ok(pool.clone());
        }

        let database_url = config.get_url();
        
        let pool = AnyPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect(&database_url)
            .await
            .map_err(|e| DbError::Connection(format!("{} (URL: redacted)", e)))?;

        let arc_pool = Arc::new(pool);
        cache.insert(config_name.to_string(), arc_pool.clone());
        Ok(arc_pool)
    }

    pub fn set_table(&mut self, table_name: &str, pre: &str) -> &mut Self {
        self.table_name = format!("{}{}", pre, table_name);
        self
    }

    pub fn listen(&mut self, listener: Arc<dyn EventListener>) {
        self.listener = Some(listener);
    }

    async fn trigger(&self, event: DbEvent, make_data: impl FnOnce() -> HashMap<String, String>) {
        if let Some(listener) = &self.listener {
            let data = make_data();
            listener.on_event(event, Some(data)).await;
        }
    }

    fn reset_query_state(&mut self) {
        self.where_clause = None;
        self.group_by = None;
        self.join = None;
        self.order_by = None;
        self.limit = None;
    }

    async fn execute_raw(&self, sql: &str, params: &[JsonValue]) -> Result<u64, DbError> {
        let mut query = sqlx::query(sql);
        for param in params {
            query = self.bind_param(query, param);
        }
        
        let query: sqlx::query::Query<'_, sqlx::Any, AnyArguments<'_>> = query;
        
        query.execute(&*self.pool)
            .await
            .map(|result| result.rows_affected())
            .map_err(|e| DbError::Query(e.to_string(), sql.to_string()))
    }

    fn bind_param<'q>(&self, query: sqlx::query::Query<'q, sqlx::Any, AnyArguments<'q>>, param: &JsonValue) -> sqlx::query::Query<'q, sqlx::Any, AnyArguments<'q>> {
        match param {
            JsonValue::Null => query.bind(None::<String>),
            JsonValue::Bool(b) => query.bind(*b),
            JsonValue::Number(n) => {
                if n.is_i64() {
                    query.bind(n.as_i64().unwrap())
                } else {
                    query.bind(n.as_f64().unwrap())
                }
            }
            JsonValue::String(s) => query.bind(s.clone()),
            JsonValue::Array(a) => query.bind(serde_json::to_string(a).unwrap_or_default()),
            JsonValue::Object(o) => query.bind(serde_json::to_string(o).unwrap_or_default()),
        }
    }

    pub async fn fetch_all(&mut self) -> Result<Vec<HashMap<String, JsonValue>>, DbError> {
        let (sql, params) = self.build_select_sql();
        
        self.trigger(DbEvent::BeforeSelect, || {
            let mut m = HashMap::new();
            m.insert("sql".into(), sql.clone());
            m
        }).await;

        let start = Utc::now();
        let mut query = sqlx::query(&sql);
        for param in &params {
            query = self.bind_param(query, param);
        }

        let rows = query.fetch_all(&*self.pool)
            .await
            .map_err(|e| DbError::Query(e.to_string(), sql.clone()))?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            results.push(self.row_to_map(&row));
        }

        self.trigger(DbEvent::AfterSelect, || {
            let mut m = HashMap::new();
            m.insert("cost_ms".into(), (Utc::now() - start).num_milliseconds().to_string());
            m.insert("count".into(), results.len().to_string());
            m
        }).await;

        self.reset_query_state();
        Ok(results)
    }

    pub async fn fetch(&mut self) -> Result<Option<HashMap<String, JsonValue>>, DbError> {
        let _original_limit = self.limit;
        self.limit = Some((0, 1));
        
        let result = self.fetch_all().await;
        
        match result {
            Ok(mut rows) => Ok(rows.pop()),
            Err(e) => Err(e),
        }
    }

    pub async fn add(&mut self, data: &HashMap<String, JsonValue>) -> Result<u64, DbError> {
        if data.is_empty() { return Ok(0); }
        
        let cols: Vec<&str> = data.keys().map(|k| k.as_str()).collect();
        let vals: Vec<JsonValue> = data.values().cloned().collect();
        let placeholders: Vec<&str> = cols.iter().map(|_| "?").collect();
        
        let sql = format!("INSERT INTO {} ({}) VALUES ({})", 
            self.table_name, cols.join(", "), placeholders.join(", "));

        self.trigger(DbEvent::BeforeInsert, || {
             let mut m = HashMap::new(); m.insert("sql".into(), sql.clone()); m 
        }).await;

        let res = self.execute_raw(&sql, &vals).await;

        self.trigger(DbEvent::AfterInsert, || {
             let mut m = HashMap::new(); 
             m.insert("result".into(), if res.is_ok() { "success".into() } else { "fail".into() }); 
             m 
        }).await;
        
        res
    }

    pub async fn add_batch<T>(&mut self, keys: &[&str], data: &[Vec<T>]) -> Result<u64, DbError>
    where
        T: Into<JsonValue> + Clone,
    {
        if data.is_empty() { return Ok(0); }
        if keys.is_empty() { return Err(DbError::InvalidArgument("Keys cannot be empty".into())); }

        let mut sql = format!("INSERT INTO {} ({}) VALUES ", self.table_name, keys.join(", "));
        let mut flat_values: Vec<JsonValue> = Vec::new();

        for (i, row) in data.iter().enumerate() {
            if row.len() != keys.len() {
                return Err(DbError::InvalidArgument(format!("Row {} length mismatch with keys", i)));
            }
            if i > 0 { sql.push_str(", "); }
            let placeholders: Vec<&str> = row.iter().map(|_| "?").collect();
            sql.push_str(&format!("({})", placeholders.join(", ")));
            
            for item in row {
                flat_values.push(item.clone().into());
            }
        }

        self.trigger(DbEvent::BeforeBatchInsert, || {
             let mut m = HashMap::new(); m.insert("sql".into(), sql.clone()); m 
        }).await;

        self.execute_raw(&sql, &flat_values).await
    }

    pub async fn update(&mut self, data: &HashMap<String, JsonValue>) -> Result<u64, DbError> {
        if data.is_empty() { return Ok(0); }
        if self.where_clause.is_none() { 
            return Err(DbError::InvalidArgument("Safety: Update requires where condition".into())); 
        }

        let mut sets = Vec::new();
        let mut values = Vec::new();
        for (k, v) in data {
            sets.push(format!("{} = ?", k));
            values.push(v.clone());
        }

        let (where_sql, where_params) = self.where_clause.as_ref().unwrap();
        values.extend(where_params.clone());

        let sql = format!("UPDATE {} SET {} {}", self.table_name, sets.join(", "), where_sql);

        self.trigger(DbEvent::BeforeUpdate, || {
            let mut m = HashMap::new(); m.insert("sql".into(), sql.clone()); m 
       }).await;
       
       let res = self.execute_raw(&sql, &values).await;
       self.reset_query_state(); 
       res
    }

    pub async fn delete(&mut self) -> Result<u64, DbError> {
        if self.where_clause.is_none() { 
            return Err(DbError::InvalidArgument("Safety: Delete requires where condition".into())); 
        }
        
        let (where_sql, params) = self.where_clause.as_ref().unwrap();
        let sql = format!("DELETE FROM {} {}", self.table_name, where_sql);

        self.trigger(DbEvent::BeforeDelete, || {
            let mut m = HashMap::new(); m.insert("sql".into(), sql.clone()); m 
       }).await;

       let res = self.execute_raw(&sql, params).await;
       self.reset_query_state(); 
       res
    }

    pub fn where_condition(&mut self, condition: &str, params: Vec<JsonValue>) -> &mut Self {
        self.where_clause = Some((format!("WHERE {}", condition), params));
        self
    }

    pub fn group_by(&mut self, group: &str) -> &mut Self {
        self.group_by = Some(format!("GROUP BY {}", group));
        self
    }

    pub fn order_by(&mut self, order: &str) -> &mut Self {
        self.order_by = Some(format!("ORDER BY {}", order));
        self
    }

    pub fn limit(&mut self, offset: u32, count: u32) -> &mut Self {
        self.limit = Some((offset, count));
        self
    }

    pub fn page(&mut self, page: u32, page_size: u32) -> &mut Self {
        let page = if page < 1 { 1 } else { page };
        self.limit((page - 1) * page_size, page_size)
    }

    fn build_select_sql(&self) -> (String, Vec<JsonValue>) {
        let mut sql = format!("SELECT * FROM {}", self.table_name);
        let mut params = Vec::new();

        if let Some(join) = &self.join { sql.push_str(&format!(" {}", join)); }
        
        if let Some((w, p)) = &self.where_clause {
            sql.push_str(&format!(" {}", w));
            params.extend(p.clone());
        }

        if let Some(g) = &self.group_by { sql.push_str(&format!(" {}", g)); }
        if let Some(o) = &self.order_by { sql.push_str(&format!(" {}", o)); }

        if let Some((offset, count)) = self.limit {
            sql.push_str(&format!(" LIMIT {} OFFSET {}", count, offset));
        }

        (sql, params)
    }

    fn row_to_map(&self, row: &AnyRow) -> HashMap<String, JsonValue> {
        let mut map = HashMap::new();
        
        for col in row.columns() {
            let ordinal = col.ordinal();
            let key = col.name().to_string();
            
            if let Ok(raw) = row.try_get_raw(ordinal) {
                if raw.is_null() {
                    map.insert(key, JsonValue::Null);
                    continue;
                }
            }

            let value = if let Ok(v) = row.try_get::<i64, _>(ordinal) {
                JsonValue::Number(v.into())
            } else if let Ok(v) = row.try_get::<f64, _>(ordinal) {
                 Number::from_f64(v).map(JsonValue::Number).unwrap_or(JsonValue::Null)
            } else if let Ok(v) = row.try_get::<bool, _>(ordinal) {
                JsonValue::Bool(v)
            } else if let Ok(v) = row.try_get::<String, _>(ordinal) {
                if (v.starts_with('{') && v.ends_with('}')) || (v.starts_with('[') && v.ends_with(']')) {
                    serde_json::from_str(&v).unwrap_or_else(|_| JsonValue::String(v))
                } else {
                    Self::parse_string_value(&v)
                }
            } else {
                JsonValue::String("Unsupported Type".to_string())
            };

            map.insert(key, value);
        }
        map
    }

    fn parse_string_value(v: &str) -> JsonValue {
        match v.to_lowercase().as_str() {
            "true" => return JsonValue::Bool(true),
            "false" => return JsonValue::Bool(false),
            _ => {}
        }
        
        if v.parse::<i64>().is_ok() {
             v.parse::<i64>().ok().map(|i| JsonValue::Number(i.into()))
        } else if v.parse::<f64>().is_ok() {
             v.parse::<f64>().ok().and_then(|f| Number::from_f64(f).map(JsonValue::Number))
        } else {
             None
        }.unwrap_or(JsonValue::String(v.to_string()))
    }
}
