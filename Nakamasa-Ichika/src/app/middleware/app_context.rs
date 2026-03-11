//! 应用上下文中间件
//! 一比一还原PHP base/user.php的__init方法逻辑
//! 从路由路径中提取 appid、ver_key、ver_val，并查询 app 数据存入 depot
//! 优化版：使用 FastJson 减少序列化开销，使用 Cow 减少字符串分配

use salvo::prelude::*;
use crate::core::AppState;
use crate::core::json_optimize::FastJson;
use crate::app::utils::response::ApiResponse;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use sqlx::Row;

/// 应用信息（从数据库查询）
/// 一比一还原PHP的$this->app结构
#[derive(Debug, Clone, Serialize)]
pub struct AppInfo {
    pub id: u64,
    pub app_key: String,
    pub app_type: String,          // 'user' or 'kami'
    pub app_name: String,
    pub app_logo: Option<String>,
    pub app_state: String,         // 'y' or 'off'
    pub app_off_msg: Option<String>,
    pub logon_state: String,       // 'on' or 'off'
    pub logon_off_msg: Option<String>,
    pub logon_sn_num: i32,         // 设备绑定数量上限
    pub logon_sn_dk: String,       // 是否允许同设备多开 'y' or 'n'
    pub logon_token_exp: i32,      // token过期时间(秒)
    pub logon_ban_expire: String,  // 是否禁止到期卡密登录 'y' or 'n'
    pub reg_state: String,         // 'on' or 'off'
    pub reg_off_msg: Option<String>,
    pub reg_way: String,           // 'phone', 'email', 'wordnum'
    pub reg_time_ip: i32,          // IP重复注册间隔(小时)
    pub reg_time_sn: i32,          // 设备重复注册间隔(小时)
    pub reg_award: String,         // 注册奖励类型 'vip' or 'fen'
    pub reg_award_val: i64,
    pub reg_is_inviter: String,    // 是否需要邀请人 'y' or 'n'
    pub inviter_award: String,     // 邀请人奖励类型
    pub inviter_award_val: i64,
    pub invitee_award: String,     // 受邀者奖励类型
    pub invitee_award_val: i64,
    pub vc_time: i32,              // 验证码有效期(分钟)
    pub vc_length: i32,            // 验证码长度
    pub diary_award: String,       // 签到奖励类型
    pub diary_award_val: i32,
    pub logon_open_wxconfig: Option<String>,
    pub logon_open_qqconfig: Option<String>,
    pub smtp_state: String,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i32>,
    pub smtp_user: Option<String>,
    pub smtp_pass: Option<String>,
    pub sms_state: String,
    pub sms_config: Option<String>,
    pub sms_type: Option<String>,
    pub logon_sn_unbdeVal: i32,    // 解绑设备消耗数值
    pub logon_sn_unbdeType: String, // 解绑设备消耗类型 'vip' or 'fen'
    /// 支付宝支付配置
    pub alipay_state: String,      // 'on' or 'off'
    pub alipay_type: String,       // 支付插件类型 'jie', 'ali' 等
    pub alipay_config: Option<Vec<u8>>,
    /// 微信支付配置
    pub wechat_pay_state: String,  // 'on' or 'off'
    pub wechat_pay_type: String,   // 支付插件类型 'jie', 'wx' 等
    pub wechat_pay_config: Option<Vec<u8>>,
    /// 版本信息
    pub ver: VersionInfo,
    /// 加密配置
    pub mi: Option<EncryptionInfo>,
}

/// 版本信息
#[derive(Debug, Clone, Serialize)]
pub struct VersionInfo {
    pub ver_state: String,         // 'on' or 'off'
    pub ver_off_msg: Option<String>,
    pub ver_url: Option<String>,
    pub ver_content: Option<String>,
}

/// 加密配置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInfo {
    #[serde(rename = "type")]
    pub enc_type: String,          // 加密类型
    pub config: serde_json::Value, // 加密配置JSON
    pub sign: String,              // 是否启用签名 'y' or 'n'
    pub time: i32,                 // 时间戳验证(秒)，0表示不验证
}

