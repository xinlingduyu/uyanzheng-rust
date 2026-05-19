//! AI 统一接口处理器
//!
//! 支持多种 AI 提供商（OpenAI、Claude、Gemini、本地模型等）
//! 支持流式和非流式响应
//!
//! 架构说明：
//! - 用户必须登录（通过 UserAuth 中间件）
//! - AI 配置从数据库 u_app 表读取（缓存在 AppInfo 中）
//! - 参考 vip.rs 的模式

use futures_util::StreamExt;
use salvo::prelude::*;
use serde::Serialize;

use nakamasa_ai::{
    AiConfig, AiError, AiProvider, CompletionRequest, Message, MessageRole, PresetConfigs,
    ProviderType, StreamChunk,
};

use crate::app::middleware::app_context::{AppInfo, EncryptionInfo};
use crate::app::middleware::user_auth::UserInfo;
use crate::app::utils::response::{render_error, render_success};

/// AI 请求体
#[derive(serde::Deserialize)]
pub struct AiRequest {
    pub messages: Vec<AiMessage>,
    #[serde(default)]
    pub stream: bool,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// AI 消息
#[derive(serde::Deserialize)]
pub struct AiMessage {
    pub role: String,
    pub content: String,
}

/// AI 非流式响应
#[derive(Serialize)]
pub struct AiCompletionResponse {
    pub id: String,
    pub model: String,
    pub content: String,
    pub usage: Option<AiUsage>,
}

#[derive(Serialize)]
pub struct AiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

fn is_safe_business_message(message: &str) -> bool {
    let msg = message.trim();
    if msg.is_empty() || msg.len() > 200 {
        return false;
    }

    let lower = msg.to_ascii_lowercase();
    let sensitive_markers = [
        "api_key",
        "apikey",
        "authorization",
        "bearer ",
        "password",
        "secret",
        "token",
        "stack backtrace",
        "panicked at",
        "mysql://",
        "redis://",
        "http://",
        "https://",
    ];

    !sensitive_markers
        .iter()
        .any(|marker| lower.contains(marker))
}

fn ai_business_error_message(error: &AiError) -> &'static str {
    match error {
        AiError::RateLimitError => "AI请求过于频繁，请稍后再试",
        AiError::AuthError => "AI认证失败，请检查配置",
        AiError::UnsupportedProvider(_) => "不支持的AI提供商",
        AiError::ConfigError(_) => "AI配置错误",
        AiError::ProviderError(msg) if is_safe_business_message(msg) => "AI服务返回错误",
        AiError::StreamError(msg) if is_safe_business_message(msg) => "AI流式响应中断",
        AiError::HttpError(e) if e.is_timeout() => "AI请求超时，请稍后重试",
        AiError::HttpError(e) if e.is_connect() => "AI服务连接失败",
        _ => "AI请求失败",
    }
}

fn ai_stream_business_error_message(error: &AiError) -> &'static str {
    match error {
        AiError::RateLimitError => "AI请求过于频繁，请稍后再试",
        AiError::AuthError => "AI认证失败，请检查配置",
        AiError::UnsupportedProvider(_) => "不支持的AI提供商",
        AiError::ConfigError(_) => "AI配置错误",
        AiError::ProviderError(msg) if is_safe_business_message(msg) => "AI服务返回错误",
        AiError::StreamError(msg) if is_safe_business_message(msg) => "AI流式响应中断",
        AiError::HttpError(e) if e.is_timeout() => "AI请求超时，请稍后重试",
        AiError::HttpError(e) if e.is_connect() => "AI服务连接失败",
        _ => "AI流式请求失败",
    }
}

/// 从 AppInfo 创建 AI 配置
/// 使用 Nakamasa-Ai 的 PresetConfigs 更好地支持本地模型
fn create_ai_config_from_app_info(app_info: &AppInfo) -> Result<AiConfig, String> {
    if app_info.ai_state != "on" {
        return Err("AI功能未开启".to_string());
    }

    let provider_str = app_info.ai_provider.as_deref().unwrap_or("openai");
    let provider_type = match provider_str.to_lowercase().as_str() {
        "openai" => ProviderType::OpenAI,
        "claude" => ProviderType::Claude,
        "gemini" => ProviderType::Gemini,
        "vllm" => ProviderType::Vllm,
        "sglang" => ProviderType::Sglang,
        "ollama" => ProviderType::Ollama,
        "lmstudio" | "lm_studio" => ProviderType::LmStudio,
        "llamacpp" | "llama_cpp" => ProviderType::LlamaCpp,
        "mistral" | "mistral_rust" => ProviderType::MistralRust,
        _ => return Err(format!("不支持的AI提供商: {}", provider_str)),
    };

    let model = app_info.ai_model.as_deref().unwrap_or("default");

    // 使用 PresetConfigs 创建基础配置（已包含本地模型的默认端口和设置）
    let mut builder = PresetConfigs::from_provider_type(provider_type, model);

    // 用数据库中的值覆盖默认值
    if let Some(api_base) = &app_info.ai_api_base {
        builder = builder.api_base(api_base);
    }

    // API key：优先使用数据库中的值，否则保持预设配置的默认值（本地模型为 "EMPTY"）
    if let Some(api_key) = &app_info.ai_api_key {
        builder = builder.api_key(api_key);
    }
    // 如果数据库中是 None，就不设置，保持预设配置中的默认值（对于本地模型是 "EMPTY"）

    // 温度参数：如果数据库中有有效值则使用，否则使用预设配置的默认值（通常为0.7）
    if let Some(temp) = app_info.ai_temperature {
        if temp > 0.0 && temp <= 2.0 {
            builder = builder.temperature(temp);
        }
    }

    // 最大token数：如果数据库中有有效值则使用，否则使用预设配置的默认值
    if let Some(max_tok) = app_info.ai_max_tokens {
        if max_tok > 0 && max_tok <= 32000 {
            builder = builder.max_tokens(max_tok as u32);
        }
    }

    Ok(builder.build())
}
/// 转换消息格式
fn convert_messages(messages: Vec<AiMessage>) -> Vec<Message> {
    messages
        .into_iter()
        .map(|m| {
            let role = match m.role.as_str() {
                "system" => MessageRole::System,
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                _ => MessageRole::User,
            };
            Message {
                role,
                content: m.content,
                name: None,
            }
        })
        .collect()
}

