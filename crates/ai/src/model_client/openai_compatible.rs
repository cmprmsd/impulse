//! OpenAI-compatible HTTP client.
//!
//! POSTs to `<base_url>/chat/completions` with the OpenAI Chat
//! Completions request shape and parses streamed Server-Sent Events back
//! into [`ChatDelta`]s.
//!
//! Tested-shape providers: OpenAI, OpenRouter, LiteLLM, vLLM, Ollama
//! (with `OLLAMA_KEEP_ALIVE` set), text-generation-webui.

use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    ChatBlock, ChatDelta, ChatMessage, ChatRequest, ChatRole, ModelClient, ModelError, StopReason,
};

/// Configuration for an OpenAI-compatible endpoint.
#[derive(Debug, Clone)]
pub struct OpenAiCompatibleConfig {
    /// Base URL ending in `/v1` (or wherever `/chat/completions` lives).
    /// e.g. `https://api.openai.com/v1`, `https://openrouter.ai/api/v1`,
    /// `http://localhost:11434/v1` (Ollama), `http://localhost:8080/v1`
    /// (LiteLLM/vLLM).
    pub base_url: String,
    /// Bearer API key. Empty string is allowed for local servers that
    /// don't require auth.
    pub api_key: String,
    /// Optional extra headers. Useful for OpenRouter's
    /// `HTTP-Referer` / `X-Title` headers.
    pub extra_headers: Vec<(String, String)>,
}

impl OpenAiCompatibleConfig {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            extra_headers: Vec::new(),
        }
    }
}

pub struct OpenAiCompatibleClient {
    config: OpenAiCompatibleConfig,
    http: Client,
}

impl OpenAiCompatibleClient {
    pub fn new(config: OpenAiCompatibleConfig) -> Result<Self, ModelError> {
        let http = Client::builder()
            .build()
            .map_err(|e| ModelError::Config(format!("failed to build reqwest client: {e}")))?;
        Ok(Self { config, http })
    }
}

#[async_trait]
impl ModelClient for OpenAiCompatibleClient {
    async fn stream(
        &self,
        request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatDelta, ModelError>>, ModelError> {
        let body = build_request_body(&request);
        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );

        let mut req = self
            .http
            .post(&url)
            .header("Accept", "text/event-stream")
            .header("Content-Type", "application/json")
            .json(&body);

        if !self.config.api_key.is_empty() {
            req = req.bearer_auth(&self.config.api_key);
        }
        for (k, v) in &self.config.extra_headers {
            req = req.header(k, v);
        }

        let event_source = EventSource::new(req)
            .map_err(|e| ModelError::Http(format!("failed to open event source: {e}")))?;

        Ok(event_source_to_deltas(event_source).boxed())
    }
}

fn build_request_body(request: &ChatRequest) -> serde_json::Value {
    let mut messages: Vec<serde_json::Value> = Vec::new();
    if let Some(system) = &request.system {
        messages.push(json!({"role": "system", "content": system}));
    }
    for msg in &request.messages {
        messages.extend(message_to_openai(msg));
    }

    let mut body = json!({
        "model": request.model,
        "messages": messages,
        "stream": true,
    });
    if let Some(t) = request.temperature {
        body["temperature"] = json!(t);
    }
    if let Some(m) = request.max_tokens {
        body["max_tokens"] = json!(m);
    }
    if !request.tools.is_empty() {
        body["tools"] = json!(request
            .tools
            .iter()
            .map(|t| json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                },
            }))
            .collect::<Vec<_>>());
    }
    body
}

/// Convert one [`ChatMessage`] into one or more OpenAI-shape messages.
///
/// Tool results are split into their own `role: "tool"` messages
/// because OpenAI's schema requires them at the top level (not nested
/// inside an assistant message).
fn message_to_openai(msg: &ChatMessage) -> Vec<serde_json::Value> {
    let role = match msg.role {
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
        ChatRole::System => "system",
        ChatRole::Tool => "tool",
    };

    // Split blocks into:
    //   - text + tool_calls (one assistant/user message)
    //   - tool_results (each becomes its own role: "tool" message)
    let mut text = String::new();
    let mut tool_calls: Vec<serde_json::Value> = Vec::new();
    let mut tool_results: Vec<serde_json::Value> = Vec::new();
    for block in &msg.blocks {
        match block {
            ChatBlock::Text { text: t } => {
                if !text.is_empty() {
                    text.push('\n');
                }
                text.push_str(t);
            }
            ChatBlock::ToolCall {
                id,
                name,
                arguments,
            } => {
                tool_calls.push(json!({
                    "id": id,
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": arguments.to_string(),
                    },
                }));
            }
            ChatBlock::ToolResult {
                tool_call_id,
                content,
                is_error,
            } => {
                let mut content_str = content.clone();
                if *is_error {
                    content_str = format!("[error] {content_str}");
                }
                tool_results.push(json!({
                    "role": "tool",
                    "tool_call_id": tool_call_id,
                    "content": content_str,
                }));
            }
        }
    }

    let mut out = Vec::with_capacity(1 + tool_results.len());
    if !text.is_empty() || !tool_calls.is_empty() {
        let mut m = json!({"role": role});
        if !text.is_empty() {
            m["content"] = json!(text);
        } else {
            // OpenAI requires content key even for assistant tool-only messages.
            m["content"] = json!(null);
        }
        if !tool_calls.is_empty() {
            m["tool_calls"] = json!(tool_calls);
        }
        out.push(m);
    }
    out.extend(tool_results);
    out
}

