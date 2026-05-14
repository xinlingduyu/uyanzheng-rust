//! OpenAI 提供商实现

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use crate::error::Result;
use crate::provider::AiProvider;
use crate::types::*;

/// OpenAI API 请求体
#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// OpenAI API 响应体
#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    id: String,
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    index: u32,
    message: OpenAiMessageResponse,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessageResponse {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI 流式响应数据块
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAiStreamChunk {
    id: String,
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAiStreamChoice {
    index: u32,
    delta: OpenAiDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiDelta {
    content: Option<String>,
}

pub struct OpenAiProvider {
    config: AiConfig,
    client: reqwest::Client,
    api_base: String,
}

impl OpenAiProvider {
    pub fn new(config: AiConfig) -> Result<Self> {
        let api_base = config
            .api_base
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        let mut headers = reqwest::header::HeaderMap::new();
        let auth_value = format!("Bearer {}", config.api_key);
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&auth_value)
                .map_err(|e| crate::error::AiError::ConfigError(e.to_string()))?,
        );

        if let Some(org) = &config.organization {
            headers.insert(
                "OpenAI-Organization",
                reqwest::header::HeaderValue::from_str(org)
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

    fn convert_messages(messages: &[Message]) -> Vec<OpenAiMessage> {
        messages
            .iter()
            .map(|m| OpenAiMessage {
                role: match m.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::Function => "function".to_string(),
                },
                content: m.content.clone(),
                name: m.name.clone(),
            })
            .collect()
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn completion(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let openai_req = OpenAiRequest {
            model: request.model,
            messages: Self::convert_messages(&request.messages),
            temperature: request.temperature,
            top_p: request.top_p,
            max_tokens: request.max_tokens,
            stream: Some(false),
            tools: request.tools.map(|skills| {
                skills
                    .into_iter()
                    .map(|s| serde_json::to_value(s).unwrap())
                    .collect()
            }),
        };

        let url = format!("{}/chat/completions", self.api_base);

        let response = self
            .client
            .post(&url)
            .json(&openai_req)
            .send()
            .await?
            .error_for_status()?
            .json::<OpenAiResponse>()
            .await?;

        let choices = response
            .choices
            .into_iter()
            .map(|c| Choice {
                index: c.index,
                message: Message {
                    role: match c.message.role.as_str() {
                        "system" => MessageRole::System,
                        "user" => MessageRole::User,
                        "assistant" => MessageRole::Assistant,
                        "function" => MessageRole::Function,
                        _ => MessageRole::Assistant,
                    },
                    content: c.message.content,
                    name: None,
                },
                finish_reason: c.finish_reason,
            })
            .collect();

        Ok(CompletionResponse {
            id: response.id,
            model: response.model,
            choices,
            usage: response.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }

    async fn completion_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>> {
        let openai_req = OpenAiRequest {
            model: request.model,
            messages: Self::convert_messages(&request.messages),
            temperature: request.temperature,
            top_p: request.top_p,
            max_tokens: request.max_tokens,
            stream: Some(true),
            tools: request.tools.map(|skills| {
                skills
                    .into_iter()
                    .map(|s| serde_json::to_value(s).unwrap())
                    .collect()
            }),
        };

        let url = format!("{}/chat/completions", self.api_base);

        let response = self
            .client
            .post(&url)
            .json(&openai_req)
            .send()
            .await?
            .error_for_status()?;

        let stream = response.bytes_stream().map(|chunk| {
            let chunk = chunk.map_err(crate::error::AiError::from)?;
            let text = String::from_utf8_lossy(&chunk);
            parse_stream_chunk(&text)
        });

        Ok(Box::pin(stream))
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::OpenAI
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/models", self.api_base);
        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelData>,
        }
        #[derive(Deserialize)]
        struct ModelData {
            id: String,
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<ModelsResponse>()
            .await?;

        Ok(response.data.into_iter().map(|m| m.id).collect())
    }
}

fn parse_stream_chunk(text: &str) -> Result<StreamChunk> {
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("data: ") {
            let data = &line[6..];
            if data == "[DONE]" {
                return Ok(StreamChunk::done());
            }
            match serde_json::from_str::<OpenAiStreamChunk>(data) {
                Ok(chunk) => {
                    if let Some(choice) = chunk.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            return Ok(StreamChunk::text(content));
                        }
                    }
                }
                Err(_) => continue,
            }
        }
    }
    Ok(StreamChunk::text(""))
}