/// AI 对话接口
#[handler]
pub async fn ai_chat(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    // 1. 验证用户是否登录
    let _user_info = match depot.get::<UserInfo>("user_info") {
        Ok(info) => info,
        Err(_) => {
            render_error(res, "未授权", 201, "");
            return;
        }
    };

    // 2. 获取应用信息
    let app_info = match depot.get::<AppInfo>("app_info") {
        Ok(info) => info,
        Err(_) => {
            render_error(res, "应用信息不存在", 201, "");
            return;
        }
    };
    let app_key = &app_info.app_key;
    let enc_info = app_info.mi.as_ref();

    // 3. 解析请求
    let ai_req = match req.parse_json::<AiRequest>().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("AI参数解析失败: {}", e);
            render_error(res, "参数解析失败", 201, app_key);
            return;
        }
    };

    // 4. 从 AppInfo 创建 AI 配置
    let mut config = match create_ai_config_from_app_info(&app_info) {
        Ok(cfg) => cfg,
        Err(e) => {
            render_error(res, e, 201, app_key);
            return;
        }
    };

    // 5. 允许请求参数覆盖配置
    if let Some(temp) = ai_req.temperature {
        config.temperature = Some(temp);
    }

    if let Some(max_tok) = ai_req.max_tokens {
        config.max_tokens = Some(max_tok);
    }

    // 6. 创建 AI 提供商
    let provider: Box<dyn AiProvider> = match nakamasa_ai::create_provider(config) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("创建AI提供商失败: {}", e);
            render_error(res, "创建AI提供商失败", 201, app_key);
            return;
        }
    };

    // 7. 构建请求
    let messages = convert_messages(ai_req.messages);
    let model = provider.model().to_string();
    let completion_req = CompletionRequest::new(messages, &model);

    // 8. 处理响应
    if ai_req.stream {
        handle_stream_response(provider, completion_req, res, app_key, enc_info).await;
    } else {
        handle_normal_response(provider, completion_req, res, app_key, enc_info).await;
    }
}

/// 处理非流式响应
async fn handle_normal_response(
    provider: Box<dyn AiProvider>,
    request: CompletionRequest,
    res: &mut Response,
    app_key: &str,
    enc_info: Option<&EncryptionInfo>,
) {
    match provider.completion(request).await {
        Ok(response) => {
            let content = response
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default();

            let usage = response.usage.map(|u| AiUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            });

            let ai_response = AiCompletionResponse {
                id: response.id,
                model: response.model,
                content,
                usage,
            };

            render_success(res, app_key, Some(ai_response), enc_info);
        }
        Err(e) => {
            tracing::error!("AI请求失败: {}", e);
            render_error(res, ai_business_error_message(&e), 201, app_key);
        }
    }
}

/// 处理流式响应
async fn handle_stream_response(
    provider: Box<dyn AiProvider>,
    request: CompletionRequest,
    res: &mut Response,
    app_key: &str,
    enc_info: Option<&EncryptionInfo>,
) {
    use futures_util::StreamExt;

    res.headers_mut()
        .insert("Content-Type", "text/event-stream".parse().unwrap());
    res.headers_mut()
        .insert("Cache-Control", "no-cache".parse().unwrap());
    res.headers_mut()
        .insert("Connection", "keep-alive".parse().unwrap());

    match provider.completion_stream(request).await {
        Ok(mut stream) => {
            let mut full_text = String::new();

            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        if chunk.is_done {
                            break;
                        }
                        // 使用 as_deref() 将 Option<String> 转换为 Option<&str>
                        if let Some(text) = chunk.text.as_deref() {
                            full_text.push_str(text);
                        }
                    }
                    Err(e) => {
                        tracing::error!("流式响应错误: {}", e);
                        break;
                    }
                }
            }

            render_success(res, app_key, Some(full_text), enc_info);
        }
        Err(e) => {
            tracing::error!("AI流式请求失败: {}", e);
            render_error(res, ai_stream_business_error_message(&e), 201, app_key);
        }
    }
}