/// OpenAI-shape SSE delta payload.
#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<ChunkChoice>,
}

#[derive(Deserialize)]
struct ChunkChoice {
    delta: ChunkDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize, Default)]
struct ChunkDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ToolCallDelta>,
}

#[derive(Deserialize, Serialize)]
struct ToolCallDelta {
    #[serde(default)]
    index: u32,
    #[serde(default)]
    id: Option<String>,
    #[serde(default, rename = "type")]
    _kind: Option<String>,
    #[serde(default)]
    function: Option<FunctionDelta>,
}

#[derive(Deserialize, Serialize, Default)]
struct FunctionDelta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

/// Adapt the SSE stream into [`ChatDelta`]s. Buffers tool-call ids across
/// multiple SSE chunks (OpenAI emits the id once and then streams arg
/// fragments without it).
fn event_source_to_deltas(
    mut es: EventSource,
) -> impl futures::Stream<Item = Result<ChatDelta, ModelError>> {
    use std::collections::HashMap;
    async_stream::stream! {
        // index -> tool-call id; lets later chunks resolve their id.
        let mut tool_ids: HashMap<u32, String> = HashMap::new();
        // tool-call ids we've already emitted ToolCallStart for.
        let mut announced: std::collections::HashSet<String> = std::collections::HashSet::new();

        while let Some(ev) = es.next().await {
            let ev = match ev {
                Ok(ev) => ev,
                Err(reqwest_eventsource::Error::StreamEnded) => break,
                Err(e) => {
                    yield Err(ModelError::Http(format!("event source error: {e}")));
                    yield Ok(ChatDelta::Done(StopReason::Error));
                    return;
                }
            };

            let msg = match ev {
                Event::Open => continue,
                Event::Message(m) => m,
            };

            // OpenAI's stream-end sentinel is the literal "[DONE]".
            if msg.data.trim() == "[DONE]" {
                yield Ok(ChatDelta::Done(StopReason::EndTurn));
                return;
            }

            let chunk: StreamChunk = match serde_json::from_str(&msg.data) {
                Ok(c) => c,
                Err(e) => {
                    yield Err(ModelError::Serialization(format!(
                        "failed to parse stream chunk: {e}; payload: {}",
                        truncate(&msg.data, 500)
                    )));
                    continue;
                }
            };

            for choice in chunk.choices {
                if let Some(text) = choice.delta.content {
                    if !text.is_empty() {
                        yield Ok(ChatDelta::TextDelta(text));
                    }
                }

                for tc in choice.delta.tool_calls {
                    let idx = tc.index;
                    if let Some(id) = tc.id.clone() {
                        tool_ids.insert(idx, id);
                    }
                    let id = tool_ids.get(&idx).cloned().unwrap_or_default();

                    let name = tc
                        .function
                        .as_ref()
                        .and_then(|f| f.name.clone())
                        .unwrap_or_default();
                    let args_delta = tc
                        .function
                        .as_ref()
                        .and_then(|f| f.arguments.clone())
                        .unwrap_or_default();

                    if !id.is_empty() && !announced.contains(&id) && !name.is_empty() {
                        yield Ok(ChatDelta::ToolCallStart {
                            id: id.clone(),
                            name,
                            index: idx,
                        });
                        announced.insert(id.clone());
                    }

                    if !args_delta.is_empty() {
                        yield Ok(ChatDelta::ToolCallArgsDelta {
                            id: id.clone(),
                            index: idx,
                            delta: args_delta,
                        });
                    }
                }

                if let Some(reason) = choice.finish_reason {
                    let stop = match reason.as_str() {
                        "stop" => StopReason::EndTurn,
                        "length" => StopReason::MaxTokens,
                        "tool_calls" | "function_call" => StopReason::ToolUse,
                        _ => StopReason::EndTurn,
                    };
                    // Emit ToolCallEnd for each announced tool id so the
                    // dispatcher knows the args JSON is complete.
                    for id in announced.drain() {
                        yield Ok(ChatDelta::ToolCallEnd { id });
                    }
                    yield Ok(ChatDelta::Done(stop));
                    return;
                }
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
