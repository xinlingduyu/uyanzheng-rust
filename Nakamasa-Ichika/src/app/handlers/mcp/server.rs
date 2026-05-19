//! MCP (Model Context Protocol) 服务端实现
//!
//! 嵌入式 MCP Server，通过 SSE 传输协议暴露 AI 可调用的工具。
//!
//! ## 端点
//!
//! - `GET /mcp/sse` — SSE 连接端点
//! - `POST /mcp/messages` — JSON-RPC 消息端点
//!
//! ## 工具
//!
//! - `create_payment` — 创建支付订单
//! - `query_order` — 查询订单状态

use std::collections::HashMap;
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

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
                "name": "create_payment",
                "description": "创建支付订单，返回支付链接或二维码",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "app_id": { "type": "string", "description": "应用ID" },
                        "user_id": { "type": "integer", "description": "用户ID" },
                        "amount": { "type": "number", "description": "支付金额(元)" },
                        "channel": {
                            "type": "string",
                            "description": "支付通道: ali=支付宝, wx=微信支付, jie=借条",
                            "enum": ["ali", "wx", "jie"]
                        },
                        "description": { "type": "string", "description": "订单描述" }
                    },
                    "required": ["app_id", "user_id", "amount", "channel"]
                }
            },
            {
                "name": "query_order",
                "description": "查询订单支付状态",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "order_no": { "type": "string", "description": "商户订单号" },
                        "app_id": { "type": "string", "description": "应用ID" }
                    },
                    "required": ["order_no"]
                }
            }
        ]
    })
}

fn get_pay_manager(_state: &Arc<AppState>) -> Option<Arc<PayPluginManager>> {
    // TODO: PayPluginManager 待加入 AppState
    None
}

// ============================================================================
// Tool 处理器
// ============================================================================

fn handle_create_payment(args: &serde_json::Value, _state: &Arc<AppState>) -> String {
    let amount = args.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    if amount <= 0.0 {
        return make_error(-32602, "Invalid amount: must be greater than 0");
    }

    let channel = args.get("channel").and_then(|v| v.as_str()).unwrap_or("ali");
    let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("MCP支付");
    let app_id = args.get("app_id").and_then(|v| v.as_str()).unwrap_or("");
    let user_id = args.get("user_id").and_then(|v| v.as_i64()).unwrap_or(0);

    if channel != "ali" && channel != "wx" && channel != "jie" {
        return make_error(-32602, &format!("Unsupported channel: {}", channel));
    }

    let _ = (app_id, user_id);

    // 生成订单号
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let order_no = format!("MCP{}{:04}", ts, rand::random::<u16>());

    let pay_order = PayOrder {
        order_no: order_no.clone(),
        name: description.to_string(),
        money: amount,
        notify_url: String::new(),
        return_url: String::new(),
        pay_type: "h5".to_string(),
        client_ip: None,
        scene_info: None,
    };

    match get_pay_manager(_state).and_then(|mgr| {
        let plugin = mgr.get_plugin(channel).ok()?;
        plugin.create(&pay_order).ok()
    }) {
        Some(result) => {
            let resp = serde_json::json!({
                "order_no": order_no,
                "amount": amount,
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
                "amount": amount,
                "channel": channel,
                "status": "pending",
                "note": "Payment plugin not configured, please complete manually"
            });
            make_no_id_result(&resp)
        }
    }
}

fn handle_query_order(args: &serde_json::Value, state: &Arc<AppState>) -> String {
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

fn dispatch(body: &str, state: &Arc<AppState>) -> String {
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
                "create_payment" => handle_create_payment(tool_args, state),
                "query_order" => handle_query_order(tool_args, state),
                _ => make_error(-32601, &format!("Unknown tool: {}", name)),
            };
            // 如果有 id，包装为带 id 的响应
            if let Some(id_val) = id {
                // resp is already a complete jsonrpc response, we need to add the id
                // Simple approach: parse as JSON and re-serialize with id
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

/// SSE 端点：`GET /mcp/sse`
#[handler]
pub async fn sse_handler(_req: &mut Request, res: &mut Response) {
    let (tx, rx) = broadcast::channel::<String>(256);
    let session_id = generate_session_id();

    {
        let mut sessions = SESSIONS.write().unwrap_or_else(|e| e.into_inner());
        sessions.insert(
            session_id.clone(),
            Arc::new(McpSession { tx: tx.clone(), _rx: rx }),
        );
    }

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
        "event: endpoint\ndata: /mcp/messages?session_id={}\n\n",
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
                        let sse = format!("data: {}\n\n", msg);
                        Some((Ok(sse), (rx, true, String::new())))
                    }
                    Err(broadcast::error::RecvError::Closed) => None,
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        Some((Ok(": keepalive\n\n".to_string()), (rx, true, String::new())))
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

    let response = dispatch(&body, &state);
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