//! Pluggable AI model backends.
//!
//! The original Warp client called a Warp-hosted protobuf endpoint
//! (`/ai/multi-agent`). After the cloud detach (Phase 0) that endpoint
//! is gone; this module replaces it with two user-supplied backends:
//!
//! 1. [`openai_compatible::OpenAiCompatibleClient`] — POSTs OpenAI-shape
//!    `/v1/chat/completions` requests to a configurable base URL
//!    (OpenRouter, LiteLLM, Ollama, vLLM, …).
//!
//! 2. [`claude_cli::ClaudeCliClient`] — spawns the user's `claude`
//!    binary with `--output-format stream-json --input-format
//!    stream-json --verbose` and bridges its JSON line protocol into
//!    the same [`ChatDelta`] event stream.
//!
//! Both implement the [`ModelClient`] trait so the rest of the app sees
//! one uniform streaming interface.

pub mod claude_cli;
pub mod openai_compatible;

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

/// A pluggable AI model backend.
#[async_trait]
pub trait ModelClient: Send + Sync {
    /// Begin streaming a turn for the given request. The returned stream
    /// yields deltas until a [`ChatDelta::Done`] is emitted, after which
    /// the stream may close.
    async fn stream(
        &self,
        request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatDelta, ModelError>>, ModelError>;
}

/// A complete chat request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Conversation history (oldest first).
    pub messages: Vec<ChatMessage>,
    /// Tools the model is allowed to call.
    pub tools: Vec<ToolDef>,
    /// Optional system prompt.
    pub system: Option<String>,
    /// Sampling temperature, 0..=1.
    pub temperature: Option<f32>,
    /// Hard cap on response tokens.
    pub max_tokens: Option<u32>,
    /// Provider-specific model identifier
    /// (e.g. `gpt-4o-mini`, `anthropic/claude-sonnet-4`, `claude` for the CLI).
    pub model: String,
}

/// One message in a conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    /// Multi-part content (text + tool calls + tool results).
    pub blocks: Vec<ChatBlock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

/// A content block within a [`ChatMessage`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatBlock {
    /// Plain text. Multiple text blocks in a single message are
    /// concatenated by serializers that don't support multi-part text.
    Text { text: String },

    /// The model called a tool. `arguments` is the parsed JSON the
    /// model emitted; tool dispatchers convert it to their concrete
    /// argument type before running the tool.
    ToolCall {
        id: String,
        name: String,
        #[serde(default)]
        arguments: serde_json::Value,
    },

    /// The result of running a previously-emitted tool call.
    ToolResult {
        tool_call_id: String,
        content: String,
        #[serde(default)]
        is_error: bool,
    },
}

impl ChatMessage {
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            blocks: vec![ChatBlock::Text { text: text.into() }],
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            blocks: vec![ChatBlock::Text { text: text.into() }],
        }
    }

    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            blocks: vec![ChatBlock::Text { text: text.into() }],
        }
    }

    /// Concatenated plain-text content of the message, ignoring tool blocks.
    pub fn text(&self) -> String {
        self.blocks
            .iter()
            .filter_map(|b| match b {
                ChatBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

/// A tool definition the model may call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    /// JSON Schema for the tool's argument object.
    pub parameters: serde_json::Value,
}

/// Streaming output from the model.
#[derive(Debug, Clone)]
pub enum ChatDelta {
    /// Incremental assistant text. Concatenate to build the full text.
    TextDelta(String),

    /// A tool call started. Some providers emit this once with full
    /// arguments; others emit it with empty arguments and stream
    /// the JSON via [`ChatDelta::ToolCallArgsDelta`].
    ToolCallStart {
        id: String,
        name: String,
        #[allow(dead_code)]
        index: u32,
    },

    /// Streaming partial JSON for a tool call's arguments.
    ToolCallArgsDelta {
        id: String,
        index: u32,
        delta: String,
    },

    /// A tool call's arguments are complete.
    ToolCallEnd { id: String },

    /// Conversation turn ended. The reason indicates why the model stopped.
    Done(StopReason),

    /// Informational events (system prompts, session metadata, harness chatter)
    /// that should be surfaced to the UI but don't contribute to the conversation.
    Info(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// Model finished naturally.
    EndTurn,
    /// Hit the `max_tokens` cap.
    MaxTokens,
    /// Stopped to wait for tool results.
    ToolUse,
    /// Underlying client cancelled the request.
    Cancelled,
    /// Provider returned an error mid-stream.
    Error,
}

/// Errors a [`ModelClient`] can produce.
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("http error: {0}")]
    Http(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("provider error: {0}")]
    Provider(String),

    #[error("subprocess error: {0}")]
    Process(String),

    #[error("cancelled")]
    Cancelled,

    #[error("invalid configuration: {0}")]
    Config(String),
}

impl From<reqwest::Error> for ModelError {
    fn from(value: reqwest::Error) -> Self {
        ModelError::Http(value.to_string())
    }
}

impl From<serde_json::Error> for ModelError {
    fn from(value: serde_json::Error) -> Self {
        ModelError::Serialization(value.to_string())
    }
}

impl From<std::io::Error> for ModelError {
    fn from(value: std::io::Error) -> Self {
        ModelError::Process(value.to_string())
    }
}
