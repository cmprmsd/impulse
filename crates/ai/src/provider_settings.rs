//! User-configurable AI provider settings.
//!
//! Separate from [`api_keys`](crate::api_keys) (which holds credentials)
//! because the *choice of provider* is a UX-level setting, not a secret.
//! Together they're consumed by [`build_client`] to construct the
//! [`Arc<dyn ModelClient>`] the rest of the app uses.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

use crate::api_keys::ApiKeys;
use crate::model_client::{
    claude_cli::{ClaudeCliClient, ClaudeCliConfig},
    openai_compatible::{OpenAiCompatibleClient, OpenAiCompatibleConfig},
    ModelClient, ModelError,
};

/// Which backend the agent talks to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AiProvider {
    /// Any OpenAI-compatible HTTP endpoint (OpenRouter, LiteLLM, Ollama,
    /// vLLM, raw OpenAI, …).
    OpenAiCompatible {
        /// Base URL up to but not including `/chat/completions`. e.g.
        /// `https://api.openai.com/v1`,
        /// `https://openrouter.ai/api/v1`,
        /// `http://localhost:11434/v1`.
        base_url: String,
        /// Model identifier the endpoint expects. e.g. `gpt-4o-mini`,
        /// `anthropic/claude-sonnet-4`, `llama3.2:3b`.
        model: String,
        /// Which credential from [`ApiKeys`] to send as Bearer auth.
        /// Use [`ApiKeyRef::None`] for local servers that don't need
        /// auth (Ollama, etc.).
        api_key_ref: ApiKeyRef,
    },
    /// Spawn the user's `claude` binary with the streaming JSON
    /// protocol. Requires an active Claude subscription / login on the
    /// host.
    ClaudeCli {
        /// Optional explicit path. If `None`, looks up `claude` on `$PATH`.
        binary_path: Option<PathBuf>,
        /// Extra arguments to pass to `claude`.
        #[serde(default)]
        extra_args: Vec<String>,
        /// Optional text appended to Claude's system prompt via
        /// `--append-system-prompt`.
        #[serde(default)]
        system_prompt_append: Option<String>,
    },
}

/// Picks which slot in [`ApiKeys`] supplies the bearer token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyRef {
    None,
    OpenAi,
    Anthropic,
    Google,
    OpenRouter,
}

impl ApiKeyRef {
    pub fn resolve<'a>(&self, keys: &'a ApiKeys) -> Option<&'a str> {
        match self {
            ApiKeyRef::None => None,
            ApiKeyRef::OpenAi => keys.openai.as_deref(),
            ApiKeyRef::Anthropic => keys.anthropic.as_deref(),
            ApiKeyRef::Google => keys.google.as_deref(),
            ApiKeyRef::OpenRouter => keys.open_router.as_deref(),
        }
    }
}

impl Default for AiProvider {
    fn default() -> Self {
        // Sensible default: hit OpenRouter with the user's key. Easy to
        // swap to Ollama / OpenAI directly via the settings UI.
        AiProvider::OpenAiCompatible {
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "anthropic/claude-sonnet-4".to_string(),
            api_key_ref: ApiKeyRef::OpenRouter,
        }
    }
}

/// Build a [`ModelClient`] from settings + credentials.
///
/// The returned `Arc<dyn ModelClient>` is what the rest of the app
/// holds. When settings change, callers swap the Arc for a new one
/// rather than mutating internal state.
pub fn build_client(
    provider: &AiProvider,
    keys: &ApiKeys,
) -> Result<Arc<dyn ModelClient>, ModelError> {
    match provider {
        AiProvider::OpenAiCompatible {
            base_url,
            model: _,
            api_key_ref,
        } => {
            if base_url.trim().is_empty() {
                return Err(ModelError::Config(
                    "OpenAI-compatible base_url is empty".into(),
                ));
            }
            let api_key = api_key_ref.resolve(keys).unwrap_or("").to_string();
            let cfg = OpenAiCompatibleConfig::new(base_url.clone(), api_key);
            let client = OpenAiCompatibleClient::new(cfg)?;
            Ok(Arc::new(client))
        }
        AiProvider::ClaudeCli {
            binary_path,
            extra_args,
            system_prompt_append,
        } => {
            let cfg = ClaudeCliConfig {
                binary_path: binary_path.clone(),
                extra_args: extra_args.clone(),
                system_prompt_append: system_prompt_append.clone(),
                cwd: None,
            };
            Ok(Arc::new(ClaudeCliClient::new(cfg)))
        }
    }
}

