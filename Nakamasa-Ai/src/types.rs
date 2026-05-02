//! AI 相关类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::skills::Skill;

/// AI 提供商类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    OpenAI,
    Claude,
    Gemini,
    // 本地推理模型
    Vllm,
    Sglang,
    Ollama,
    // 其他本地服务
    LmStudio,    // LM Studio - 本地运行，兼容 OpenAI API
    LlamaCpp,    // llama.cpp - 本地运行，兼容 OpenAI API
    MistralRust, // Mistral.rs - Mistral AI 官方 Rust 实现
}

/// AI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider_type: ProviderType,
    pub api_key: String,
    pub api_base: Option<String>,
    pub model: String,
    pub organization: Option<String>,
    pub extra_headers: HashMap<String, String>,
    // 默认生成参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

impl AiConfig {
    pub fn new(provider_type: ProviderType, api_key: &str, model: &str) -> Self {
        Self {
            provider_type,
            api_key: api_key.to_string(),
            api_base: None,
            model: model.to_string(),
            organization: None,
            extra_headers: HashMap::new(),
            temperature: None,
            top_p: None,
            max_tokens: None,
        }
    }

    pub fn with_api_base(mut self, api_base: &str) -> Self {
        self.api_base = Some(api_base.to_string());
        self
    }

    pub fn with_organization(mut self, org: &str) -> Self {
        self.organization = Some(org.to_string());
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Function,
}

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    pub fn system(content: &str) -> Self {
        Self {
            role: MessageRole::System,
            content: content.to_string(),
            name: None,
        }
    }

    pub fn user(content: &str) -> Self {
        Self {
            role: MessageRole::User,
            content: content.to_string(),
            name: None,
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.to_string(),
            name: None,
        }
    }
}

/// 流式响应的数据块
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub text: Option<String>,
    pub is_done: bool,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl StreamChunk {
    pub fn text(text: &str) -> Self {
        Self {
            text: Some(text.to_string()),
            is_done: false,
            metadata: None,
        }
    }

    pub fn done() -> Self {
        Self {
            text: None,
            is_done: true,
            metadata: None,
        }
    }
}

/// 请求参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Skill>>,
}

impl CompletionRequest {
    pub fn new(messages: Vec<Message>, model: &str) -> Self {
        Self {
            messages,
            model: model.to_string(),
            temperature: None,
            top_p: None,
            max_tokens: None,
            stream: None,
            tools: None,
        }
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    pub fn with_tools(mut self, tools: Vec<Skill>) -> Self {
        self.tools = Some(tools);
        self
    }
}

/// 响应结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

/// 响应选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: Option<String>,
}

/// Token 使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
