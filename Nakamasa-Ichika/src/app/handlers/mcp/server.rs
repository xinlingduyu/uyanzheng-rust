//! MCP (Model Context Protocol) 服务端实现
//!
//! 嵌入式 MCP Server，通过 SSE 传输协议暴露 AI 可调用的工具。
//!
//! ## 端点
//!
//! - `GET /mcp/sse?app_id=xxx` — SSE 连接端点（需传入 app_id）
//! - `POST /mcp/messages?session_id=xxx` — JSON-RPC 消息端点
//!
//! ## 工具
//!
//! - `check_user` — 查询用户信息（支持手机号/邮箱/自定义账号）
//! - `list_goods` — 获取可用商品列表（使用会话绑定的 app_id）
//! - `create_payment` — 创建支付订单，需提供 account + goods_id
//! - `query_order` — 查询订单支付状态

use std::collections::HashMap;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use futures_util::Stream;
use once_cell::sync::Lazy;
use salvo::prelude::*;
use tokio::sync::broadcast;

use crate::app::plugins::pay::manager::PayPluginManager;
use crate::app::plugins::pay::{PayOrder, PayPlugin};
use crate::core::AppState;

// ============================================================================
// 会话管理
// ============================================================================

struct McpSession {
    tx: broadcast::Sender<String>,
    _rx: broadcast::Receiver<String>,
    /// 会话绑定的 app_id（建立 SSE 连接时传入）
    app_id: u64,
}

static SESSIONS: Lazy<std::sync::RwLock<HashMap<String, Arc<McpSession>>>> =
    Lazy::new(|| std::sync::RwLock::new(HashMap::new()));

fn generate_session_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let rand = rand::random::<u32>();
    format!("mcp_{:x}_{:x}", ts, rand)
}

const SERVER_CAPS: &str = r#"{"protocolVersion":"2025-03-26","capabilities":{"tools":{"listChanged":false}},"serverInfo":{"name":"nakamasa-mcp","version":"1.0.0"}}"#;

fn make_error(code: i32, message: &str) -> String {
    format!(r#"{{"jsonrpc":"2.0","id":null,"error":{{"code":{},"message":"{}"}}}}"#, code, message.replace('"', r#"\""#))
}

fn make_result(id: &serde_json::Value, result: &serde_json::Value) -> String {
    let id_json = serde_json::to_string(id).unwrap_or_else(|_| "null".to_string());
    let result_json = serde_json::to_string(result).unwrap_or_else(|_| "{}".to_string());
    format!(r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#, id_json, result_json)
}

fn make_no_id_result(result: &serde_json::Value) -> String {
    let result_json = serde_json::to_string(result).unwrap_or_else(|_| "{}".to_string());
    format!(r#"{{"jsonrpc":"2.0","id":null,"result":{}}}"#, result_json)
}

// ============================================================================
// Tool 定义
// ============================================================================

fn tools_json() -> serde_json::Value {
    serde_json::json!({
        "tools": [
            {
                "name": "check_user",
                "description": "查询用户是否存在，支持手机号/邮箱/自定义账号三种方式查找，返回uid和账号信息",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "account": { "type": "string", "description": "用户账号（手机号/邮箱/自定义账号，三种均可）" }
                    },
                    "required": ["account"]
                }
            },
            {
                "name": "list_goods",
                "description": "获取当前应用的可用商品列表（含名称、价格、类型、简介）",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "page": { "type": "integer", "description": "页码，默认1" }
                    }
                }
            },
            {
                "name": "create_payment",
                "description": "创建支付订单，需提供用户账号和商品ID，返回支付链接",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "account": { "type": "string", "description": "充值账号（手机号/邮箱/自定义账号）" },
                        "goods_id": { "type": "integer", "description": "商品ID" },
                        "channel": {
                            "type": "string",
                            "description": "支付通道: ali=支付宝, wx=微信支付",
                            "enum": ["ali", "wx"]
                        }
                    },
                    "required": ["account", "goods_id", "channel"]
                }
            },
            {
                "name": "query_order",
                "description": "查询订单支付状态",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "order_no": { "type": "string", "description": "商户订单号" }
                    },
                    "required": ["order_no"]
                }
            }
        ]
    })
}

fn get_pay_manager(state: &Arc<AppState>) -> Option<Arc<PayPluginManager>> {
    state.get_pay_manager()
}

// ============================================================================
// Tool 处理器
// ============================================================================

