//! OpenAI 提供商实现
//!
//! OpenAI 官方接口已优先使用 Responses API：
//! - 非流式：POST /v1/responses
//! - 流式：SSE 事件（response.output_text.delta / response.completed 等）

use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::pin::Pin;

use crate::error::{AiError, Result};
use crate::provider::AiProvider;
use crate::skills::Skill;
use crate::types::*;

/// OpenAI Responses API 请求体
#[derive(Debug, Serialize)]
struct OpenAiResponsesRequest {
    model: String,
    input: Vec<OpenAiResponsesInputMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(rename = "max_output_tokens", skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

/// Responses API 的 EasyInputMessage 格式
#[derive(Debug, Serialize)]
struct OpenAiResponsesInputMessage {
    role: String,
    content: String,
}

/// OpenAI Responses API 响应体
#[derive(Debug, Deserialize)]
struct OpenAiResponsesResponse {
    id: String,
    model: String,
    #[serde(default)]
    output: Vec<OpenAiResponsesOutputItem>,
    #[serde(default)]
    output_text: Option<String>,
    usage: Option<OpenAiResponsesUsage>,
    status: Option<String>,
    error: Option<OpenAiResponsesError>,
    incomplete_details: Option<OpenAiResponsesIncompleteDetails>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponsesOutputItem {
    #[serde(default)]
    content: Vec<OpenAiResponsesContent>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponsesContent {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    refusal: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponsesUsage {
    #[serde(default)]
    input_tokens: u32,
    #[serde(default)]
    output_tokens: u32,
    #[serde(default)]
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponsesError {
    code: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponsesIncompleteDetails {
    reason: Option<String>,
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
                .map_err(|e| AiError::ConfigError(e.to_string()))?,
        );

        if let Some(org) = &config.organization {
            headers.insert(
                "OpenAI-Organization",
                reqwest::header::HeaderValue::from_str(org)
                    .map_err(|e| AiError::ConfigError(e.to_string()))?,
            );
        }

        for (key, value) in &config.extra_headers {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes())
                    .map_err(|e| AiError::ConfigError(e.to_string()))?,
                reqwest::header::HeaderValue::from_str(value)
                    .map_err(|e| AiError::ConfigError(e.to_string()))?,
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| AiError::ConfigError(e.to_string()))?;

        Ok(Self {
            config,
            client,
            api_base,
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.api_base.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn convert_messages(messages: &[Message]) -> Vec<OpenAiResponsesInputMessage> {
        messages
            .iter()
            .map(|m| OpenAiResponsesInputMessage {
                role: match m.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    // Responses API 的普通 input message 不支持 function 角色，
                    // 工具输出应走专用 item；当前统一接口只有文本，降级为 user。
                    MessageRole::Function => "user".to_string(),
                },
                content: m.content.clone(),
            })
            .collect()
    }

    fn convert_tools(tools: Option<Vec<Skill>>) -> Option<Vec<serde_json::Value>> {
        tools.map(|skills| {
            skills
                .into_iter()
                .map(|skill| {
                    serde_json::json!({
                        "type": "function",
                        "name": skill.name,
                        "description": skill.description,
                        "parameters": skill.parameters,
                    })
                })
                .collect()
        })
    }

    fn build_responses_request(
        &self,
        request: CompletionRequest,
        stream: bool,
    ) -> OpenAiResponsesRequest {
        OpenAiResponsesRequest {
            model: request.model,
            input: Self::convert_messages(&request.messages),
            temperature: request.temperature.or(self.config.temperature),
            top_p: request.top_p.or(self.config.top_p),
            max_output_tokens: request.max_tokens.or(self.config.max_tokens),
            stream: Some(stream),
            tools: Self::convert_tools(request.tools),
        }
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn completion(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let openai_req = self.build_responses_request(request, false);
        let url = self.endpoint("responses");

        let response = self
            .client
            .post(&url)
            .json(&openai_req)
            .send()
            .await?
            .error_for_status()?
            .json::<OpenAiResponsesResponse>()
            .await?;

        responses_to_completion(response)
    }

    async fn completion_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>> {
        let openai_req = self.build_responses_request(request, true);
        let url = self.endpoint("responses");

        let response = self
            .client
            .post(&url)
            .json(&openai_req)
            .send()
            .await?
            .error_for_status()?;

        let byte_stream = response.bytes_stream().boxed();
        let stream = futures_util::stream::unfold(
            (
                byte_stream,
                String::new(),
                VecDeque::<StreamChunk>::new(),
                false,
            ),
            |(mut byte_stream, mut buffer, mut pending, mut finished)| async move {
                loop {
                    if let Some(chunk) = pending.pop_front() {
                        if chunk.is_done {
                            finished = true;
                        }
                        return Some((Ok(chunk), (byte_stream, buffer, pending, finished)));
                    }

                    if finished {
                        return None;
                    }

                    match byte_stream.next().await {
                        Some(Ok(bytes)) => {
                            buffer.push_str(&String::from_utf8_lossy(&bytes));
                            while let Some(event) = take_next_sse_event(&mut buffer) {
                                match parse_responses_stream_event(&event) {
                                    Ok(Some(chunk)) => pending.push_back(chunk),
                                    Ok(None) => {}
                                    Err(e) => {
                                        finished = true;
                                        return Some((
                                            Err(e),
                                            (byte_stream, buffer, pending, finished),
                                        ));
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => {
                            finished = true;
                            return Some((
                                Err(AiError::from(e)),
                                (byte_stream, buffer, pending, finished),
                            ));
                        }
                        None => {
                            finished = true;
                            if !buffer.trim().is_empty() {
                                match parse_responses_stream_event(&buffer) {
                                    Ok(Some(chunk)) => {
                                        return Some((
                                            Ok(chunk),
                                            (byte_stream, String::new(), pending, finished),
                                        ));
                                    }
                                    Ok(None) => {}
                                    Err(e) => {
                                        return Some((
                                            Err(e),
                                            (byte_stream, String::new(), pending, finished),
                                        ));
                                    }
                                }
                            }
                            return Some((
                                Ok(StreamChunk::done()),
                                (byte_stream, String::new(), pending, finished),
                            ));
                        }
                    }
                }
            },
        );

        Ok(Box::pin(stream))
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::OpenAI
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let url = self.endpoint("models");
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

fn responses_to_completion(response: OpenAiResponsesResponse) -> Result<CompletionResponse> {
    if response.status.as_deref() == Some("failed") {
        let message = response
            .error
            .as_ref()
            .map(format_responses_error)
            .unwrap_or_else(|| "OpenAI Responses请求失败".to_string());
        return Err(AiError::ProviderError(message));
    }

    if let Some(error) = response.error.as_ref() {
        return Err(AiError::ProviderError(format_responses_error(error)));
    }

    let finish_reason = response
        .incomplete_details
        .as_ref()
        .and_then(|details| details.reason.clone())
        .or_else(|| response.status.clone());

    let usage = response.usage.as_ref().map(|u| Usage {
        prompt_tokens: u.input_tokens,
        completion_tokens: u.output_tokens,
        total_tokens: u.total_tokens,
    });

    Ok(CompletionResponse {
        id: response.id.clone(),
        model: response.model.clone(),
        choices: vec![Choice {
            index: 0,
            message: Message {
                role: MessageRole::Assistant,
                content: extract_response_text(&response),
                name: None,
            },
            finish_reason,
        }],
        usage,
    })
}

fn extract_response_text(response: &OpenAiResponsesResponse) -> String {
    if let Some(output_text) = response.output_text.as_deref()
        && !output_text.is_empty() {
            return output_text.to_string();
        }

    let mut text = String::new();
    for item in &response.output {
        for content in &item.content {
            if let Some(part) = content.text.as_deref() {
                text.push_str(part);
            } else if let Some(refusal) = content.refusal.as_deref() {
                text.push_str(refusal);
            }
        }
    }
    text
}

fn format_responses_error(error: &OpenAiResponsesError) -> String {
    match (error.code.as_deref(), error.message.as_deref()) {
        (Some(code), Some(message)) => format!("{}: {}", code, message),
        (Some(code), None) => code.to_string(),
        (None, Some(message)) => message.to_string(),
        (None, None) => "OpenAI Responses请求失败".to_string(),
    }
}

fn take_next_sse_event(buffer: &mut String) -> Option<String> {
    let lf_pos = buffer.find("\n\n");
    let crlf_pos = buffer.find("\r\n\r\n");

    let (pos, delimiter_len) = match (lf_pos, crlf_pos) {
        (Some(lf), Some(crlf)) if lf < crlf => (lf, 2),
        (Some(_), Some(crlf)) => (crlf, 4),
        (Some(lf), None) => (lf, 2),
        (None, Some(crlf)) => (crlf, 4),
        (None, None) => return None,
    };

    let event = buffer[..pos].to_string();
    buffer.drain(..pos + delimiter_len);
    Some(event)
}

fn parse_responses_stream_event(event: &str) -> Result<Option<StreamChunk>> {
    let event_name = sse_event_name(event);
    let data = sse_data(event);
    let data = data.trim();

    if data.is_empty() {
        return Ok(None);
    }

    if data == "[DONE]" {
        return Ok(Some(StreamChunk::done()));
    }

    let value: serde_json::Value = serde_json::from_str(data)?;
    let event_type = value
        .get("type")
        .and_then(|v| v.as_str())
        .or(event_name.as_deref())
        .unwrap_or_default();

    match event_type {
        "response.output_text.delta" => Ok(value
            .get("delta")
            .and_then(|v| v.as_str())
            .filter(|delta| !delta.is_empty())
            .map(StreamChunk::text)),
        "response.completed" => Ok(Some(StreamChunk::done())),
        "response.incomplete" => Ok(Some(StreamChunk::done())),
        "response.failed" => Err(AiError::StreamError(
            extract_stream_error(&value)
                .unwrap_or_else(|| "OpenAI Responses流式响应失败".to_string()),
        )),
        "error" => Err(AiError::StreamError(
            extract_stream_error(&value)
                .unwrap_or_else(|| "OpenAI Responses流式响应错误".to_string()),
        )),
        _ => Ok(None),
    }
}

fn sse_event_name(event: &str) -> Option<String> {
    event.lines().find_map(|line| {
        let line = line.trim_end_matches('\r');
        line.strip_prefix("event:")
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(|name| name.to_string())
    })
}

fn sse_data(event: &str) -> String {
    event
        .lines()
        .filter_map(|line| {
            let line = line.trim_end_matches('\r');
            line.strip_prefix("data:").map(str::trim_start)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn extract_stream_error(value: &serde_json::Value) -> Option<String> {
    value
        .get("message")
        .and_then(|message| message.as_str())
        .map(|message| message.to_string())
        .or_else(|| extract_nested_error(value.get("error")?))
        .or_else(|| extract_nested_error(value.get("response")?.get("error")?))
}

fn extract_nested_error(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(message) => Some(message.clone()),
        serde_json::Value::Object(map) => map
            .get("message")
            .and_then(|message| message.as_str())
            .map(|message| message.to_string())
            .or_else(|| {
                map.get("code")
                    .and_then(|code| code.as_str())
                    .map(|code| code.to_string())
            }),
        _ => None,
    }
}
