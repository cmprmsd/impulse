//! Claude CLI subprocess client.
//!
//! Spawns the user's `claude` binary with
//! `--output-format stream-json --input-format stream-json --verbose`
//! and bridges the line-delimited JSON protocol into [`ChatDelta`]s.
//!
//! Claude CLI is responsible for running its own tools (Bash, Read, Edit,
//! …) using the user's `~/.claude/settings.json` permissions. The client
//! here only forwards prompts and surfaces tool-use / tool-result events
//! to the UI as informational cards — we don't intercept and re-dispatch
//! them through Warp's tool registry. (See `FORK_PLAN.md` Phase 3 for the
//! tool-ownership decision.)
//!
//! ## Wire protocol
//!
//! Each line of stdout is a JSON object. The shapes we care about:
//!
//! ```text
//! {"type": "system", "subtype": "init", "session_id": "...", ...}
//! {"type": "assistant", "message": {"content": [
//!     {"type": "text", "text": "..."},
//!     {"type": "tool_use", "id": "...", "name": "Bash",
//!      "input": {"command": "ls"}}
//! ]}}
//! {"type": "user", "message": {"content": [
//!     {"type": "tool_result", "tool_use_id": "...",
//!      "content": "..."}
//! ]}}
//! {"type": "result", "subtype": "success", "duration_ms": 123, ...}
//! ```
//!
//! Stdin (when sending follow-up turns):
//!
//! ```text
//! {"type": "user", "message": {"role": "user", "content": "..."}}
//! ```

use async_process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use async_trait::async_trait;
use futures::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use futures::stream::BoxStream;
use futures::StreamExt as _;
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::{ChatBlock, ChatDelta, ChatRequest, ChatRole, ModelClient, ModelError, StopReason};

/// Configuration for the Claude CLI backend.
#[derive(Debug, Clone)]
pub struct ClaudeCliConfig {
    /// Path to the `claude` executable. If `None`, looks up `claude` on `$PATH`.
    pub binary_path: Option<PathBuf>,
    /// Extra arguments passed to `claude` (e.g. `["--allowedTools", "Bash"]`).
    pub extra_args: Vec<String>,
    /// Optional text appended to Claude's system prompt via
    /// `--append-system-prompt`.
    pub system_prompt_append: Option<String>,
    /// Optional working directory the subprocess runs in.
    pub cwd: Option<PathBuf>,
}

impl Default for ClaudeCliConfig {
    fn default() -> Self {
        Self {
            binary_path: None,
            extra_args: Vec::new(),
            system_prompt_append: None,
            cwd: None,
        }
    }
}

pub struct ClaudeCliClient {
    config: ClaudeCliConfig,
    /// One subprocess per [`ClaudeCliClient`]. The process is spawned on
    /// the first call to [`stream`] and kept alive across follow-up turns
    /// in the same conversation.
    proc: Arc<Mutex<Option<RunningProc>>>,
}