/// 查询用户信息（支持手机号/邮箱/自定义账号）
fn handle_check_user(args: &serde_json::Value, state: &Arc<AppState>, app_id: u64) -> String {
    let account = args.get("account").and_then(|v| v.as_str()).unwrap_or("");
    if account.is_empty() {
        return make_error(-32602, "Missing required parameter: account");
    }

    let pool = match state.db.as_ref() {
        Some(p) => p,
        None => return make_error(-32603, "Database not available"),
    };

    let rt = tokio::runtime::Handle::current();
    let result = rt.block_on(async {
        sqlx::query_as::<_, (u64, Option<String>, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, phone, email, acctno, nickname FROM u_user WHERE (phone = ? OR email = ? OR acctno = ?) AND appid = ?",
        )
        .bind(account)
        .bind(account)
        .bind(account)
        .bind(app_id)
        .fetch_optional(pool)
        .await
    });

    match result {
        Ok(Some((uid, phone, email, acctno, nickname))) => {
            let resp = serde_json::json!({
                "found": true,
                "uid": uid,
                "phone": phone,
                "email": email,
                "acctno": acctno,
                "nickname": nickname,
            });
            make_no_id_result(&resp)
        }
        Ok(None) => {
            let resp = serde_json::json!({
                "found": false,
                "uid": null,
                "note": "User not found with this account identifier"
            });
            make_no_id_result(&resp)
        }
        Err(e) => make_error(-32603, &format!("Database error: {}", e)),
    }
}

/// 获取商品列表
fn handle_list_goods(_args: &serde_json::Value, state: &Arc<AppState>, app_id: u64) -> String {
    let pool = match state.db.as_ref() {
        Some(p) => p,
        None => return make_error(-32603, "Database not available"),
    };

    let rt = tokio::runtime::Handle::current();
    let result = rt.block_on(async {
        sqlx::query_as::<_, (i64, String, String, f64, Option<String>)>(
            "SELECT id, name, type, money, blurb FROM u_goods WHERE state = 'y' AND appid = ? ORDER BY id DESC",
        )
        .bind(app_id)
        .fetch_all(pool)
        .await
    });

    match result {
        Ok(rows) => {
            let goods: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|(id, name, r#type, money, blurb)| {
                    serde_json::json!({
                        "id": id,
                        "name": name,
                        "type": r#type,
                        "money": money,
                        "blurb": blurb.unwrap_or_default(),
                    })
                })
                .collect();
            let resp = serde_json::json!({ "goods": goods, "total": goods.len() });
            make_no_id_result(&resp)
        }
        Err(e) => make_error(-32603, &format!("Database error: {}", e)),
    }
}

