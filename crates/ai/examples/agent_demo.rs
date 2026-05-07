//! End-to-end demo of the new local-only AI agent stack.
//!
//! Reads provider config from environment variables, spins up the
//! configured backend (OpenAI-compatible HTTP or Claude CLI), runs a
//! single turn, and prints the streaming output to stdout. Conversations
//! are logged to a temp directory so you can inspect the JSONL.
//!
//! Usage:
//! ```bash
//! # OpenAI-compatible (e.g. OpenRouter):
//! WARP_AI_PROVIDER=openai_compatible \
//! WARP_AI_BASE_URL=https://openrouter.ai/api/v1 \
//! WARP_AI_MODEL=anthropic/claude-sonnet-4 \
//! WARP_AI_API_KEY=$OPENROUTER_API_KEY \
//! cargo run --example agent_demo -p ai -- "What's 2+2?"
//!
//! # Local Ollama:
//! WARP_AI_PROVIDER=openai_compatible \
//! WARP_AI_BASE_URL=http://localhost:11434/v1 \
//! WARP_AI_MODEL=llama3.2 \
//! cargo run --example agent_demo -p ai -- "Hello"
//!
//! # Claude CLI (uses your subscription via the `claude` binary):
//! WARP_AI_PROVIDER=claude_cli \
//! cargo run --example agent_demo -p ai -- "What's the time?"
//! ```

use ai::agent_loop::{run_turn, TurnOutcome};
use ai::api_keys::ApiKeys;
use ai::conversation::store::ConversationStore;
use ai::conversation::{Conversation, ProviderTag};
use ai::model_client::StopReason;
use ai::provider_settings::{build_client, AiProvider, ApiKeyRef};
use std::path::PathBuf;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install rustls crypto provider for reqwest.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // Read provider from env.
    let provider_kind = std::env::var("WARP_AI_PROVIDER").unwrap_or_else(|_| {
        eprintln!("WARP_AI_PROVIDER not set; defaulting to openai_compatible");
        "openai_compatible".to_string()
    });

    let provider = match provider_kind.as_str() {
        "openai_compatible" => {
            let base_url = std::env::var("WARP_AI_BASE_URL").map_err(|_| {
                "WARP_AI_BASE_URL is required for openai_compatible (e.g. \
                 https://openrouter.ai/api/v1, http://localhost:11434/v1)"
            })?;
            let model = std::env::var("WARP_AI_MODEL")
                .unwrap_or_else(|_| "anthropic/claude-sonnet-4".to_string());
            AiProvider::OpenAiCompatible {
                base_url,
                model,
                api_key_ref: ApiKeyRef::OpenAi,
            }
        }
        "claude_cli" => AiProvider::ClaudeCli {
            binary_path: std::env::var("WARP_CLAUDE_BIN").ok().map(PathBuf::from),
            extra_args: vec![],
            system_prompt_append: None,
        },
        other => {
            return Err(format!(
                "unknown WARP_AI_PROVIDER={other:?}; expected openai_compatible or claude_cli"
            )
            .into());
        }
    };

    // API key from env.
    let keys = ApiKeys {
        openai: std::env::var("WARP_AI_API_KEY").ok(),
        anthropic: std::env::var("ANTHROPIC_API_KEY").ok(),
        google: std::env::var("GOOGLE_API_KEY").ok(),
        open_router: std::env::var("OPENROUTER_API_KEY").ok(),
    };

    // Where to write conversation logs.
    let store_root = std::env::temp_dir().join("warp-ai-demo");
    let store = ConversationStore::new(store_root.clone());
    eprintln!("Conversation log directory: {}", store_root.display());

    // What to ask.
    let user_text: String = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let user_text = if user_text.is_empty() {
        "Hello! Tell me a joke.".to_string()
    } else {
        user_text
    };
    eprintln!("[user] {user_text}");
    eprintln!();

    // Build everything.
    let provider_tag = match &provider {
        AiProvider::OpenAiCompatible { .. } => ProviderTag::OpenAiCompatible,
        AiProvider::ClaudeCli { .. } => ProviderTag::ClaudeCli,
    };
    let model_id = ai::provider_settings::model_id(&provider);
    let client: Arc<dyn ai::model_client::ModelClient> = build_client(&provider, &keys)?;

    let mut conversation = Conversation::new(provider_tag, model_id);
    store.create(&conversation)?;
    eprintln!("[conversation_id] {}", conversation.id);
    eprintln!();

    // Run a single turn (no tools yet — tools wiring is Track B).
    let rt = tokio::runtime::Runtime::new()?;
    let outcome: TurnOutcome = rt.block_on(run_turn(
        client.as_ref(),
        &store,
        &mut conversation,
        user_text,
        vec![],
        None,
    ))?;

    println!();
    println!("--- assistant ---");
    println!("{}", outcome.text);
    if !outcome.tool_calls.is_empty() {
        println!();
        println!("--- tool calls ({}) ---", outcome.tool_calls.len());
        for tc in &outcome.tool_calls {
            println!("  {}({}) → {}", tc.name, tc.id, tc.arguments);
        }
    }
    if !outcome.info_messages.is_empty() {
        println!();
        println!("--- harness info ---");
        for m in &outcome.info_messages {
            println!("  {m}");
        }
    }
    println!();
    println!("--- stop ---");
    println!(
        "{}",
        match outcome.stop_reason {
            StopReason::EndTurn => "end_turn",
            StopReason::MaxTokens => "max_tokens",
            StopReason::ToolUse => "tool_use",
            StopReason::Cancelled => "cancelled",
            StopReason::Error => "error",
        }
    );
    println!();
    eprintln!(
        "Conversation persisted: {}",
        store.path_for(conversation.id).display()
    );
    Ok(())
}
