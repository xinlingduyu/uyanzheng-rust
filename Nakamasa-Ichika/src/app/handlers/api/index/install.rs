//! 安装程序
use salvo::prelude::*;
use std::sync::Arc;
use std::fs;
use std::path::Path;
use rand::Rng;
use deadpool_redis::redis::{Client, cmd};

use crate::core::AppState;
use crate::core::md5_optimize::{md5_hex, md5_to_str};
use crate::app::utils::response::ApiResponse;
use crate::app::utils::validator::Validator;
use Nakamasa_utils::encrypt;

/// 检查系统是否已安装（通过 config.yaml 文件是否存在判断）
fn is_installed() -> bool {
    Path::new("config.yaml").exists()
}

// 安装请求数据结构
#[derive(serde::Deserialize)]
pub struct InstallRequest {
    mysql_host: String,
    mysql_port: u16,
    mysql_name: String,
    mysql_user: String,
    mysql_pwd: String,
    mysql_pre: String,
    redis_host: String,
    redis_port: u16,
    redis_pwd: Option<String>,
    admin_user: String,
    admin_pwd: String,
    admin_authcode: String,
    install_type: String,
    install_upgrade: Option<String>,
    adm_pwd: Option<String>,
    /// 是否启用TLS (HTTPS)，默认为 true
    #[serde(default = "default_tls_enabled")]
    tls_enabled: bool,
    /// 自定义证书文件路径（可选）
    cert_path: Option<String>,
    /// 自定义私钥文件路径（可选）
    key_path: Option<String>,
}

fn default_tls_enabled() -> bool {
    true
}

fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 高性能 MD5 哈希 - 使用栈上数组
#[inline]
fn md5_hash(input: &str) -> String {
    md5_to_str(&md5_hex(input.as_bytes())).to_string()
}