/// The model identifier to record on the conversation header. For
/// OpenAI-compatible this is the explicit model field; for Claude CLI
/// we record `"claude"` since the binary picks the model itself.
pub fn model_id(provider: &AiProvider) -> String {
    match provider {
        AiProvider::OpenAiCompatible { model, .. } => model.clone(),
        AiProvider::ClaudeCli { .. } => "claude".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_openrouter() {
        let p = AiProvider::default();
        assert!(matches!(
            p,
            AiProvider::OpenAiCompatible {
                api_key_ref: ApiKeyRef::OpenRouter,
                ..
            }
        ));
    }

    #[test]
    fn build_client_openai_requires_base_url() {
        let provider = AiProvider::OpenAiCompatible {
            base_url: String::new(),
            model: "gpt".into(),
            api_key_ref: ApiKeyRef::None,
        };
        let keys = ApiKeys::default();
        match build_client(&provider, &keys) {
            Err(ModelError::Config(_)) => {}
            Ok(_) => panic!("expected Config error, got Ok"),
            Err(other) => panic!("expected Config error, got {other:?}"),
        }
    }

    fn install_crypto_for_tests() {
        // reqwest with `rustls-tls-native-roots-no-provider` requires a
        // CryptoProvider to be installed before the first Client::build.
        // The app installs it during startup; tests have to do it
        // themselves. install_default panics if called twice — try +
        // ignore covers the common case where this test runs alongside
        // others that already installed it.
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    }

    #[test]
    fn build_client_openai_succeeds_with_url() {
        install_crypto_for_tests();
        let provider = AiProvider::OpenAiCompatible {
            base_url: "http://localhost:11434/v1".into(),
            model: "llama3.2".into(),
            api_key_ref: ApiKeyRef::None,
        };
        let keys = ApiKeys::default();
        assert!(build_client(&provider, &keys).is_ok());
    }

    #[test]
    fn build_client_claude_cli() {
        let provider = AiProvider::ClaudeCli {
            binary_path: None,
            extra_args: vec![],
            system_prompt_append: None,
        };
        let keys = ApiKeys::default();
        assert!(build_client(&provider, &keys).is_ok());
    }

    #[test]
    fn api_key_ref_resolves() {
        let keys = ApiKeys {
            openai: Some("openai-key".into()),
            anthropic: None,
            google: None,
            open_router: Some("or-key".into()),
        };
        assert_eq!(ApiKeyRef::OpenAi.resolve(&keys), Some("openai-key"));
        assert_eq!(ApiKeyRef::Anthropic.resolve(&keys), None);
        assert_eq!(ApiKeyRef::OpenRouter.resolve(&keys), Some("or-key"));
        assert_eq!(ApiKeyRef::None.resolve(&keys), None);
    }

    #[test]
    fn provider_settings_round_trip_json() {
        let provider = AiProvider::OpenAiCompatible {
            base_url: "https://example.com/v1".into(),
            model: "x".into(),
            api_key_ref: ApiKeyRef::Anthropic,
        };
        let json = serde_json::to_string(&provider).expect("serialize");
        let back: AiProvider = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(provider, back);

        let provider = AiProvider::ClaudeCli {
            binary_path: Some(PathBuf::from("/usr/local/bin/claude")),
            extra_args: vec!["--allowedTools".into(), "Bash".into()],
            system_prompt_append: Some("be concise".into()),
        };
        let json = serde_json::to_string(&provider).expect("serialize");
        let back: AiProvider = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(provider, back);
    }
}