/// 应用上下文中间件
/// 一比一还原PHP的__init方法
pub struct AppContext {
    /// 是否检查API路由
    pub api_router_check: bool,
    /// 是否进行数据校验（加密解密）
    pub data_check: bool,
}

impl AppContext {
    #[inline]
    pub fn new() -> Self {
        Self {
            api_router_check: true,
            data_check: true,
        }
    }
    
    /// 跳过API路由检查
    #[inline]
    pub fn skip_router_check(mut self) -> Self {
        self.api_router_check = false;
        self
    }
    
    /// 跳过数据校验
    #[inline]
    pub fn skip_data_check(mut self) -> Self {
        self.data_check = false;
        self
    }
}

impl Default for AppContext {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Handler for AppContext {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        // 提前克隆避免借用冲突
        let app_state = match depot.obtain::<Arc<AppState>>() {
            Ok(state) => state.clone(),
            Err(_) => {
                res.render(Json(ApiResponse::<()>::error_static("server error", 201)));
                ctrl.skip_rest();
                return;
            }
        };

        // 从路径参数中提取 appid、ver_key 和 ver_val
        let appid = match req.param::<u64>("appid") {
            Some(id) => id,
            None => {
                if self.api_router_check {
                    res.render(Json(ApiResponse::<()>::error_static("api error", 201)));
                    ctrl.skip_rest();
                    return;
                }
                ctrl.call_next(req, depot, res).await;
                return;
            }
        };

        let ver_key = req.param::<String>("ver_key");
        let ver_val = req.param::<String>("ver_val");

        // 验证 appid
        if appid == 0 {
            res.render(Json(ApiResponse::<()>::error_static("appid error", 201)));
            ctrl.skip_rest();
            return;
        }

        // 查询应用信息
        let app_info = match fetch_app_info_with_version(&app_state, appid, ver_key.as_deref(), ver_val.as_deref()).await {
            Ok(Some(info)) => info,
            Ok(None) => {
                res.render(Json(ApiResponse::<()>::error_static("appid error", 201)));
                ctrl.skip_rest();
                return;
            }
            Err(e) => {
                tracing::error!("数据库查询失败: {}", e);
                res.render(Json(ApiResponse::<()>::error_static("数据库错误", 201)));
                ctrl.skip_rest();
                return;
            }
        };

        // 检查应用状态
        if app_info.app_state == "off" {
            let msg = app_info.app_off_msg.clone().unwrap_or_else(|| "应用已关闭".to_string());
            res.render(Json(ApiResponse::<()>::error(msg, 100)));
            ctrl.skip_rest();
            return;
        }

        // 检查版本状态
        if app_info.ver.ver_state == "off" {
            let msg = app_info.ver.ver_off_msg.clone().unwrap_or_else(|| "版本已关闭".to_string());
            res.render(Json(ApiResponse::<()>::error(msg, 101)));
            ctrl.skip_rest();
            return;
        }

        // 存储到Depot供后续使用
        depot.insert("app_appid", appid);
        depot.insert("app_version_index", ver_key);
        depot.insert("app_version", ver_val);
        depot.insert("app_info", app_info);

        // 继续执行下一个处理器
        ctrl.call_next(req, depot, res).await;
    }
}