#[handler]
pub async fn install(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    // 安全检查：首先检查是否已安装，防止任何操作
    if is_installed() {
        res.render(Json(ApiResponse::<()>::error("已经安装过了", 201)));
        return;
    }

    let _app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    let install_req = match req.parse_json::<InstallRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(ApiResponse::<()>::error("参数解析失败", 201)));
            return;
        }
    };

    // 验证输入参数
    let mut validator = Validator::new();
    validator.string("mysql_host", &install_req.mysql_host, 9, 128);
    validator.int("mysql_port", install_req.mysql_port as i64, 1, 65535);
    validator.string("mysql_name", &install_req.mysql_name, 1, 64);
    validator.string("mysql_user", &install_req.mysql_user, 4, 64);
    validator.string("mysql_pwd", &install_req.mysql_pwd, 4, 64);
    validator.wordnum("mysql_pre", &install_req.mysql_pre, 1, 8);
    validator.string("redis_host", &install_req.redis_host, 9, 128);
    validator.int("redis_port", install_req.redis_port as i64, 1, 65535);
    if let Some(ref pwd) = install_req.redis_pwd {
        validator.string("redis_pwd", pwd, 4, 32);
    }
    validator.wordnum("admin_user", &install_req.admin_user, 5, 12);
    validator.password("admin_pwd", &install_req.admin_pwd, 6, 18);
    validator.wordnum("admin_authcode", &install_req.admin_authcode, 16, 32);
    validator.sameone("install_type", &install_req.install_type, vec!["new", "upgrade"]);
    
    if install_req.install_type == "upgrade"
        && let Some(ref upgrade_ver) = install_req.install_upgrade {
            let version_re = regex::Regex::new(r"^\d+\.\d+(\.\d+)?$").unwrap();
            if !version_re.is_match(upgrade_ver) {
                res.render(Json(ApiResponse::<()>::error("升级版本格式有误", 201)));
                return;
            }
        }
    
    if install_req.install_type == "upgrade"
        && let Some(ref adm_pwd) = install_req.adm_pwd {
            validator.wordnum("adm_pwd", adm_pwd, 32, 32);
        }

    if let Err(msg) = validator.validate() {
        res.render(Json(ApiResponse::<()>::error(msg, 201)));
        return;
    }

    // 生成密码密钥
    let adm_pwd_key = install_req.adm_pwd.unwrap_or_else(|| generate_random_string(32));
    let _adm_jwt_key = generate_random_string(64);
    
    // 加密管理员密码
    let encrypted_pwd = md5_hash(&format!("{}{}", install_req.admin_pwd, adm_pwd_key));

    // 测试MySQL连接
    let mysql_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        install_req.mysql_user,
        install_req.mysql_pwd,
        install_req.mysql_host,
        install_req.mysql_port,
        install_req.mysql_name
    );

    let db_pool = match sqlx::MySqlPool::connect(&mysql_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("MySQL连接失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("数据库连接失败，请检查配置", 201)));
            return;
        }
    };

    // 测试Redis连接
    let redis_url = if let Some(ref pwd) = install_req.redis_pwd {
        format!("redis://:{}@{}:{}", pwd, install_req.redis_host, install_req.redis_port)
    } else {
        format!("redis://{}:{}", install_req.redis_host, install_req.redis_port)
    };

    let redis_client = match Client::open(redis_url.as_str()) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Redis客户端创建失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("Redis配置错误", 201)));
            return;
        }
    };

    match redis_client.get_multiplexed_async_connection().await {
        Ok(mut conn) => {
            if let Err(e) = cmd("PING").query_async::<()>(&mut conn).await {
                tracing::error!("Redis PING失败: {}", e);
                res.render(Json(ApiResponse::<()>::error("Redis连接失败", 201)));
                return;
            }
        }
        Err(e) => {
            tracing::error!("Redis连接失败: {}", e);
            res.render(Json(ApiResponse::<()>::error("Redis连接失败", 201)));
            return;
        }
    }

    // 如果是新安装，创建管理员账号
    if install_req.install_type == "new" {
        let create_admin_sql = format!(
            "INSERT INTO {}_admin (username, password, notes) VALUES (?, ?, '超级管理员')",
            install_req.mysql_pre
        );

        match sqlx::query(&create_admin_sql)
            .bind(&install_req.admin_user)
            .bind(&encrypted_pwd)
            .execute(&db_pool)
            .await
        {
            Ok(_) => {},
            Err(e) => {
                tracing::error!("创建管理员失败: {}", e);
                res.render(Json(ApiResponse::<()>::error("创建管理员账号失败", 201)));
                return;
            }
        }
    } else {
        // 升级模式，验证管理员
        let check_admin_sql = format!(
            "SELECT id FROM {}_admin WHERE username = ? AND password = ?",
            install_req.mysql_pre
        );

        match sqlx::query_as::<_, (i64,)>(&check_admin_sql)
            .bind(&install_req.admin_user)
            .bind(&encrypted_pwd)
            .fetch_optional(&db_pool)
            .await
        {
            Ok(Some(_)) => {},
            Ok(None) => {
                res.render(Json(ApiResponse::<()>::error("管理员账号密码有误或管理员密码密钥有误", 201)));
                return;
            }
            Err(e) => {
                tracing::error!("查询管理员失败: {}", e);
                res.render(Json(ApiResponse::<()>::error("数据库错误", 201)));
                return;
            }
        }
    }

    // 创建 config.yaml 文件
    let config_content = generate_config_yaml(
        &install_req.mysql_host,
        install_req.mysql_port,
        &install_req.mysql_name,
        &install_req.mysql_user,
        &install_req.mysql_pwd,
        &install_req.mysql_pre,
        &install_req.redis_host,
        install_req.redis_port,
        install_req.redis_pwd.as_deref(),
        &adm_pwd_key,
        install_req.tls_enabled,
        install_req.cert_path.as_deref(),
        install_req.key_path.as_deref(),
    );
    
    if let Err(e) = fs::write("config.yaml", &config_content) {
        tracing::error!("创建配置文件失败: {}", e);
        res.render(Json(ApiResponse::<()>::error("创建配置文件失败", 201)));
        return;
    }

    let auth_result = serde_json::json!({
        "state": true,
        "msg": "安装成功，请重启服务"
    });

    res.render(Json(ApiResponse::success("安装成功，请重启服务", Some(auth_result))));
}