/// 创建支付订单
fn handle_create_payment(args: &serde_json::Value, state: &Arc<AppState>, app_id: u64) -> String {
    let account = args.get("account").and_then(|v| v.as_str()).unwrap_or("");
    if account.is_empty() {
        return make_error(-32602, "Missing required parameter: account");
    }

    let goods_id = args.get("goods_id").and_then(|v| v.as_i64()).unwrap_or(0);
    if goods_id <= 0 {
        return make_error(-32602, "Missing or invalid parameter: goods_id");
    }

    let channel = args.get("channel").and_then(|v| v.as_str()).unwrap_or("ali");
    if channel != "ali" && channel != "wx" {
        return make_error(-32602, &format!("Unsupported channel: {}", channel));
    }

    let pool = match state.db.as_ref() {
        Some(p) => p,
        None => return make_error(-32603, "Database not available"),
    };

    let rt = tokio::runtime::Handle::current();

    // 查询用户
    let user = rt.block_on(async {
        sqlx::query_as::<_, (u64,)>(
            "SELECT id FROM u_user WHERE (phone = ? OR email = ? OR acctno = ?) AND appid = ?",
        )
        .bind(account)
        .bind(account)
        .bind(account)
        .bind(app_id)
        .fetch_optional(pool)
        .await
    });

    let uid = match user {
        Ok(Some((uid,))) => uid,
        Ok(None) => return make_error(-32602, "User account not found"),
        Err(e) => return make_error(-32603, &format!("Database error when querying user: {}", e)),
    };

    // 查询商品
    let goods = rt.block_on(async {
        sqlx::query_as::<_, (i64, String, String, f64, i64, String)>(
            "SELECT id, name, type, money, val, state FROM u_goods WHERE id = ? AND appid = ?",
        )
        .bind(goods_id)
        .bind(app_id)
        .fetch_optional(pool)
        .await
    });

    let (gid, goods_name, goods_type, money, val, goods_state) = match goods {
        Ok(Some(row)) => row,
        Ok(None) => return make_error(-32602, "Goods not found"),
        Err(e) => return make_error(-32603, &format!("Database error when querying goods: {}", e)),
    };

    if goods_state != "y" {
        return make_error(-32602, "Goods is no longer available");
    }

    // 生成订单号
    let current_time = Utc::now().timestamp();
    let order_no = format!(
        "MCP{}{:05}",
        Utc::now().format("%Y%m%d%H%M%S"),
        rand::random::<u16>()
    );

    // 插入订单
    let insert = rt.block_on(async {
        sqlx::query(
            "INSERT INTO u_order (uid, gid, order_no, name, money, type, val, pay_type, status, add_time, appid) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'wait', ?, ?)",
        )
        .bind(uid)
        .bind(gid)
        .bind(&order_no)
        .bind(&goods_name)
        .bind(money)
        .bind(&goods_type)
        .bind(val)
        .bind(channel)
        .bind(current_time)
        .bind(app_id)
        .execute(pool)
        .await
    });

    if let Err(e) = insert {
        tracing::error!("Failed to insert order: {}", e);
        return make_error(-32603, "Failed to create order record");
    }

    // 构建 PayOrder
    let pay_order = PayOrder {
        order_no: order_no.clone(),
        name: goods_name,
        money,
        notify_url: String::new(),
        return_url: String::new(),
        pay_type: "h5".to_string(),
        client_ip: None,
        scene_info: None,
    };

    match get_pay_manager(state).and_then(|mgr| {
        let plugin = mgr.get_plugin(channel).ok()?;
        plugin.create(&pay_order).ok()
    }) {
        Some(result) => {
            let resp = serde_json::json!({
                "order_no": order_no,
                "amount": money,
                "channel": channel,
                "pay_url": result.pay_url,
                "qrcode": result.qrcode,
                "message": result.message,
            });
            make_no_id_result(&resp)
        }
        None => {
            let resp = serde_json::json!({
                "order_no": order_no,
                "amount": money,
                "channel": channel,
                "status": "pending",
                "note": "Payment plugin not configured, please complete payment manually through the app"
            });
            make_no_id_result(&resp)
        }
    }
}

/// 查询订单
fn handle_query_order(args: &serde_json::Value, state: &Arc<AppState>, _app_id: u64) -> String {
    let order_no = args.get("order_no").and_then(|v| v.as_str()).unwrap_or("");
    if order_no.is_empty() {
        return make_error(-32602, "Missing required parameter: order_no");
    }

    let pool = match state.db.as_ref() {
        Some(p) => p,
        None => return make_error(-32603, "Database not available"),
    };

    let rt = tokio::runtime::Handle::current();
    let result = rt.block_on(async {
        sqlx::query_as::<_, (String, f64, String, String)>(
            "SELECT order_no, money, status, pay_type FROM u_order WHERE order_no = ?",
        )
        .bind(order_no)
        .fetch_optional(pool)
        .await
    });

    match result {
        Ok(Some((no, money, status, pay_type))) => {
            let resp = serde_json::json!({
                "order_no": no,
                "amount": money,
                "status": status,
                "channel": pay_type,
            });
            make_no_id_result(&resp)
        }
        Ok(None) => make_error(-32602, &format!("Order not found: {}", order_no)),
        Err(e) => make_error(-32603, &format!("Database error: {}", e)),
    }
}

// ============================================================================
// JSON-RPC 分发
// ============================================================================

fn dispatch(body: &str, state: &Arc<AppState>, app_id: u64) -> String {
    let req: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(e) => return make_error(-32700, &format!("Parse error: {}", e)),
    };

    let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let id = req.get("id");
    let params = req.get("params").unwrap_or(&serde_json::Value::Null);

    match method {
        "initialize" => {
            let caps: serde_json::Value = serde_json::from_str(SERVER_CAPS).unwrap_or_default();
            match id {
                Some(id_val) => make_result(id_val, &caps),
                None => make_no_id_result(&caps),
            }
        }
        "notifications/initialized" => String::new(),
        "tools/list" => {
            let tools = tools_json();
            match id {
                Some(id_val) => make_result(id_val, &tools),
                None => make_no_id_result(&tools),
            }
        }
        "tools/call" => {
            let tool_args = params.get("arguments").unwrap_or(&serde_json::Value::Null);
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let resp = match name {
                "check_user" => handle_check_user(tool_args, state, app_id),
                "list_goods" => handle_list_goods(tool_args, state, app_id),
                "create_payment" => handle_create_payment(tool_args, state, app_id),
                "query_order" => handle_query_order(tool_args, state, app_id),
                _ => make_error(-32601, &format!("Unknown tool: {}", name)),
            };
            // 如果有 id，包装为带 id 的响应
            if let Some(id_val) = id {
                if let Ok(mut resp_val) = serde_json::from_str::<serde_json::Value>(&resp) {
                    resp_val["id"] = id_val.clone();
                    return serde_json::to_string(&resp_val).unwrap_or(resp);
                }
            }
            resp
        }
        _ => make_error(-32601, &format!("Method not found: {}", method)),
    }
}