/// 查询应用信息（包含版本和加密配置）
/// 优化版：使用单次JOIN查询替代三次独立查询，减少数据库往返
async fn fetch_app_info_with_version(
    app_state: &Arc<AppState>, 
    appid: u64, 
    ver_key: Option<&str>, 
    ver_val: Option<&str>
) -> Result<Option<AppInfo>, sqlx::Error> {
    // 解析版本号 - 在查询前完成，避免作用域问题
    let version_info: Option<(String, i32, i32, i32)> = if let (Some(vk), Some(vv)) = (ver_key, ver_val) {
        let parts: Vec<&str> = vv.split('.').collect();
        let (major, minor, patch) = if parts.len() >= 3 {
            (parts[0].parse().unwrap_or(0), parts[1].parse().unwrap_or(0), parts[2].parse().unwrap_or(0))
        } else if parts.len() == 2 {
            (parts[0].parse().unwrap_or(0), parts[1].parse().unwrap_or(0), 0)
        } else if parts.len() == 1 {
            (parts[0].parse().unwrap_or(0), 0, 0)
        } else {
            (0, 0, 0)
        };
        Some((vk.to_string(), major, minor, patch))
    } else {
        None
    };

    // 单次JOIN查询获取所有数据
    let row = sqlx::query(
        r#"
        SELECT 
            A.id, A.app_key, A.app_type, A.app_name, A.app_logo,
            A.app_state, A.app_off_msg, A.logon_state, A.logon_off_msg,
            A.logon_sn_num, A.logon_sn_dk, A.logon_token_exp, A.logon_ban_expire,
            A.reg_state, A.reg_off_msg, A.reg_way, A.reg_time_ip, A.reg_time_sn,
            A.reg_award, A.reg_award_val, A.reg_is_inviter,
            A.inviter_award, A.inviter_award_val, A.invitee_award, A.invitee_award_val,
            A.vc_time, A.vc_length, A.diary_award, A.diary_award_val,
            A.logon_open_wxconfig, A.logon_open_qqconfig,
            A.smtp_state, A.smtp_host, A.smtp_port, A.smtp_user, A.smtp_pass,
            A.sms_state, A.sms_config, A.sms_type,
            A.logon_sn_unbde_val, A.logon_sn_unbde_type,
            A.pay_ali_state, A.pay_ali_type, A.pay_ali_config,
            A.pay_wx_state, A.pay_wx_type, A.pay_wx_config,
            V.ver_state, V.ver_off_msg, V.ver_url, V.ver_content, V.mid,
            M.type, M.config, M.sign, M.time
        FROM u_app A
        LEFT JOIN u_app_ver V ON V.appid = A.id 
            AND (? IS NULL OR V.ver_key = ?)
            AND (? IS NULL OR V.ver_major = ?)
            AND (? IS NULL OR V.ver_minor = ?)
            AND (? IS NULL OR V.ver_patch = ?)
        LEFT JOIN u_app_mi M ON M.id = V.mid
        WHERE A.id = ?
        "#
    )
    .bind(version_info.as_ref().map(|v| v.0.clone()))
    .bind(version_info.as_ref().map(|v| v.0.clone()))
    .bind(version_info.as_ref().map(|v| v.1))
    .bind(version_info.as_ref().map(|v| v.1))
    .bind(version_info.as_ref().map(|v| v.2))
    .bind(version_info.as_ref().map(|v| v.2))
    .bind(version_info.as_ref().map(|v| v.3))
    .bind(version_info.as_ref().map(|v| v.3))
    .bind(appid)
    .fetch_optional(app_state.get_db())
    .await?;

    let row = match row {
        Some(r) => r,
        None => return Ok(None),
    };

    // 解析版本信息
    let ver_state: String = row.try_get::<Option<String>, _>(47)?.unwrap_or_else(|| "on".to_string());
    if ver_state.is_empty() {
        return Ok(None);
    }

    let ver = VersionInfo {
        ver_state,
        ver_off_msg: row.try_get(48).ok(),
        ver_url: row.try_get(49).ok(),
        ver_content: row.try_get(50).ok(),
    };

    // 解析加密配置
    let mi: Option<EncryptionInfo> = match row.try_get::<Option<String>, _>(51)? {
        Some(enc_type) => {
            let config_str: Option<String> = row.try_get(52).ok();
            let config = config_str
                .and_then(|s| FastJson::parse_borrowed(&s).ok())
                .unwrap_or(serde_json::Value::Null);
            Some(EncryptionInfo {
                enc_type,
                config,
                sign: row.try_get::<Option<String>, _>(53)?.unwrap_or_else(|| "n".to_string()),
                time: row.try_get::<Option<i32>, _>(54)?.unwrap_or(0),
            })
        }
        None => None,
    };

    Ok(Some(AppInfo {
        id: row.try_get(0)?,
        app_key: row.try_get(1)?,
        app_type: row.try_get(2)?,
        app_name: row.try_get(3)?,
        app_logo: row.try_get(4).ok(),
        app_state: row.try_get(5)?,
        app_off_msg: row.try_get(6).ok(),
        logon_state: row.try_get(7)?,
        logon_off_msg: row.try_get(8).ok(),
        logon_sn_num: row.try_get::<Option<i32>, _>(9)?.unwrap_or(3),
        logon_sn_dk: row.try_get::<Option<String>, _>(10)?.unwrap_or_else(|| "n".to_string()),
        logon_token_exp: row.try_get::<Option<i32>, _>(11)?.unwrap_or(86400),
        logon_ban_expire: row.try_get::<Option<String>, _>(12)?.unwrap_or_else(|| "n".to_string()),
        reg_state: row.try_get(13)?,
        reg_off_msg: row.try_get(14).ok(),
        reg_way: row.try_get::<Option<String>, _>(15)?.unwrap_or_else(|| "wordnum".to_string()),
        reg_time_ip: row.try_get::<Option<i32>, _>(16)?.unwrap_or(0),
        reg_time_sn: row.try_get::<Option<i32>, _>(17)?.unwrap_or(0),
        reg_award: row.try_get::<Option<String>, _>(18)?.unwrap_or_else(|| "fen".to_string()),
        reg_award_val: row.try_get::<Option<i64>, _>(19)?.unwrap_or(0),
        reg_is_inviter: row.try_get::<Option<String>, _>(20)?.unwrap_or_else(|| "n".to_string()),
        inviter_award: row.try_get::<Option<String>, _>(21)?.unwrap_or_else(|| "fen".to_string()),
        inviter_award_val: row.try_get::<Option<i64>, _>(22)?.unwrap_or(0),
        invitee_award: row.try_get::<Option<String>, _>(23)?.unwrap_or_else(|| "fen".to_string()),
        invitee_award_val: row.try_get::<Option<i64>, _>(24)?.unwrap_or(0),
        vc_time: row.try_get::<Option<i32>, _>(25)?.unwrap_or(10),
        vc_length: row.try_get::<Option<i32>, _>(26)?.unwrap_or(6),
        diary_award: row.try_get::<Option<String>, _>(27)?.unwrap_or_else(|| "fen".to_string()),
        diary_award_val: row.try_get::<Option<i32>, _>(28)?.unwrap_or(0),
        logon_open_wxconfig: row.try_get(29).ok(),
        logon_open_qqconfig: row.try_get(30).ok(),
        smtp_state: row.try_get::<Option<String>, _>(31)?.unwrap_or_else(|| "off".to_string()),
        smtp_host: row.try_get(32).ok(),
        smtp_port: row.try_get(33).ok(),
        smtp_user: row.try_get(34).ok(),
        smtp_pass: row.try_get(35).ok(),
        sms_state: row.try_get::<Option<String>, _>(36)?.unwrap_or_else(|| "off".to_string()),
        sms_config: row.try_get(37).ok(),
        sms_type: row.try_get(38).ok(),
        logon_sn_unbdeVal: row.try_get::<Option<i32>, _>(39)?.unwrap_or(0),
        logon_sn_unbdeType: row.try_get::<Option<String>, _>(40)?.unwrap_or_else(|| "fen".to_string()),
        alipay_state: row.try_get::<Option<String>, _>(41)?.unwrap_or_else(|| "off".to_string()),
        alipay_type: row.try_get::<Option<String>, _>(42)?.unwrap_or_else(|| "jie".to_string()),
        alipay_config: row.try_get(43).ok(),
        wechat_pay_state: row.try_get::<Option<String>, _>(44)?.unwrap_or_else(|| "off".to_string()),
        wechat_pay_type: row.try_get::<Option<String>, _>(45)?.unwrap_or_else(|| "jie".to_string()),
        wechat_pay_config: row.try_get(46).ok(),
        ver,
        mi,
    }))
}