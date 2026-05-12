use salvo::prelude::*;
use salvo::http::Method;
use std::sync::LazyLock;
use salvo::http::header::{HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_MAX_AGE, ACCESS_CONTROL_ALLOW_CREDENTIALS};

// 预分配 CORS headers 值
static CORS_ORIGIN: LazyLock<HeaderValue> = LazyLock::new(|| {
    // 使用具体 origin 替代通配符 *，以支持凭证（credentials）传递
    // 生产环境应替换为实际前端域名
    "http://127.0.0.1:8888".parse()
        .or_else(|_| "https://127.0.0.1:8888".parse())
        .unwrap_or_else(|_| HeaderValue::from_static("http://127.0.0.1:8888"))
});
static CORS_HEADERS: LazyLock<HeaderValue> = LazyLock::new(|| {
    "content-type, authorization, accept-language, token".parse()
        .unwrap_or_else(|_| HeaderValue::from_static("*"))
});
static CORS_METHODS: LazyLock<HeaderValue> = LazyLock::new(|| {
    "GET, POST, PUT, DELETE, OPTIONS".parse()
        .unwrap_or_else(|_| HeaderValue::from_static("GET, POST, OPTIONS"))
});
static CORS_MAX_AGE: LazyLock<HeaderValue> = LazyLock::new(|| {
    "86400".parse().unwrap_or_else(|_| HeaderValue::from_static("86400"))
});

#[handler]
pub async fn cors(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    let headers = res.headers_mut();
    headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, CORS_ORIGIN.clone());
    headers.insert(ACCESS_CONTROL_ALLOW_HEADERS, CORS_HEADERS.clone());
    headers.insert(ACCESS_CONTROL_ALLOW_METHODS, CORS_METHODS.clone());
    headers.insert(ACCESS_CONTROL_MAX_AGE, CORS_MAX_AGE.clone());
    headers.insert(ACCESS_CONTROL_ALLOW_CREDENTIALS, HeaderValue::from_static("true"));
    
    // 处理OPTIONS预检请求
    if req.method() == Method::OPTIONS {
        res.status_code(StatusCode::OK);
        ctrl.skip_rest();
        return;
    }
    
    // 继续处理其他请求
    ctrl.call_next(req, depot, res).await;
}
