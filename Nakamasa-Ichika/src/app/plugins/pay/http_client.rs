//! HTTP 客户端辅助模块
//! 用于支付插件的 HTTP 请求

use std::sync::LazyLock;
use std::time::Duration;

static HTTP_CLIENT: LazyLock<Result<reqwest::Client, String>> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))
});

fn client() -> Result<&'static reqwest::Client, String> {
    HTTP_CLIENT.as_ref().map_err(|e| e.clone())
}

/// 发送 POST 请求（表单格式）
pub async fn post_form(url: &str, data: &str) -> Result<String, String> {
    let response = client()?
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(data.to_string())
        .send()
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;

    let text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    Ok(text)
}

/// 发送 POST 请求（XML格式）
pub async fn post_xml(url: &str, xml: &str) -> Result<String, String> {
    let response = client()?
        .post(url)
        .header("Content-Type", "application/xml")
        .body(xml.to_string())
        .send()
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;

    let text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    Ok(text)
}

/// 发送 GET 请求
pub async fn get(url: &str) -> Result<String, String> {
    let response = client()?
        .get(url)
        .send()
        .await
        .map_err(|e| format!("发送请求失败: {}", e))?;

    let text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    Ok(text)
}