/// 生成 config.yaml 文件内容
/// 
/// 敏感信息会被加密：
/// - MySQL 密码
/// - Redis 密码
/// - admin.keys (密码密钥)
/// - admin.token_key (JWT 密钥)
/// 
/// 加密密钥为 app.code
fn generate_config_yaml(
    mysql_host: &str,
    mysql_port: u16,
    mysql_name: &str,
    mysql_user: &str,
    mysql_pwd: &str,
    mysql_pre: &str,
    redis_host: &str,
    redis_port: u16,
    redis_pwd: Option<&str>,
    adm_pwd_key: &str,
    tls_enabled: bool,
    cert_path: Option<&str>,
    key_path: Option<&str>,
) -> String {
    // 生成随机密钥
    let app_code = generate_random_string(32);
    let adm_jwt_key = generate_random_string(64);
    
    // 加密敏感信息
    let encrypted_mysql_pwd = encrypt(mysql_pwd, &app_code)
        .expect("MySQL密码加密失败");
    let encrypted_redis_pwd = redis_pwd
        .map(|p| encrypt(p, &app_code).expect("Redis密码加密失败"))
        .unwrap_or_default();
    let encrypted_adm_pwd_key = encrypt(adm_pwd_key, &app_code)
        .expect("管理员密码密钥加密失败");
    let encrypted_adm_jwt_key = encrypt(&adm_jwt_key, &app_code)
        .expect("JWT密钥加密失败");
    
    // 构建TLS配置部分
    let tls_config = if tls_enabled {
        let cert_line = cert_path
            .map(|p| format!("\n    cert_path: {}", p))
            .unwrap_or_default();
        let key_line = key_path
            .map(|p| format!("\n    key_path: {}", p))
            .unwrap_or_default();
        format!("    tls_enabled: true{}{}", cert_line, key_line)
    } else {
        "    tls_enabled: false".to_string()
    };
    
    // 根据TLS状态设置host协议
    let host_protocol = if tls_enabled { "https" } else { "http" };
    
    format!(
        r#"app:
    host: {}://127.0.0.1:8080
    code: "{}"
    upload_dir: ./data/upload
    upload_size: 2
    cache: false
    user_api_rewrite: false
    output_msg: true
    ver: 3.3
    wx_appid: ""
    wx_secret: ""
    qq_appid: ""
    qq_appkey: ""
    admin:
        path: admin
        keys: {}
        token_exp: 86400
        token_key: {}
server:
    port: 8080
{}
mysql:
    host: {}
    port: {}
    user: {}
    password: "{}"
    dbname: {}
    prefix: {}
    charset: utf8mb4
    log_level: debug
    max_open_conns: 150
    max_idle_conns: 20
redis:
    host: {}
    port: {}
    password: "{}"
    db: 0
    prefix: re_
log:
    path: ./log
    max_size: 100
    show_line: true
    level: debug
debug:
    debug: false
i18n:
  default_language: "zh-CN"
  supported_languages:
    - "zh-CN"
    - "en"
    - "ja"
  resources_path: "./Nakamasa-Ichika/locales"
  cookie_name: "lang"
  query_param: "lang"
  header_name: "Accept-Language"
"#,
        host_protocol,
        app_code,
        encrypted_adm_pwd_key,
        encrypted_adm_jwt_key,
        tls_config,
        mysql_host,
        mysql_port,
        mysql_user,
        encrypted_mysql_pwd,
        mysql_name,
        mysql_pre,
        redis_host,
        redis_port,
        encrypted_redis_pwd
    )
}

#[handler]
pub async fn env(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    let hostname = match hostname::get() {
        Ok(name) => name.to_string_lossy().to_string(),
        Err(_) => "unknown".to_string(),
    };

    let env_info = serde_json::json!({
        "name": hostname,
        "software": true,
        "php": true,
        "os": true,
        "redis": true,
        "ue": true,
        "mysql": true,
        "config_dir": true,
        "config_db": true,
        "config_app": true,
        "config_cache": true,
        "normal": true
    });

    res.render(Json(ApiResponse::success("ok", Some(env_info))));
}

#[handler]
pub async fn check(_req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    if is_installed() {
        res.render(Json(ApiResponse::<()>::success("ok", None)));
    } else {
        res.render(Json(ApiResponse::<()>::error("not installed", -2)));
    }
}