// ============================================================================
// HTTP Handlers
// ============================================================================

/// SSE 端点：`GET /mcp/sse?app_id=xxx`
/// app_id 必须传入，绑定到会话供后续工具调用使用
#[handler]
pub async fn sse_handler(req: &mut Request, res: &mut Response) {
    let app_id = req.query::<u64>("app_id").unwrap_or(0);
    if app_id == 0 {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Text::Plain(make_error(-32000, "Missing or invalid app_id query parameter")));
        return;
    }

    let (tx, rx) = broadcast::channel::<String>(256);
    let session_id = generate_session_id();

    {
        let mut sessions = SESSIONS.write().unwrap_or_else(|e| e.into_inner());
        sessions.insert(
            session_id.clone(),
            Arc::new(McpSession { tx: tx.clone(), _rx: rx, app_id }),
        );
    }

    tracing::info!("MCP SSE session created: id={}, app_id={}", session_id, app_id);

    // SSE 响应头
    res.headers_mut()
        .insert("Content-Type", "text/event-stream".parse().unwrap());
    res.headers_mut()
        .insert("Cache-Control", "no-cache".parse().unwrap());
    res.headers_mut()
        .insert("Connection", "keep-alive".parse().unwrap());
    res.headers_mut()
        .insert("X-Accel-Buffering", "no".parse().unwrap());

    // 构建 SSE 流
    let endpoint_data = format!(
        "event: endpoint\\ndata: /mcp/messages?session_id={}\\n\\n",
        session_id
    );

    let stream: Pin<Box<dyn Stream<Item = Result<String, Infallible>> + Send>> = {
        let rx = tx.subscribe();
        let sent_endpoint = false;

        Box::pin(futures_util::stream::unfold(
            (rx, sent_endpoint, endpoint_data),
            |(mut rx, sent, ep)| async move {
                if !sent {
                    return Some((Ok(ep), (rx, true, String::new())));
                }
                match rx.recv().await {
                    Ok(msg) => {
                        let sse = format!("data: {}\\n\\n", msg);
                        Some((Ok(sse), (rx, true, String::new())))
                    }
                    Err(broadcast::error::RecvError::Closed) => None,
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        Some((Ok(": keepalive\\n\\n".to_string()), (rx, true, String::new())))
                    }
                }
            },
        ))
    };

    res.stream(stream);
}

/// 消息端点：`POST /mcp/messages?session_id=xxx`
#[handler]
pub async fn messages_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let session_id = req.query::<String>("session_id").unwrap_or_default();
    if session_id.is_empty() {
        res.status_code(StatusCode::BAD_REQUEST);
        res.render(Text::Plain(make_error(-32000, "Missing session_id")));
        return;
    }

    let session = {
        let sessions = SESSIONS.read().unwrap_or_else(|e| e.into_inner());
        sessions.get(&session_id).cloned()
    };

    let session = match session {
        Some(s) => s,
        None => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Text::Plain(make_error(-32000, "Session not found")));
            return;
        }
    };

    let state = match depot.obtain::<Arc<AppState>>() {
        Ok(s) => s.clone(),
        Err(_) => {
            res.render(Text::Plain(make_error(-32000, "Server state not available")));
            return;
        }
    };

    let body = match req.parse_body::<String>().await {
        Ok(b) => b,
        Err(_) => {
            res.render(Text::Plain(make_error(-32700, "Failed to read request body")));
            return;
        }
    };

    let app_id = session.app_id;
    let response = dispatch(&body, &state, app_id);
    if response.is_empty() {
        // notification - 无响应
        res.status_code(StatusCode::ACCEPTED);
        return;
    }

    // 通过 SSE 通道推送响应
    let _ = session.tx.send(response.clone());

    // HTTP 直接返回（兼容 MCP 协议）
    res.render(Text::Plain(response));
}
