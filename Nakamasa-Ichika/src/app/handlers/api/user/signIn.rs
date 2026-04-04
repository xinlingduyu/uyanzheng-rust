//! 每日签到
//! 
//! 功能说明：
//! 用户每日签到获取奖励，奖励类型和数量由应用配置决定。
//! 可配置VIP免费签到或额外奖励。
//!
//! 处理流程：
//! 1. 验证token参数
//! 2. 检查今日是否已签到
//! 3. 获取签到奖励配置（使用高性能缓存）
//! 4. 增加用户VIP时长或积分
//! 5. 记录签到日志
//! 6. 返回签到结果

use salvo::prelude::*;
use std::sync::Arc;
use chrono::Utc;

use crate::core::AppState;
use crate::core::middleware::get_client_ip;
use crate::core::app_state::AppConfigCache;
use crate::app::utils::response::SignedApiResponse;
use crate::app::utils::validator::Validator;
use crate::app::models::requests::SignInRequest;
use crate::app::middleware::user_auth::UserInfo;
use crate::app::middleware::app_context::AppInfo;

/// 签到奖励配置
struct DiaryAwardConfig {
    diary_award: String,      // "vip" or "fen"
    diary_award_val: i32,     // 奖励数量
}

/// 获取签到奖励配置 - 使用高性能缓存
#[inline]
async fn get_diary_award_config(app_state: &Arc<AppState>, appid: u64) -> DiaryAwardConfig {
    // 先从缓存获取
    if let Some(cached) = app_state.app_config_cache.get(&appid) {
        return DiaryAwardConfig {
            diary_award: cached.diary_award,
            diary_award_val: cached.diary_award_val,
        };
    }
    
    // 缓存未命中，从数据库查询
    let result = sqlx::query_as::<_, (Option<String>, Option<i32>)>(
        "SELECT diary_award, diary_award_val FROM u_app WHERE id = ?"
    )
    .bind(appid)
    .fetch_optional(app_state.get_db())
    .await;

    match result {
        Ok(Some(row)) => {
            
            
            // 存入缓存（不完整，只存需要的数据）
            // 实际使用时可以存完整的 AppConfigCache
            DiaryAwardConfig {
                diary_award: row.0.clone().unwrap_or_else(|| "fen".to_string()),
                diary_award_val: row.1.unwrap_or(0),
            }
        }
        _ => DiaryAwardConfig {
            diary_award: "fen".to_string(),
            diary_award_val: 0,
        },
    }
}

