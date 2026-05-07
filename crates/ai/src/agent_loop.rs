//! Minimal agent loop tying [`ModelClient`] + [`ConversationStore`]
//! together.
//!
//! This is the simplest viable agent loop: send the next user message,
//! stream deltas, accumulate text + tool calls, persist the final
//! assistant message. Tool results are *not* dispatched here — that's
//! the caller's job. The Claude-CLI backend runs its own tools so this
//! loop just records them; the OpenAI-compatible backend will need a
//! separate dispatcher (Track B in `FORK_PLAN.md`) to convert
//! `ToolCallEnd` deltas into [`AIAgentActionType`](crate::agent::action::AIAgentActionType)
//! and route them through Warp's tool registry.

use chrono::Utc;
use futures::StreamExt;
use std::sync::Arc;

use crate::conversation::store::{ConversationStore, StoreError};
use crate::conversation::{Conversation, ConversationEvent, ConversationId};
use crate::model_client::{
    ChatBlock, ChatDelta, ChatRequest, ChatRole, ModelClient, ModelError, StopReason, ToolDef,
};

#[derive(Debug, thiserror::Error)]
pub enum AgentLoopError {
    #[error("model error: {0}")]
    Model(#[from] ModelError),
    #[error("conversation store error: {0}")]
    Store(#[from] StoreError),
}

/// Outcome of a single turn through the agent loop.
#[derive(Debug, Clone)]
pub struct TurnOutcome {
    pub text: String,
    pub tool_calls: Vec<TurnToolCall>,
    pub stop_reason: StopReason,
    pub info_messages: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TurnToolCall {
    pub id: String,
    pub name: String,
    /// JSON-encoded arguments. Streams of partial-arg deltas are
    /// concatenated; if the model emitted a single complete object we
    /// pass it through unchanged.
    pub arguments: String,
}

/// Append a user message to the conversation, send the conversation to
/// the model, stream the response, and persist whatever comes back.
///
/// The returned [`TurnOutcome`] gives the caller everything they need
/// to (a) render the assistant message in the UI and (b) decide whether
/// to dispatch any tool calls and start another turn.
pub async fn run_turn(
    client: &dyn ModelClient,
    store: &ConversationStore,
    conversation: &mut Conversation,
    user_text: String,
    tools: Vec<ToolDef>,
    system_prompt: Option<String>,
) -> Result<TurnOutcome, AgentLoopError> {
    // 1. Persist + apply the user message.
    let user_event = ConversationEvent::UserMessage {
        ts: Utc::now(),
        text: user_text,
    };
    store.append(conversation.id, &user_event)?;
    conversation.events.push(user_event);

    // 2. Build the request from the running history.
    let request = ChatRequest {
        messages: conversation.to_chat_history(),
        tools,
        system: system_prompt,
        temperature: None,
        max_tokens: None,
        model: conversation.model.clone(),
    };

    // 3. Stream and accumulate.
    let mut stream = client.stream(request).await?;

    let mut text = String::new();
    let mut info_messages = Vec::new();
    let mut tool_calls: Vec<TurnToolCall> = Vec::new();
    let mut stop_reason = StopReason::EndTurn;

    while let Some(item) = stream.next().await {
        let delta = item?;
        match delta {
            ChatDelta::TextDelta(t) => text.push_str(&t),
            ChatDelta::ToolCallStart { id, name, .. } => {
                tool_calls.push(TurnToolCall {
                    id,
                    name,
                    arguments: String::new(),
                });
            }
            ChatDelta::ToolCallArgsDelta { id, delta, .. } => {
                if let Some(tc) = tool_calls.iter_mut().find(|t| t.id == id) {
                    tc.arguments.push_str(&delta);
                } else {
                    // Some providers (Claude CLI) emit args without a
                    // preceding ToolCallStart for that id. Create one.
                    tool_calls.push(TurnToolCall {
                        id: id.clone(),
                        name: String::new(),
                        arguments: delta,
                    });
                }
            }
            ChatDelta::ToolCallEnd { .. } => {}
            ChatDelta::Info(msg) => {
                let info_event = ConversationEvent::Info {
                    ts: Utc::now(),
                    text: msg.clone(),
                };
                store.append(conversation.id, &info_event)?;
                conversation.events.push(info_event);
                info_messages.push(msg);
            }
            ChatDelta::Done(reason) => {
                stop_reason = reason;
                break;
            }
        }
    }

    // 4. Persist the assistant message.
    let mut blocks = Vec::new();
    if !text.is_empty() {
        blocks.push(ChatBlock::Text { text: text.clone() });
    }
    for tc in &tool_calls {
        // Try to parse arguments as JSON; fall back to a string if invalid.
        let arguments = serde_json::from_str::<serde_json::Value>(&tc.arguments)
            .unwrap_or_else(|_| serde_json::Value::String(tc.arguments.clone()));
        blocks.push(ChatBlock::ToolCall {
            id: tc.id.clone(),
            name: tc.name.clone(),
            arguments,
        });
    }

    if !blocks.is_empty() {
        let assistant_event = ConversationEvent::AssistantMessage {
            ts: Utc::now(),
            blocks,
        };
        store.append(conversation.id, &assistant_event)?;
        conversation.events.push(assistant_event);
    }

    Ok(TurnOutcome {
        text,
        tool_calls,
        stop_reason,
        info_messages,
    })
}

/// Append a tool result for a previously-emitted tool call. Use this
/// after the caller has dispatched the tool on its own.
pub fn record_tool_result(
    store: &ConversationStore,
    conversation: &mut Conversation,
    tool_call_id: String,
    content: String,
    is_error: bool,
) -> Result<(), StoreError> {
    let event = ConversationEvent::ToolResult {
        ts: Utc::now(),
        tool_call_id,
        content,
        is_error,
    };
    store.append(conversation.id, &event)?;
    conversation.events.push(event);
    Ok(())
}

/// Convenience: load an existing conversation, or create a fresh one if
/// the id doesn't exist on disk yet.
pub fn load_or_new(
    store: &ConversationStore,
    id: Option<ConversationId>,
    provider: crate::conversation::ProviderTag,
    model: impl Into<String>,
) -> Result<Conversation, StoreError> {
    if let Some(id) = id {
        match store.load(id) {
            Ok(c) => return Ok(c),
            Err(StoreError::Io { .. }) => { /* fall through to create */ }
            Err(e) => return Err(e),
        }
    }
    let conv = Conversation::new(provider, model);
    store.create(&conv)?;
    Ok(conv)
}

/// Trait-object friendly wrapper around an [`Arc<dyn ModelClient>`]. The
/// app layer typically holds the client as `Arc<dyn ModelClient>` so it
/// can be swapped at runtime when settings change.
pub type SharedModelClient = Arc<dyn ModelClient>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::ProviderTag;
    use async_trait::async_trait;
    use futures::stream::{self, BoxStream};
    use tempfile::tempdir;

    /// A canned [`ModelClient`] that replays a fixed sequence of deltas.
    struct CannedClient(Vec<ChatDelta>);

    #[async_trait]
    impl ModelClient for CannedClient {
        async fn stream(
            &self,
            _request: ChatRequest,
        ) -> Result<BoxStream<'static, Result<ChatDelta, ModelError>>, ModelError> {
            let items: Vec<_> = self.0.iter().cloned().map(Ok).collect();
            Ok(Box::pin(stream::iter(items)))
        }
    }

    #[test]
    fn turn_with_text_only_persists_assistant_message() {
        let dir = tempdir().expect("tmpdir");
        let store = ConversationStore::new(dir.path().to_path_buf());
        let mut conversation = Conversation::new(ProviderTag::OpenAiCompatible, "gpt-test");
        store.create(&conversation).expect("create");

        let client = CannedClient(vec![
            ChatDelta::TextDelta("hello".into()),
            ChatDelta::TextDelta(" world".into()),
            ChatDelta::Done(StopReason::EndTurn),
        ]);

        let outcome = futures::executor::block_on(run_turn(
            &client,
            &store,
            &mut conversation,
            "what's up?".into(),
            vec![],
            None,
        ))
        .expect("run_turn");

        assert_eq!(outcome.text, "hello world");
        assert!(outcome.tool_calls.is_empty());
        assert_eq!(outcome.stop_reason, StopReason::EndTurn);

        // Reload and check persistence: header + user + assistant.
        let loaded = store.load(conversation.id).expect("load");
        assert_eq!(loaded.events.len(), 3);
    }

    #[test]
    fn turn_with_tool_calls_accumulates_arguments() {
        let dir = tempdir().expect("tmpdir");
        let store = ConversationStore::new(dir.path().to_path_buf());
        let mut conversation = Conversation::new(ProviderTag::OpenAiCompatible, "gpt-test");
        store.create(&conversation).expect("create");

        let client = CannedClient(vec![
            ChatDelta::ToolCallStart {
                id: "call_1".into(),
                name: "Bash".into(),
                index: 0,
            },
            ChatDelta::ToolCallArgsDelta {
                id: "call_1".into(),
                index: 0,
                delta: "{\"command\":\"ls".into(),
            },
            ChatDelta::ToolCallArgsDelta {
                id: "call_1".into(),
                index: 0,
                delta: " -la\"}".into(),
            },
            ChatDelta::ToolCallEnd { id: "call_1".into() },
            ChatDelta::Done(StopReason::ToolUse),
        ]);

        let outcome = futures::executor::block_on(run_turn(
            &client,
            &store,
            &mut conversation,
            "list files".into(),
            vec![],
            None,
        ))
        .expect("run_turn");

        assert_eq!(outcome.stop_reason, StopReason::ToolUse);
        assert_eq!(outcome.tool_calls.len(), 1);
        assert_eq!(outcome.tool_calls[0].id, "call_1");
        assert_eq!(outcome.tool_calls[0].name, "Bash");
        assert_eq!(outcome.tool_calls[0].arguments, "{\"command\":\"ls -la\"}");

        // record_tool_result should append a ToolResult event.
        record_tool_result(
            &store,
            &mut conversation,
            "call_1".into(),
            "main.rs Cargo.toml".into(),
            false,
        )
        .expect("record");

        let loaded = store.load(conversation.id).expect("load");
        // Header, UserMessage, AssistantMessage(blocks: [ToolCall]), ToolResult
        assert_eq!(loaded.events.len(), 4);
    }

    #[test]
    fn info_deltas_are_persisted_as_info_events() {
        let dir = tempdir().expect("tmpdir");
        let store = ConversationStore::new(dir.path().to_path_buf());
        let mut conversation = Conversation::new(ProviderTag::ClaudeCli, "claude");
        store.create(&conversation).expect("create");

        let client = CannedClient(vec![
            ChatDelta::Info("session=abc".into()),
            ChatDelta::TextDelta("ok".into()),
            ChatDelta::Done(StopReason::EndTurn),
        ]);

        let outcome = futures::executor::block_on(run_turn(
            &client,
            &store,
            &mut conversation,
            "hi".into(),
            vec![],
            None,
        ))
        .expect("run_turn");

        assert_eq!(outcome.info_messages, vec!["session=abc".to_string()]);

        let loaded = store.load(conversation.id).expect("load");
        // Header, UserMessage, Info, AssistantMessage
        assert_eq!(loaded.events.len(), 4);
    }
}
