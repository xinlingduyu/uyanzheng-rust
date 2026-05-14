//! Claude (Anthropic) 提供商实现

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::error::Result;
use crate::provider::AiProvider;
use crate::types::*;

/// Claude API 请求体
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// Claude API 响应体
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    id: String,
    model: String,
    content: Vec<ClaudeContent>,
    usage: Option<ClaudeUsage>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClaudeContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Claude 流式响应数据块
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClaudeStreamChunk {
    #[serde(rename = "type")]
    chunk_type: String,
    delta: Option<ClaudeDelta>,
    message: Option<ClaudeMessageResponse>,
}

#[derive(Debug, Deserialize)]
struct ClaudeDelta {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ClaudeMessageResponse {
    content: Vec<ClaudeContent>,
}

pub struct ClaudeProvider {
    config: AiConfig,
    client: reqwest::Client,
    api_base: String,
}

impl ClaudeProvider {
    pub fn new(config: AiConfig) -> Result<Self> {
        let api_base = config
            .api_base
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com".to_string());

        let mut headers = HeaderMap::new();

        // Claude API key 使用 x-api-key 头部
        let api_key = config.api_key.clone();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&api_key)
                .map_err(|e| crate::error::AiError::ConfigError(e.to_string()))?,
        );

        // Claude 需要 anthropic-version 头部
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // 添加额外的自定义头部
        for (key, value) in &config.extra_headers {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes())
                    .map_err(|e| crate::error::AiError::ConfigError(e.to_string()))?,
                HeaderValue::from_str(value)
                    .map_err(|e| crate::error::AiError::ConfigError(e.to_string()))?,
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| crate::error::AiError::ConfigError(e.to_string()))?;

        Ok(Self {
            config,
            client,
            api_base,
        })
    }

    fn convert_messages(messages: &[Message]) -> Vec<ClaudeMessage> {
        messages
            .iter()
            .filter_map(|m| {
                // Claude 不支持 system 角色在 messages 中，需要特殊处理
                if matches!(m.role, MessageRole::System) {
                    None
                } else {
                    Some(ClaudeMessage {
                        role: match m.role {
                            MessageRole::User => "user".to_string(),
                            MessageRole::Assistant => "assistant".to_string(),
                            MessageRole::Function => "user".to_string(), // 转换为 user
                            MessageRole::System => unreachable!(),
                        },
                        content: m.content.clone(),
                    })
                }
            })
            .collect()
    }

    fn extract_system_message(messages: &[Message]) -> Option<String> {
        messages
            .iter()
            .find(|m| matches!(m.role, MessageRole::System))
            .map(|m| m.content.clone())
    }
}

#[async_trait]
impl AiProvider for ClaudeProvider {
    async fn completion(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let claude_req = ClaudeRequest {
            model: request.model,
            messages: Self::convert_messages(&request.messages),
            max_tokens: request.max_tokens.or(Some(4096)), // Claude 需要 max_tokens
            temperature: request.temperature,
            top_p: request.top_p,
            stream: Some(false),
            tools: request.tools.map(|skills| {
                skills
                    .into_iter()
                    .map(|s| serde_json::to_value(s).unwrap())
                    .collect()
            }),
        };

        // 处理 system 消息
        let system_message = Self::extract_system_message(&request.messages);

        let url = format!("{}/v1/messages", self.api_base);

        let mut req_builder = self.client.post(&url).json(&claude_req);

        if let Some(system) = system_message {
            req_builder = req_builder.header("x-system-prompt", system);
        }

        let response = req_builder
            .send()
            .await?
            .error_for_status()?
            .json::<ClaudeResponse>()
            .await?;

        // 提取文本内容
        let text = response
            .content
            .iter()
            .filter_map(|c| c.text.clone())
            .collect::<Vec<_>>()
            .join("");

        let choices = vec![Choice {
            index: 0,
            message: Message {
                role: MessageRole::Assistant,
                content: text,
                name: None,
            },
            finish_reason: response.stop_reason,
        }];

        Ok(CompletionResponse {
            id: response.id,
            model: response.model,
            choices,
            usage: response.usage.map(|u| Usage {
                prompt_tokens: u.input_tokens,
                completion_tokens: u.output_tokens,
                total_tokens: u.input_tokens + u.output_tokens,
            }),
        })
    }

    async fn completion_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>> {
        let claude_req = ClaudeRequest {
            model: request.model,
            messages: Self::convert_messages(&request.messages),
            max_tokens: request.max_tokens.or(Some(4096)),
            temperature: request.temperature,
            top_p: request.top_p,
            stream: Some(true),
            tools: request.tools.map(|skills| {
                skills
                    .into_iter()
                    .map(|s| serde_json::to_value(s).unwrap())
                    .collect()
            }),
        };

        let system_message = Self::extract_system_message(&request.messages);
        let url = format!("{}/v1/messages", self.api_base);

        let mut req_builder = self.client.post(&url).json(&claude_req);

        if let Some(system) = system_message {
            req_builder = req_builder.header("x-system-prompt", system);
        }

        let response = req_builder.send().await?.error_for_status()?;

        let stream = response.bytes_stream().map(|chunk| {
            let chunk = chunk.map_err(crate::error::AiError::from)?;
            let text = String::from_utf8_lossy(&chunk);
            parse_claude_stream_chunk(&text)
        });

        Ok(Box::pin(stream))
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Claude
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        // Claude 没有公开的列出模型 API，返回常用模型列表
        Ok(vec![
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
            "claude-2.1".to_string(),
            "claude-2.0".to_string(),
            "claude-instant-1.2".to_string(),
        ])
    }
}

fn parse_claude_stream_chunk(text: &str) -> Result<StreamChunk> {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Claude 流式响应是 JSON 对象
        match serde_json::from_str::<ClaudeStreamChunk>(line) {
            Ok(chunk) => {
                if chunk.chunk_type == "content_block_delta" {
                    if let Some(delta) = chunk.delta {
                        if let Some(text) = delta.text {
                            return Ok(StreamChunk::text(&text));
                        }
                    }
                } else if chunk.chunk_type == "message_stop" {
                    return Ok(StreamChunk::done());
                }
            }
            Err(_) => continue,
        }
    }
    Ok(StreamChunk::text(""))
}
