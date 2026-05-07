//! Local conversation model + JSONL persistence.
//!
//! Replaces the proprietary cloud-backed conversation history that lived
//! in the (now deleted) `app/src/ai/agent/conversation.rs` and
//! `app/src/ai/blocklist/history_model.rs`. Conversations live as plain
//! text files under
//! `<root>/ai_conversations/<conversation-id>.jsonl`, where `<root>` is
//! the user's WarpData directory (typically `$XDG_DATA_HOME/WarpData` or
//! `~/WarpData`). One JSON-encoded [`ConversationEvent`] per line, append-
//! only.
//!
//! This layout is Syncthing-friendly: line-oriented, monotonic-append,
//! and survives concurrent writers via newline-aligned merges (Syncthing
//! presents conflicts as `.sync-conflict-*` files which the user can
//! inspect manually).

pub mod store;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model_client::{ChatBlock, ChatMessage, ChatRole};

/// A unique conversation identifier. Used as the filename under
/// `ai_conversations/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConversationId(pub Uuid);

impl ConversationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ConversationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ConversationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// One append-only event in the conversation log.
///
/// The first event is always a [`ConversationEvent::Header`]; everything
/// after is `Message`s and provider events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConversationEvent {
    /// First line of the file. Identifies the conversation, when it
    /// started, and which provider/model is in use.
    Header {
        id: ConversationId,
        created_at: DateTime<Utc>,
        title: Option<String>,
        provider: ProviderTag,
        model: String,
    },

    /// A user-side message. Typed verbatim by the user.
    UserMessage {
        ts: DateTime<Utc>,
        text: String,
    },

    /// An assistant message. May contain text and/or tool calls.
    /// Tool results live in their own [`ToolResult`] events.
    AssistantMessage {
        ts: DateTime<Utc>,
        blocks: Vec<ChatBlock>,
    },

    /// The result of a tool call dispatched by the model.
    ToolResult {
        ts: DateTime<Utc>,
        tool_call_id: String,
        content: String,
        is_error: bool,
    },

    /// Provider-side note (system prompt info, harness chatter).
    Info {
        ts: DateTime<Utc>,
        text: String,
    },

    /// Conversation ended (e.g. user closed it, or `result` event from
    /// the harness). Optional — most conversations stay open and resume
    /// later.
    Closed {
        ts: DateTime<Utc>,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderTag {
    OpenAiCompatible,
    ClaudeCli,
}

/// In-memory view of a conversation, materialized from the JSONL log.
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: ConversationId,
    pub created_at: DateTime<Utc>,
    pub title: Option<String>,
    pub provider: ProviderTag,
    pub model: String,
    pub events: Vec<ConversationEvent>,
}

impl Conversation {
    pub fn new(provider: ProviderTag, model: impl Into<String>) -> Self {
        let id = ConversationId::new();
        let created_at = Utc::now();
        let header = ConversationEvent::Header {
            id,
            created_at,
            title: None,
            provider: provider.clone(),
            model: model.into(),
        };
        let model = match &header {
            ConversationEvent::Header { model, .. } => model.clone(),
            _ => unreachable!(),
        };
        Self {
            id,
            created_at,
            title: None,
            provider,
            model,
            events: vec![header],
        }
    }

    /// Build the chat-message history that the [`ModelClient`](crate::model_client::ModelClient)
    /// expects for the next turn.
    pub fn to_chat_history(&self) -> Vec<ChatMessage> {
        let mut out = Vec::new();
        for ev in &self.events {
            match ev {
                ConversationEvent::Header { .. } | ConversationEvent::Info { .. } => {}
                ConversationEvent::UserMessage { text, .. } => {
                    out.push(ChatMessage::user(text.clone()));
                }
                ConversationEvent::AssistantMessage { blocks, .. } => {
                    out.push(ChatMessage {
                        role: ChatRole::Assistant,
                        blocks: blocks.clone(),
                    });
                }
                ConversationEvent::ToolResult {
                    tool_call_id,
                    content,
                    is_error,
                    ..
                } => {
                    out.push(ChatMessage {
                        role: ChatRole::Tool,
                        blocks: vec![ChatBlock::ToolResult {
                            tool_call_id: tool_call_id.clone(),
                            content: content.clone(),
                            is_error: *is_error,
                        }],
                    });
                }
                ConversationEvent::Closed { .. } => {}
            }
        }
        out
    }
}
