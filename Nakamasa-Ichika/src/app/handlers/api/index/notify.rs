//! 支付异步通知
//! 
//! 逻辑流程：
//! 1. 从URL获取订单号
//! 2. 查询订单信息（包含appid）
//! 3. 根据appid查询应用的支付配置
//! 4. 调用对应支付插件的notify验证
//! 5. 验证通过后更新订单状态

use salvo::prelude::*;
use std::sync::Arc;
use chrono::Utc;
use sqlx::Row;

use crate::core::AppState;
use crate::core::regex_cache::{XML_CDATA_REGEX, XML_PLAIN_REGEX};
use crate::app::plugins::pay::{PayPlugin, JiePayPlugin, AliPayPlugin, WxPayPlugin};

/// 创建支付插件实例
fn create_plugin(pay_type: &str, config: &serde_json::Value) -> Result<Box<dyn PayPlugin>, String> {
    let mut plugin: Box<dyn PayPlugin> = match pay_type {
        "jie" => Box::new(JiePayPlugin::new()),
        "ali" => Box::new(AliPayPlugin::new()),
        "wx" => Box::new(WxPayPlugin::new()),
        _ => return Err(format!("不支持的支付类型: {}", pay_type)),
    };
    plugin.init(config.clone())?;
    Ok(plugin)
}

/// 更新订单状态
async fn update_order(
    db: &sqlx::MySqlPool,
    order: &sqlx::mysql::MySqlRow,
    trade_no: &str,
    app_type: &str,
) -> bool {
    let order_id: i64 = order.get("id");
    let uid: i64 = order.get("uid");
    let appid: i64 = order.get("appid");
    let order_type: String = order.get("type");
    let val: i64 = order.get("val");
    let inviter_id: Option<i64> = order.try_get("inviter_id").ok();
    let divide_money: i64 = order.try_get("divide_money").unwrap_or(0);

    // 开启事务
    let mut tx = match db.begin().await {
        Ok(t) => t,
        Err(_) => return false,
    };

    // 更新订单状态
    let update_result = sqlx::query(
        "UPDATE u_order SET state = 2, trade_no = ?, end_time = ? WHERE id = ?"
    )
    .bind(trade_no)
    .bind(Utc::now().timestamp())
    .bind(order_id)
    .execute(&mut *tx)
    .await;

    if update_result.is_err() {
        let _ = tx.rollback().await;
        return false;
    }

    // 卡密应用只处理余额充值
    if app_type == "kami" && order_type != "balance" {
        let _ = tx.rollback().await;
        return false;
    }

    // 代理分账
    if let Some(inv_uid) = inviter_id
        && divide_money > 0 {
            let _ = sqlx::query(
                "UPDATE u_agent SET money = money + ? WHERE uid = ? AND appid = ?"
            )
            .bind(divide_money)
            .bind(inv_uid)
            .bind(appid)
            .execute(&mut *tx)
            .await;
        }

    // 根据订单类型处理
    match order_type.as_str() {
        "vip" => {
            // 查询用户当前VIP状态
            let vip_result: Result<Option<(i64,)>, _> = sqlx::query_as(
                "SELECT vip FROM u_user WHERE id = ?"
            )
            .bind(uid)
            .fetch_optional(&mut *tx)
            .await;

            if let Ok(Some((current_vip,))) = vip_result {
                let new_vip = if current_vip >= 9999999999 {
                    current_vip
                } else if current_vip > Utc::now().timestamp() {
                    current_vip + val
                } else {
                    Utc::now().timestamp() + val
                };

                if sqlx::query("UPDATE u_user SET vip = ? WHERE id = ?")
                    .bind(new_vip)
                    .bind(uid)
                    .execute(&mut *tx)
                    .await
                    .is_err()
                {
                    let _ = tx.rollback().await;
                    return false;
                }
            }
        }
        "fen" => {
            if sqlx::query("UPDATE u_user SET fen = fen + ? WHERE id = ?")
                .bind(val)
                .bind(uid)
                .execute(&mut *tx)
                .await
                .is_err()
            {
                let _ = tx.rollback().await;
                return false;
            }
        }
        "agent" => {
            // 查询代理组
            let group_result: Result<Option<(i64, Option<i32>, Option<i32>)>, _> = sqlx::query_as(
                "SELECT id, pay_divide, km_discount FROM u_agent_group WHERE id = ? AND appid = ?"
            )
            .bind(val)
            .bind(appid)
            .fetch_optional(&mut *tx)
            .await;

            if let Ok(Some((aggid, pay_divide, km_discount))) = group_result {
                // 检查是否已是代理
                let agent_result: Result<Option<(i64, Option<i32>, Option<i32>)>, _> = sqlx::query_as(
                    "SELECT id, pay_divide, km_discount FROM u_agent WHERE uid = ? AND appid = ?"
                )
                .bind(uid)
                .bind(appid)
                .fetch_optional(&mut *tx)
                .await;

                if let Ok(Some((agent_id, old_pay_divide, old_km_discount))) = agent_result {
                    // 更新代理等级
                    if old_pay_divide.unwrap_or(0) < pay_divide.unwrap_or(0)
                        || old_km_discount.unwrap_or(100) > km_discount.unwrap_or(100)
                    {
                        let _ = sqlx::query(
                            "UPDATE u_agent SET pay_divide = GREATEST(pay_divide, ?), km_discount = LEAST(km_discount, ?) WHERE id = ?"
                        )
                        .bind(pay_divide.unwrap_or(0))
                        .bind(km_discount.unwrap_or(100))
                        .bind(agent_id)
                        .execute(&mut *tx)
                        .await;
                    }
                } else {
                    // 新开通代理
                    let _ = sqlx::query(
                        "INSERT INTO u_agent (aggid, uid, pay_divide, km_discount, time, appid) VALUES (?, ?, ?, ?, ?, ?)"
                    )
                    .bind(aggid)
                    .bind(uid)
                    .bind(pay_divide.unwrap_or(0))
                    .bind(km_discount.unwrap_or(100))
                    .bind(Utc::now().timestamp())
                    .bind(appid)
                    .execute(&mut *tx)
                    .await;
                }
            } else {
                let _ = tx.rollback().await;
                return false;
            }
        }
        "balance" => {
            if sqlx::query("UPDATE u_agent SET money = money + ? WHERE uid = ? AND appid = ?")
                .bind(val)
                .bind(uid)
                .bind(appid)
                .execute(&mut *tx)
                .await
                .is_err()
            {
                let _ = tx.rollback().await;
                return false;
            }
        }
        _ => {}
    }

    tx.commit().await.is_ok()
}