struct RunningProc {
    #[allow(dead_code)]
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl ClaudeCliClient {
    pub fn new(config: ClaudeCliConfig) -> Self {
        Self {
            config,
            proc: Arc::new(Mutex::new(None)),
        }
    }

    fn spawn(&self) -> Result<RunningProc, ModelError> {
        let bin = self
            .config
            .binary_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("claude"));
        let mut cmd = Command::new(&bin);
        cmd.arg("--output-format")
            .arg("stream-json")
            .arg("--input-format")
            .arg("stream-json")
            .arg("--verbose");
        if let Some(append) = &self.config.system_prompt_append {
            cmd.arg("--append-system-prompt").arg(append);
        }
        for arg in &self.config.extra_args {
            cmd.arg(arg);
        }
        if let Some(cwd) = &self.config.cwd {
            cmd.current_dir(cwd);
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| ModelError::Process(format!("failed to spawn `{}`: {e}", bin.display())))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| ModelError::Process("claude stdin not captured".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ModelError::Process("claude stdout not captured".into()))?;
        Ok(RunningProc {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }
}

#[async_trait]
impl ModelClient for ClaudeCliClient {
    async fn stream(
        &self,
        request: ChatRequest,
    ) -> Result<BoxStream<'static, Result<ChatDelta, ModelError>>, ModelError> {
        // Lock briefly to take the existing process or spawn a new one.
        // We keep the process out of the lock while streaming.
        let mut proc = {
            let mut guard = self.proc.lock().expect("claude_cli mutex poisoned");
            guard.take()
        };
        if proc.is_none() {
            proc = Some(self.spawn()?);
        }
        let mut proc = proc.unwrap();

        // Send the next user message. Claude CLI expects a
        // `{"type":"user", "message":{"role":"user","content":"..."}}` object.
        let user_text = request
            .messages
            .iter()
            .rev()
            .find(|m| m.role == ChatRole::User)
            .map(|m| m.text())
            .unwrap_or_default();

        let payload = json!({
            "type": "user",
            "message": {"role": "user", "content": user_text},
        });
        let line = format!("{}\n", payload);
        if let Err(e) = proc.stdin.write_all(line.as_bytes()).await {
            return Err(ModelError::Process(format!(
                "failed to write user message to claude stdin: {e}"
            )));
        }

        let proc_slot = self.proc.clone();
        Ok(stream_from_proc(proc, proc_slot).boxed())
    }
}

fn stream_from_proc(
    mut proc: RunningProc,
    return_slot: Arc<Mutex<Option<RunningProc>>>,
) -> impl futures::Stream<Item = Result<ChatDelta, ModelError>> {
    async_stream::stream! {
        let mut line_buf = String::new();
        loop {
            line_buf.clear();
            let n = match proc.stdout.read_line(&mut line_buf).await {
                Ok(n) => n,
                Err(e) => {
                    yield Err(ModelError::Process(format!("claude stdout read error: {e}")));
                    yield Ok(ChatDelta::Done(StopReason::Error));
                    return;
                }
            };
            if n == 0 {
                // EOF — process ended without a `result` event.
                yield Ok(ChatDelta::Done(StopReason::Error));
                return;
            }
            let trimmed = line_buf.trim();
            if trimmed.is_empty() {
                continue;
            }

            let event: ClaudeEvent = match serde_json::from_str(trimmed) {
                Ok(ev) => ev,
                Err(e) => {
                    // Surface as an Info delta — Claude CLI sometimes
                    // prints non-JSON chatter (e.g. login prompts).
                    yield Ok(ChatDelta::Info(format!(
                        "[claude-cli] non-JSON line ({e}): {trimmed}"
                    )));
                    continue;
                }
            };

            let mut should_break = false;
            for delta in claude_event_to_deltas(event, &mut should_break) {
                yield Ok(delta);
            }
            if should_break {
                break;
            }
        }

        // Hand the process back so the next turn can reuse it.
        if let Ok(mut guard) = return_slot.lock() {
            *guard = Some(proc);
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClaudeEvent {
    #[serde(rename = "system")]
    System {
        #[serde(default)]
        subtype: Option<String>,
        #[serde(default)]
        session_id: Option<String>,
    },
    #[serde(rename = "assistant")]
    Assistant { message: ClaudeAssistantMessage },
    #[serde(rename = "user")]
    User { message: ClaudeUserMessage },
    #[serde(rename = "result")]
    Result {
        #[serde(default)]
        subtype: Option<String>,
        #[serde(default)]
        is_error: Option<bool>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct ClaudeAssistantMessage {
    #[serde(default)]
    content: Vec<ClaudeContentBlock>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUserMessage {
    #[serde(default)]
    content: Vec<ClaudeContentBlock>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClaudeContentBlock {
    #[serde(rename = "text")]
    Text {
        #[serde(default)]
        text: String,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        #[serde(default)]
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        #[serde(default)]
        content: serde_json::Value,
    },
    #[serde(other)]
    Other,
}

fn claude_event_to_deltas(event: ClaudeEvent, should_break: &mut bool) -> Vec<ChatDelta> {
    match event {
        ClaudeEvent::System {
            subtype,
            session_id,
        } => {
            let label = subtype.unwrap_or_else(|| "init".to_string());
            let info = match session_id {
                Some(id) => format!("[claude-cli] system/{label} session={id}"),
                None => format!("[claude-cli] system/{label}"),
            };
            vec![ChatDelta::Info(info)]
        }
        ClaudeEvent::Assistant { message } => {
            let mut out = Vec::new();
            for (idx, block) in message.content.into_iter().enumerate() {
                match block {
                    ClaudeContentBlock::Text { text } => {
                        if !text.is_empty() {
                            out.push(ChatDelta::TextDelta(text));
                        }
                    }
                    ClaudeContentBlock::ToolUse { id, name, input } => {
                        out.push(ChatDelta::ToolCallStart {
                            id: id.clone(),
                            name,
                            index: idx as u32,
                        });
                        out.push(ChatDelta::ToolCallArgsDelta {
                            id: id.clone(),
                            index: idx as u32,
                            delta: input.to_string(),
                        });
                        out.push(ChatDelta::ToolCallEnd { id });
                    }
                    ClaudeContentBlock::ToolResult { .. } => {
                        // Won't appear inside an assistant message.
                    }
                    ClaudeContentBlock::Other => {}
                }
            }
            out
        }
        ClaudeEvent::User { message } => {
            let mut out = Vec::new();
            for block in message.content {
                if let ClaudeContentBlock::ToolResult {
                    tool_use_id,
                    content,
                } = block
                {
                    let summary = match &content {
                        serde_json::Value::String(s) => s.clone(),
                        v => v.to_string(),
                    };
                    out.push(ChatDelta::Info(format!(
                        "[claude-cli] tool_result tool_use_id={tool_use_id} content={}",
                        truncate(&summary, 200)
                    )));
                }
            }
            out
        }
        ClaudeEvent::Result { subtype, is_error } => {
            *should_break = true;
            let stop = match (subtype.as_deref(), is_error) {
                (Some("success"), _) => StopReason::EndTurn,
                (_, Some(true)) => StopReason::Error,
                _ => StopReason::EndTurn,
            };
            vec![ChatDelta::Done(stop)]
        }
        ClaudeEvent::Other => Vec::new(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max - 1).collect::<String>() + "…"
    }
}

#[allow(dead_code)]
fn _ensure_unused_imports() {
    // Keep ChatBlock import in case downstream tool dispatching wants
    // to round-trip it. Currently the Claude-CLI client surfaces tool
    // events as Info, not as structured ChatBlocks.
    let _: Option<ChatBlock> = None;
}
