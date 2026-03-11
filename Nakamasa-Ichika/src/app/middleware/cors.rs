use salvo::prelude::*;
use salvo::http::Method;
use std::sync::LazyLock;
use salvo::http::header::{HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_MAX_AGE};

// 预分配 CORS headers 值 - 使用 unwrap_or_default 安全处理
static CORS_ORIGIN: LazyLock<HeaderValue> = LazyLock::new(|| {
    "*".parse().unwrap_or_else(|_| HeaderValue::from_static("*"))
});
static CORS_HEADERS: LazyLock<HeaderValue> = LazyLock::new(|| {
    "*".parse().unwrap_or_else(|_| HeaderValue::from_static("*"))
});
static CORS_METHODS: LazyLock<HeaderValue> = LazyLock::new(|| {
    "POST, GET, OPTIONS, PUT, DELETE".parse().unwrap_or_else(|_| HeaderValue::from_static("POST, GET, OPTIONS"))
});
static CORS_MAX_AGE: LazyLock<HeaderValue> = LazyLock::new(|| {
    "86400".parse().unwrap_or_else(|_| HeaderValue::from_static("86400"))
});

#[handler]
pub async fn cors(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    // 使用预分配的静态值设置CORS头 - 零分配
    let headers = res.headers_mut();
    headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, CORS_ORIGIN.clone());
    headers.insert(ACCESS_CONTROL_ALLOW_HEADERS, CORS_HEADERS.clone());
    headers.insert(ACCESS_CONTROL_ALLOW_METHODS, CORS_METHODS.clone());
    headers.insert(ACCESS_CONTROL_MAX_AGE, CORS_MAX_AGE.clone());
    
    // 处理OPTIONS预检请求
    if req.method() == Method::OPTIONS {
        res.status_code(StatusCode::OK);
        ctrl.skip_rest();
        return;
    }
    
    // 继续处理其他请求
    ctrl.call_next(req, depot, res).await;
}