/// 获取支付插件的通知数据
async fn get_notify_data(req: &mut Request, ptype: &str) -> serde_json::Value {
    match ptype {
        "jie" => {
            // 皆网支付使用GET参数
            let mut data = serde_json::Map::new();
            for (key, value) in req.queries().iter() {
                data.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
            serde_json::Value::Object(data)
        }
        "ali" => {
            // 支付宝使用表单POST
            if let Ok(bytes) = req.payload().await {
                let body = String::from_utf8_lossy(bytes);
                let mut data = serde_json::Map::new();
                for pair in body.split('&') {
                    if let Some(pos) = pair.find('=') {
                        let key = urlencoding::decode(&pair[..pos]).unwrap_or_default().to_string();
                        let value = urlencoding::decode(&pair[pos + 1..]).unwrap_or_default().to_string();
                        data.insert(key, serde_json::Value::String(value));
                    }
                }
                serde_json::Value::Object(data)
            } else {
                serde_json::Value::Null
            }
        }
        "wx" => {
            // 微信使用XML
            if let Ok(bytes) = req.payload().await {
                let body = String::from_utf8_lossy(bytes);
                parse_xml_to_json(&body)
            } else {
                serde_json::Value::Null
            }
        }
        _ => serde_json::Value::Null,
    }
}

/// 支付宝异步通知
#[handler]
pub async fn ali_notify(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();

    // 获取订单号
    let order_no = match req.param::<String>("order_no") {
        Some(no) => no,
        None => {
            res.render(Text::Plain("fail"));
            return;
        }
    };

    // 查询订单
    let order = match sqlx::query(
        "SELECT * FROM u_order WHERE order_no = ? AND payment = ?"
    )
    .bind(&order_no)
    .bind("ali")
    .fetch_optional(app_state.get_db())
    .await
    {
        Ok(Some(o)) => o,
        _ => {
            res.render(Text::Plain("fail"));
            return;
        }
    };

    // 已处理订单直接返回成功
    let state: i32 = order.get("state");
    if state != 0 {
        res.render(Text::Plain("success"));
        return;
    }

    // 获取应用支付配置
    let appid: i64 = order.get("appid");
    let app = match sqlx::query(
        "SELECT app_type, pay_ali_type, pay_ali_config FROM u_app WHERE id = ?"
    )
    .bind(appid)
    .fetch_optional(app_state.get_db())
    .await
    {
        Ok(Some(a)) => a,
        _ => {
            res.render(Text::Plain("fail"));
            return;
        }
    };

    let app_type: String = app.get("app_type");
    let pay_type: Option<String> = app.try_get("pay_ali_type").ok();
    let pay_config: Option<String> = app.try_get("pay_ali_config").ok();

    // 解析配置
    let config: serde_json::Value = match pay_config {
        Some(c) => serde_json::from_str(&c).unwrap_or(serde_json::Value::Null),
        _ => {
            res.render(Text::Plain("fail"));
            return;
        }
    };

    // 创建插件并验证
    let plugin = match create_plugin(&pay_type.unwrap_or_else(|| "ali".to_string()), &config) {
        Ok(p) => p,
        _ => {
            res.render(Text::Plain("fail"));
            return;
        }
    };

    let notify_data = get_notify_data(req, "ali").await;
    let trade_no = match plugin.verify_notify(notify_data) {
        Ok(t) => t,
        _ => {
            res.render(Text::Plain("fail"));
            return;
        }
    };

    // 更新订单
    if update_order(app_state.get_db(), &order, &trade_no, &app_type).await {
        res.render(Text::Plain("success"));
    } else {
        res.render(Text::Plain("fail"));
    }
}

/// 微信异步通知
#[handler]
pub async fn wx_notify(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let app_state = depot.obtain::<Arc<AppState>>().unwrap();

    // 获取订单号
    let order_no = match req.param::<String>("order_no") {
        Some(no) => no,
        None => {
            res.render(Text::Plain("fail"));
            return;
        }
    };

    // 查询订单
    let order = match sqlx::query(
        "SELECT * FROM u_order WHERE order_no = ? AND payment = ?"
    )
    .bind(&order_no)
    .bind("wx")
    .fetch_optional(app_state.get_db())
    .await
    {
        Ok(Some(o)) => o,
        _ => {
            res.render(Text::Plain("fail"));
            return;
        }
    };

    // 已处理订单直接返回成功
    let state: i32 = order.get("state");
    if state != 0 {
        res.render(Text::Plain("success"));
        return;
    }

    // 获取应用支付配置
    let appid: i64 = order.get("appid");
    let app = match sqlx::query(
        "SELECT app_type, pay_wx_type, pay_wx_config FROM u_app WHERE id = ?"
    )
    .bind(appid)
    .fetch_optional(app_state.get_db())
    .await
    {
        Ok(Some(a)) => a,
        _ => {
            res.render(Text::Plain("fail"));
            return;
        }
    };

    let app_type: String = app.get("app_type");
    let pay_type: Option<String> = app.try_get("pay_wx_type").ok();
    let pay_config: Option<String> = app.try_get("pay_wx_config").ok();

    // 解析配置
    let config: serde_json::Value = match pay_config {
        Some(c) => serde_json::from_str(&c).unwrap_or(serde_json::Value::Null),
        _ => {
            res.render(Text::Plain("fail"));
            return;
        }
    };

    // 创建插件并验证
    let plugin = match create_plugin(&pay_type.unwrap_or_else(|| "wx".to_string()), &config) {
        Ok(p) => p,
        _ => {
            res.render(Text::Plain("fail"));
            return;
        }
    };

    let notify_data = get_notify_data(req, "wx").await;
    let trade_no = match plugin.verify_notify(notify_data) {
        Ok(t) => t,
        _ => {
            res.render(Text::Plain("fail"));
            return;
        }
    };

    // 更新订单
    if update_order(app_state.get_db(), &order, &trade_no, &app_type).await {
        res.render(Text::Plain("success"));
    } else {
        res.render(Text::Plain("fail"));
    }
}

/// 解析XML为JSON - 使用预编译正则
fn parse_xml_to_json(xml: &str) -> serde_json::Value {
    let mut result = serde_json::Map::new();

    // 使用预编译的 CDATA 正则
    for cap in XML_CDATA_REGEX.captures_iter(xml) {
        if let (Some(k), Some(v)) = (cap.get(1), cap.get(2)) {
            result.insert(k.as_str().to_string(), serde_json::Value::String(v.as_str().to_string()));
        }
    }

    // 使用预编译的普通内容正则
    for cap in XML_PLAIN_REGEX.captures_iter(xml) {
        if let (Some(k), Some(v)) = (cap.get(1), cap.get(2))
            && !result.contains_key(k.as_str()) {
                result.insert(k.as_str().to_string(), serde_json::Value::String(v.as_str().to_string()));
            }
    }

    serde_json::Value::Object(result)
}