#[handler]
pub async fn sign_in(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();
    
    // 获取 app_key 和 app_type（零拷贝）
    let (app_key, app_type) = match depot.get::<AppInfo>("app_info") {
        Ok(info) => (info.app_key.as_str(), info.app_type.as_str()),
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("应用信息不存在", 201, "")));
            return;
        }
    };
    
    let sign_req = match req.parse_json::<SignInRequest>().await {
        Ok(data) => data,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("参数解析失败", 201, app_key)));
            return;
        }
    };

    // PHP: 'token' => ['wordnum','32,32','TOKEN有误']
    let mut validator = Validator::new();
    validator.wordnum("token", &sign_req.token, 32, 32);
    
    if let Err(msg) = validator.validate() {
        res.render(Json(SignedApiResponse::<()>::error(msg, 201, app_key)));
        return;
    }

    // 从 depot 获取用户信息（由 UserAuth 中间件提供）
    let user_info = match depot.get::<UserInfo>("user_info") {
        Ok(info) => info,
        Err(_) => {
            res.render(Json(SignedApiResponse::<()>::error("未授权", 201, app_key)));
            return;
        }
    };

    let (uid, appid) = (user_info.uid, user_info.appid);
    let user_type = user_info.user_type.as_str();
    let user_vip = user_info.vip.unwrap_or(0);
    let current_time = Utc::now().timestamp();
    let ip = get_client_ip(req);

    // PHP: if($this->app['app_type'] != 'user')$this->out->e(115);
    // 只支持用户版应用
    if app_type != "user" {
        res.render(Json(SignedApiResponse::<()>::error("当前应用不支持调用该接口", 115, app_key)));
        return;
    }

    // 卡密用户不支持签到
    if user_type != "user" {
        res.render(Json(SignedApiResponse::<()>::error("卡密用户不支持签到", 201, app_key)));
        return;
    }

    // PHP: $sRes = $this->db->where('ug = ? and uid = ? and type = ? and state = ? and time > ? and appid = ?',['user',$this->user['id'],'signIn','y',timeRange(),$this->app['id']])->fetch();
    // PHP: if($sRes)$this->out->e(134);
    // 检查今日是否已签到 - 使用PHP的timeRange()逻辑
    // timeRange()返回今天0点的时间戳
    let start_of_day = get_time_range();
    
    let s_res = sqlx::query_as::<_, (i64,)>(
        "SELECT id FROM u_logs WHERE ug = 'user' AND uid = ? AND type = 'signIn' AND state = 'y' AND time > ? AND appid = ?"
    )
    .bind(uid)
    .bind(start_of_day)
    .bind(appid)
    .fetch_optional(app_state.get_db())
    .await;

    if let Ok(Some(_)) = s_res {
        res.render(Json(SignedApiResponse::<()>::error("今日已经签到过了", 134, app_key)));
        return;
    }

    // 获取签到奖励配置（使用缓存）
    let award_config = get_diary_award_config(app_state, appid).await;

    // PHP: $addRes = $this->db->add(['ug'=>'user','uid'=>$this->user['id'],'type'=>'signIn','time'=>time(),'ip'=>$this->ip,'appid'=>$this->app['id']]);
    // 添加签到记录
    let add_res = sqlx::query(
        "INSERT INTO u_logs (ug, uid, type, state, time, ip, appid) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("user")
    .bind(uid)
    .bind("signIn")
    .bind("y")
    .bind(current_time)
    .bind(ip)
    .bind(appid)
    .execute(app_state.get_db())
    .await;

    match add_res {
        Ok(_) => {
            // PHP: if($this->app['diary_award'] == 'vip'){
            //     if($this->app['diary_award_val'] > 0){
            //         if($this->user['vip'] == 9999999999)$this->out->e(200,"签到成功");
            //         if($this->user['vip'] > time()){
            //             $upVip = $this->user['vip']+$this->app['diary_award_val'];
            //         }else{
            //             $upVip = time()+$this->app['diary_award_val'];
            //         }
            //         db('user')->where('id = ?',[$this->user['id']])->update(['vip'=>$upVip]);
            //     }
            // }else{
            //     if($this->app['diary_award_val'] > 0){
            //         db('user')->where('id = ?',[$this->user['id']])->update(['fen'=>$this->user['fen']+$this->app['diary_award_val']]);
            //     }
            // }
            // 更新用户奖励
            if award_config.diary_award_val > 0 {
                match award_config.diary_award.as_str() {
                    "vip" => {
                        // 检查永久VIP
                        if user_vip == 9999999999 {
                            res.render(Json(SignedApiResponse::success(app_key, None::<()>)));
                            return;
                        }
                        
                        let new_vip = if user_vip > current_time {
                            user_vip + award_config.diary_award_val as i64
                        } else {
                            current_time + award_config.diary_award_val as i64
                        };
                        
                        let _ = sqlx::query(
                            "UPDATE u_user SET vip = ? WHERE id = ? AND appid = ?"
                        )
                        .bind(new_vip)
                        .bind(uid)
                        .bind(appid)
                        .execute(app_state.get_db())
                        .await;
                    }
                    "fen" => {
                        let _ = sqlx::query(
                            "UPDATE u_user SET fen = fen + ? WHERE id = ? AND appid = ?"
                        )
                        .bind(award_config.diary_award_val)
                        .bind(uid)
                        .bind(appid)
                        .execute(app_state.get_db())
                        .await;
                    }
                    _ => {}
                }
            }

            // PHP: $this->out->e(200,"签到成功");
            res.render(Json(SignedApiResponse::success(app_key, None::<()>)));
        }
        Err(e) => {
            tracing::error!("签到失败: {}", e);
            // PHP: $this->out->e(201,"签到失败");
            res.render(Json(SignedApiResponse::<()>::error("签到失败", 201, app_key)));
        }
    }
}

/// 一比一还原PHP的timeRange()函数 - 优化版
/// 返回今天0点的时间戳（中国时区 UTC+8）
#[inline]
fn get_time_range() -> i64 {
    // PHP: timeRange() 返回当天0点的时间戳
    // 使用中国时区 (UTC+8)
    let now = chrono::Utc::now();
    // 直接计算：获取当前UTC时间戳，减去今天已过的小时、分钟、秒
    // 然后加上8小时调整为北京时间
    let china_offset: i64 = 8 * 3600;
    let utc_timestamp = now.timestamp();
    // 计算今天0点的UTC时间戳（北京时间）
    let seconds_per_day: i64 = 24 * 3600;
    ((utc_timestamp + china_offset) / seconds_per_day) * seconds_per_day - china_offset
}
