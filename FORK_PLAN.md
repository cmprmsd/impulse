# Fork plan — local-only Warp with BYO AI

This document captures the audit and roadmap for forking Warp's open-source
client into a fully local terminal: no cloud account, no Warp servers, with
Drive content stored as plain files (Syncthing-friendly) and AI provided by
user-supplied backends (OpenAI-compatible HTTP, or the Claude CLI's JSON
streaming protocol).

It is intentionally written so any contributor or coding agent picking up this
branch (`claude/audit-warp-terminal-lNAJT`) can execute the plan without
re-doing the audit.

## Why

Warp's client was open-sourced under AGPL (UI framework crates under MIT). The
terminal itself is complete: emulation, PTY, shell integration, UI, editor,
LSP, vim mode are all here. What is **not** open-sourced is the server side —
backend, Warp Drive storage, hosted authentication, hosted-model agents, and
the "Oz" agent orchestration layer (`FAQ.md`, "Is Warp fully open source?"
and "What lives in this repo and what doesn't?").

The fork keeps the terminal, deletes everything that talks to the missing
backend, and replaces the two highest-value cloud features with local
alternatives:

1. **Drive content** → plain files under `~/WarpData/`, sync via Syncthing.
2. **AI agent backend** → BYO endpoint (OpenAI-compatible HTTP) or local
   Claude CLI subprocess (`claude --output-format stream-json
   --input-format stream-json`), reusing a Claude subscription.

## What's actually in this repo (terminal-complete)

63 crates under `crates/`. The terminal-essential ones are substantive
implementations, not stubs:

- `crates/warp_terminal/` — ANSI/VT100 model, escape-sequence parser, grid,
  cursor, modes (alt-screen, mouse), shell I/O.
- `crates/warp_core/` — paths, app IDs, channels, OS detection, execution
  modes.
- `crates/warpui_core/` (MIT) — scene rendering, presenter, accessibility,
  keyboard, text layout.
- `crates/warpui/` (MIT) — platform windowing for macOS (Cocoa),
  Linux/Wayland, headless; font/graphics.
- `crates/languages/`, `crates/lsp/`, `crates/vim/`, `crates/editor/`,
  `crates/syntax_tree/`, `crates/markdown_parser/`, `crates/fuzzy_match/` —
  editor, LSP, vim mode, syntax.
- Shell integration scripts: `app/assets/bundled/bootstrap/{bash.sh, zsh.sh,
  fish.sh, pwsh.ps1}` plus subshell variants, with bash-preexec and equivalents.
- PTY backends: `app/assets/windows/x64/conpty.dll` on Windows; system PTY
  APIs on macOS/Linux from `crates/warp_terminal/src/shell/`.

The build path is documented and works: `./script/bootstrap` then `cargo run`
(`README.md`, `WARP.md`).

## What's not here (cloud-side, must be stripped or replaced)

Per `FAQ.md` and confirmed by inspection: the backend service, the Warp Drive
storage backend, hosted authentication, and Oz agent orchestration are **not**
in this repo. The repo contains the *client-side* GraphQL/Firebase/server-client
code that **talks to** those services — those are what we delete.

## Approach overview

Four phases, executed in order. Each phase is itself broken into
sub-commits that compile cleanly so the branch is always buildable.

| Phase | Goal | Estimated effort |
|---|---|---|
| 0. Cloud strip | App boots and runs as a terminal with **no** network calls to Warp / Firebase | Multi-day, staged in 6 sub-commits |
| 1. Local Drive | Notebooks/workflows/env-vars/etc. stored as files in `~/WarpData/`, hot-reloadable | ~1 week |
| 2. OpenAI-compatible AI | Client-side agent loop hitting a configurable base URL | 3–5 days |
| 3. Claude CLI streaming AI | Spawn `claude` with stream-json I/O, bridge into agent UI | 2–3 days |

### Sizing reality (informs why Phase 0 is staged)

- `app/src/` has ~1,968 `.rs` files.
- ~84 of those import a cloud crate directly
  (`firebase`, `warp_graphql`, `warp_managed_secrets`, `warp_server_client`).
- Cloud-coupled subtrees by file count:
  `server/` 55, `drive/` 45, `auth/` 22, `cloud_object/` 12,
  `workspaces/` 10, `billing/` 3, `ai/agent_sdk/` 65,
  `ai/agent_management/` 14, `ai/ambient_agents/` 7,
  `ai/cloud_environments/` 2, `remote_server/` 9 — roughly **245 files**
  in cloud-coupled directories alone.
- `app/src/workspace/view.rs` is **24,468 lines** and has 16+ inbound refs
  to a single "leaf" feature like `enable_auto_reload_modal`.
- A cold workspace `cargo check` on this codebase takes 15–30 minutes; warm
  iterations are several minutes. Each surgical removal cascades many
  compile errors. This is why a single "delete everything cloud" commit is
  not viable.

---

## Phase 0 — strip the cloud (staged sub-commits)

**Constraint:** every sub-commit must compile (`cargo check --workspace`
clean) and pass `./script/presubmit`. We delete cloud code; we do **not**
feature-flag it. Strangle from the leaves inward.

### 0a — leaf UI (billing, credits, plan-migration, capacity, sharing, teams)

Delete these features and every call site:

- `app/src/billing/` (3 files: `mod.rs`, `shared_objects_creation_denied_body.rs`,
  `shared_objects_creation_denied_modal.rs`)
- `app/src/terminal/buy_credits_banner.rs`
- `app/src/terminal/enable_auto_reload_modal.rs`
- `app/src/workspace/view/free_tier_limit_hit_modal.rs`
- `app/src/workspace/view/build_plan_migration_modal.rs`
- `app/src/workspace/view/cloud_agent_capacity_modal/` (whole dir)
- `app/src/drive/sharing/` (whole dir)
- `app/src/workspaces/` (whole dir — teams)
- `app/src/settings_view/billing_and_usage_page.rs`,
  `app/src/settings_view/billing_and_usage/`,
  `app/src/settings_view/teams_page.rs`
- `app/src/pricing/`

Each removal cascades into `app/src/workspace/view.rs` (24k lines),
`app/src/workspace/util.rs`, `app/src/workspace/mod.rs`,
`app/src/terminal/mod.rs`, `app/src/terminal/view.rs`,
`app/src/pane_group/`, `app/src/root_view.rs`, etc. Plan to surgically remove
references in each.

**Verify:** `cargo check --workspace` clean. App still launches. UI no longer
shows credit / plan / sharing / team surfaces.

### 0b — auth / login bypass

- Force `SkipFirebaseAnonymousUser` (`crates/warp_features/src/lib.rs:801`)
  on permanently and delete the feature-flag branch in
  `app/src/root_view.rs:~1705`.
- Delete `app/src/auth/` (22 files), `crates/onboarding/`'s sign-in flow,
  any "request_login_*" / "verify_email_*" call sites.
- Delete `crates/firebase/` and remove from `Cargo.toml` workspace deps.

**Verify:** App launches straight into the workspace; never hits a sign-in
screen. No outbound network calls to Firebase or `warp.dev`.

### 0c — Drive subscription / cloud-objects listener

- Delete `app/src/server/cloud_objects/` (the WebSocket subscription
  pipeline that feeds `get_warp_drive_updates`).
- Delete the `subscriptions/` GraphQL operations referenced (still keep the
  `ObjectClient` trait and a no-op impl until 0d).

**Verify:** App launches; Drive panel shows empty state instead of
streaming-from-cloud objects. No outbound traffic.

### 0d — `ObjectClient` cloud impl → no-op stub

- Replace the GraphQL impl of `ObjectClient`
  (`app/src/server/server_api/object.rs:336+`) with a `NoOpObjectClient`
  that returns `Err(NotImplemented)` from every method.
- Drop `app/src/server/server_api.rs:1135-1227`'s `generate_multi_agent_output`
  and any other `ServerApi` GraphQL methods unrelated to Phase 1.
- The trait stays so Phase 1 can drop in the real local-fs impl.

**Verify:** App compiles; opening a notebook fails gracefully (Phase 1
fixes this).

### 0e — AI cloud surface

- Delete `app/src/ai/agent_sdk/` (65 files), `app/src/ai/agent_management/`
  (14 files), `app/src/ai/ambient_agents/` (7 files),
  `app/src/ai/cloud_environments/` (2 files),
  `app/src/ai/cloud_agent_config/` (1 file).
- Delete `app/src/remote_server/` (9 files),
  `app/src/external_secrets/` (1 file),
  `app/src/launch_configs/` (4 files), `app/src/usage/` (1 file).
- Strip references from `app/src/ai/blocklist/`, `app/src/terminal/`,
  `app/src/workspace/`, `app/src/settings_view/`.

**Verify:** App compiles. AI panel shows an empty/disabled state until
Phase 2 lands.

### 0f — drop cloud crates from the workspace

When no callers remain:

- Delete crate dirs: `crates/firebase/`, `crates/graphql/`,
  `crates/warp_server_client/`, `crates/managed_secrets/`,
  `crates/managed_secrets_wasm/`, `crates/warp_graphql_schema/`.
- Remove from `Cargo.toml` `[workspace]` members and
  `[workspace.dependencies]`.
- Remove the `warp-proto-apis` git dep (used only for the hosted multi-agent
  protobuf API).

**Verify:** `cargo check --workspace` clean. `tcpdump` during a normal
session shows zero outbound connections to `warp.dev`, Firebase, Stripe,
or Google APIs. `./script/presubmit` passes.

---

## Phase 1 — local-folder Drive backend

**Goal:** all Drive object types live as readable files under a single root
the user can sync via Syncthing.

### Object inventory (already enumerated in the OSS code)

| Type | Source | UI consumer |
|---|---|---|
| Notebook | `crates/graphql/src/api/notebook.rs:8-14` | `app/src/notebooks/` |
| Workflow | `crates/graphql/src/api/workflow.rs:5-9` | `app/src/workflows/` |
| Folder | `crates/graphql/src/api/folder.rs` | tree only |
| AIConversation | `crates/graphql/src/api/ai.rs:144-155` | `app/src/ai/agent_conversations_model.rs` |
| GenericStringObject (env-var collection, AI fact, MCP server, prompt/exec profile, cloud env, scheduled agent, preference) | `crates/graphql/src/api/generic_string_object.rs:16-35` | per-feature subdirs |

### Single seam to swap

`ObjectClient` in `app/src/server/server_api/object.rs:170` (~60 async
methods). Today, exactly one impl exists (`ServerApi`, ~line 336) issuing
GraphQL calls. After Phase 0d it's a no-op stub. We add
`LocalFsObjectClient` and wire it through `app/src/lib.rs`'s
`server_api_provider.get_cloud_objects_client()`. The UI layer
(`app/src/drive/panel.rs`, `app/src/drive/index.rs`, the `Cloud{Notebook,
Workflow, EnvVarCollection}` wrappers under
`app/src/{notebooks, workflows, env_vars}/`) does not need to change.

### Folder layout

Default root: `$XDG_DATA_HOME/WarpData` (Linux/macOS) or
`%LOCALAPPDATA%\WarpData` (Windows), configurable via setting.

```
~/WarpData/
├── notebooks/{uid}.md                # body is markdown
├── notebooks/{uid}.meta.toml         # title, ai_document_id, revision_ts, created_at
├── workflows/{uid}.yaml              # current Warp workflow YAML
├── workflows/{uid}.meta.toml
├── env_vars/{uid}.json               # collection
├── env_vars/{uid}.meta.toml
├── ai_facts/{uid}.md
├── ai_facts/{uid}.meta.toml
├── mcp_servers/{uid}.json
├── prompts/{uid}.md                  # AI execution profile
├── ai_conversations/{uid}.jsonl      # one event per line
├── ai_conversations/{uid}.meta.toml
├── folders/folders.toml              # whole tree in one file (small, hand-editable)
├── trash/{uid}.{md,yaml,json}        # soft-delete just moves the file
├── .stignore                         # Syncthing exclusions (e.g. .warp/)
└── .warp/index.sqlite                # rebuilt from files on startup; never synced
```

### Notes

- **No SQLite in the synced root.** SQLite corrupts under Syncthing's
  concurrent writes. Excluded via `.stignore`. We keep SQLite as a derived
  index in `~/WarpData/.warp/`.
- Sharing, edit-locks (`grab_notebook_edit_access`), permissions,
  transfer-owner, guest emails: all become no-ops or are deleted from
  `ObjectClient`. Single-user, no concurrent editors within one client.
- Conflict policy: Syncthing handles file-level conflicts; we surface its
  `.sync-conflict-*` files as alternates the user can manually merge.
  Last-write-wins on `meta.toml` `revision_ts` for the in-app cache.
- Replace `get_warp_drive_updates` subscription with a `notify`-based
  filesystem watcher → emit the same `ObjectUpdateMessage` events the UI
  already consumes.
- Local cache: rebuild `.warp/index.sqlite` from disk on startup; tail
  filesystem events to keep it fresh. Schema can reuse most of
  `crates/persistence/src/schema.rs` (`notebooks`, `workflows`,
  `generic_string_objects`, `object_metadata`) — drop `object_permissions`,
  `cloud_objects_refreshes`.

### Critical files

- `app/src/server/server_api/object.rs` — add `LocalFsObjectClient` impl
- `app/src/lib.rs` — wire provider to local impl
- `app/src/server/cloud_objects/` (or replacement) — filesystem watcher
- `crates/persistence/src/schema.rs`, `crates/persistence/src/model.rs` —
  trim cloud-only columns
- `app/src/drive/sharing/`, `app/src/drive/export.rs` — already deleted in
  Phase 0a; just keep export

### Verify

Create a notebook in the UI; confirm `~/WarpData/notebooks/<uid>.md` appears
with the content. Edit the file in vim; the UI picks it up within ~1s.
`rm` a workflow file; it disappears from the panel. Point Syncthing at
`~/WarpData/` on two machines, edit on one, see the change on the other.
`tcpdump` confirms still no Warp/Firebase traffic.

---

## Phase 2 — OpenAI-compatible AI backend

### Where the model is called today

`app/src/server/server_api.rs:1135-1227`'s `generate_multi_agent_output`
posts a Warp-specific protobuf (`warp_multi_agent_api::Request`) to
`/ai/multi-agent` and reads base64-encoded protobuf events over SSE. Warp's
"Local" execution mode (`crates/ai/src/agent/orchestration_config.rs:18-30`)
is misleading — it still phones home. We replace this entirely.

### What to keep

- `AIAgentActionType` (`crates/ai/src/agent/action/mod.rs`) — the 32+
  portable tool actions (`ReadFiles`, `RequestFileEdits`,
  `RequestCommandOutput`, `Grep`, `FileGlob`, `CallMCPTool`,
  `AskUserQuestion`, …).
- The action-dispatch loop in `app/src/ai/blocklist/action_model.rs`
  (around `handle_action_result`, ~line 1231) — it already runs tools
  client-side using returned results.
- The `ApiKeys` struct (`crates/ai/src/api_keys.rs:19-26`) and secure
  storage.
- The UI-facing `ResponseEvent` stream — we feed events into it, the UI
  doesn't change.

### What to add

1. A new trait `ModelClient` in `crates/ai/src/model_client/mod.rs`:

   ```rust
   trait ModelClient: Send + Sync {
       async fn stream(&self, req: ChatRequest) -> Stream<ChatDelta>;
   }
   ```

   `ChatRequest` carries the conversation, the available tools (translated
   from `AIAgentActionType` to OpenAI function-calling JSON-Schema), and
   sampling params. `ChatDelta` is provider-agnostic and gets adapted into
   `warp_multi_agent_api::ResponseEvent` for the UI.

2. An `OpenAiCompatibleClient` impl that POSTs `/v1/chat/completions`
   (streaming) to a configurable base URL. Honors OpenAI tool-call deltas;
   emits `ChatDelta::ToolCall` / `ChatDelta::Text` / `ChatDelta::Done`.

3. A *client-side* agent loop in `app/src/ai/agent/` that owns the
   request-tool-result cycle:
   - send user message + tool defs → model
   - on tool_call delta, translate to `AIAgentActionType`, dispatch via the
     existing `handle_action_result` machinery
   - feed tool result back into the conversation, re-stream
   - terminate when the model emits a non-tool-call final message

4. New settings (extend `app/src/ai/cloud_agent_settings.rs`'s
   `define_settings_group!`):
   - `ai.provider` = `openai_compatible | claude_cli`
   - `ai.openai.base_url`, `ai.openai.model`, `ai.openai.api_key_ref`
     (handle into secure storage)
   - Strip `aws` from `ApiKeys` since Bedrock-via-Warp is gone.

### Tool-schema translation

Write a `tools::to_openai_schema(actions: &[AIAgentActionType]) -> Vec<ToolDef>`
once. `AIAgentActionType` variants serialize cleanly to JSON-Schema.

### Conversation persistence

Already local in `BlocklistAIHistoryModel` + `app/src/persistence/agent.rs`.
Drop the cloud-merge call (`merge_cloud_conversation_metadata`,
`agent_conversations_model.rs:~827`). Conversation files live under
`~/WarpData/ai_conversations/`.

### Critical files

- `app/src/server/server_api.rs:1135` — gut `generate_multi_agent_output`;
  route through `ModelClient` instead.
- `crates/ai/src/model_client/` — new module
- `app/src/ai/agent/api/impl.rs:54-94` — drop server-side tool selection;
  client owns it
- `app/src/ai/cloud_agent_settings.rs` — new settings group
- `crates/ai/src/api_keys.rs:19-26` — add `openai_compatible: { base_url,
  api_key }`

### Verify

Point base URL at OpenRouter with a working key + a tool-capable model
(e.g. `anthropic/claude-sonnet-4` or `openai/gpt-4o-mini`); ask the agent
to read a file and propose an edit. Watch the conversation transcript in
`~/WarpData/ai_conversations/<uid>.jsonl`.

---

## Phase 3 — Claude CLI streaming backend

### Goal

Second `ModelClient` impl that shells out to the user's `claude` binary and
bridges its stream-json protocol into the same `ResponseEvent` stream the UI
already consumes.

### What we use

`claude -p "<prompt>" --output-format stream-json --input-format stream-json
--verbose`. Claude CLI emits one JSON object per line on stdout:

- `{"type":"system",…}` — session start
- `{"type":"assistant","message":{…}}` — assistant deltas; tool calls appear
  as content blocks with `type: "tool_use"`
- `{"type":"user","message":{…tool_result…}}` — tool results (Claude CLI
  runs its own tools)
- `{"type":"result",…}` — turn end

### Tool ownership decision

Start with **Claude CLI owns its own tools** (Bash, Read, Edit, Glob, Grep, …).
Lowest-friction path: don't intercept tool calls; the user grants Claude its
own permissions via `~/.claude/settings.json`. We surface tool_use /
tool_result events to the Warp UI as read-only "the agent did X" cards,
reusing existing `ResponseEvent` rendering. Later iterations can intercept
and route through Warp's diff-approval UI.

### Implementation

1. `ClaudeCliClient` impl of `ModelClient`:
   - Spawn `claude` once per conversation with stdio piped, keep it alive
     across turns.
   - On `stream(ChatRequest)`: serialize the new user message to a Claude-CLI
     input event, write to stdin.
   - Read stdout line-by-line; for each JSON event, map → `ChatDelta`:
     - `assistant` → text deltas + `ToolCall` for `tool_use` blocks
       (informational only)
     - `user` (tool_result) → `ToolResult` (informational)
     - `result` with `subtype: success` → `Done`
   - Drain stderr into logs; restart on crash.

2. Map Claude's tool catalog to Warp's existing tool-call cards conceptually
   (Bash → "command", Edit → "file edit", etc.) so the UI looks coherent.
   No need to replicate tool execution.

3. Settings:
   - `ai.claude_cli.binary_path` (default: search `$PATH`)
   - `ai.claude_cli.extra_args` (string list)
   - `ai.claude_cli.system_prompt_append` (passed via
     `--append-system-prompt`)

### Critical files

- `crates/ai/src/model_client/claude_cli.rs` — new
- `app/src/ai/cloud_agent_settings.rs` — add `claude_cli` group

### Verify

Install Claude CLI, log in, set provider to `claude_cli`. Ask "list files in
this dir and tell me about the largest one". Confirm streamed output renders,
tool_use / tool_result events appear as cards, conversation persists to
`~/WarpData/ai_conversations/<uid>.jsonl`. No traffic to `warp.dev`.

---

## Critical-files master list

- **Drive seam:** `app/src/server/server_api/object.rs:170`
  (`ObjectClient` trait), `app/src/lib.rs` (provider wiring),
  `app/src/server/cloud_objects/listener.rs` (subscription → watcher swap).
- **Auth/login:** `crates/warp_features/src/lib.rs:801`,
  `app/src/root_view.rs:~1705`, `crates/onboarding/`.
- **Persistence:** `crates/persistence/src/schema.rs`,
  `crates/persistence/src/model.rs`, `app/src/persistence/agent.rs`.
- **AI seam (today):** `app/src/server/server_api.rs:1135-1227`,
  `app/src/ai/agent/api/impl.rs`, `app/src/ai/blocklist/action_model.rs:~1231`.
- **AI seam (new):** `crates/ai/src/model_client/{mod.rs,
  openai_compatible.rs, claude_cli.rs}`.
- **AI tools:** `crates/ai/src/agent/action/mod.rs` (`AIAgentActionType` —
  keep).
- **Settings:** `app/src/ai/cloud_agent_settings.rs`,
  `crates/ai/src/api_keys.rs`.
- **Crates to delete from workspace:** `firebase`, `graphql`,
  `warp_server_client`, `managed_secrets`, `managed_secrets_wasm`,
  `warp_graphql_schema`. Keep `warp_features` (still useful for local
  toggles).

## End-to-end verification

After Phase 3:

1. `./script/bootstrap` then `cargo run` on Linux. App starts, no login UI.
2. `tcpdump -i any host warp.dev or host firebase or host google` — no
   traffic during a normal session.
3. Create a notebook + workflow in the UI, edit them, delete one. Inspect
   `~/WarpData/`. Edit the notebook file externally in vim; the UI updates
   live.
4. Configure a Syncthing folder at `~/WarpData/` between two boxes; confirm
   bidirectional sync.
5. Set provider to `openai_compatible` with OpenRouter; run `read README.md
   and summarize`; confirm tool calls execute, file is read, summary
   appears.
6. Set provider to `claude_cli`; run the same prompt; confirm streaming
   output and tool-use cards.
7. `./script/presubmit` passes (fmt, clippy, tests).

If all seven pass, the fork is functional and Syncthing-ready.

## Status

- Audit complete (committed as part of this plan).
- **Phase 0 — partial.** Bulk cloud-crate detach landed: 7 cloud crates
  removed from the workspace (`firebase`, `graphql`,
  `warp_graphql_schema`, `managed_secrets`, `managed_secrets_wasm`,
  `warp_server_client`, `remote_server`), ~250 cloud-only files
  deleted from `app/src/`, the proto API (`warp_multi_agent_api`) and
  AWS Bedrock paths stripped from `crates/ai/`, and
  `crates/warp_files/` dropped its `Remote` backend. All non-app
  crates compile clean.
- **Phase 2 — foundation landed and runnable.** All in `crates/ai/`,
  all unit-tested:
  - `model_client/` — `ModelClient` trait,
    `OpenAiCompatibleClient` (POST `/v1/chat/completions`, SSE,
    multi-chunk tool-call argument buffering),
    `ClaudeCliClient` (subprocess of `claude --output-format
    stream-json --input-format stream-json --verbose`).
  - `conversation/` — `Conversation`, `ConversationEvent`
    (Header/UserMessage/AssistantMessage/ToolResult/Info/Closed),
    JSONL `ConversationStore` at `<root>/ai_conversations/<uuid>.jsonl`.
  - `agent_loop.rs` — `run_turn(client, store, conversation,
    user_text, tools, system)` ties the trait + store together;
    `record_tool_result()`, `load_or_new()` helpers.
  - `provider_settings.rs` — `AiProvider {OpenAiCompatible | ClaudeCli}`
    with JSON serde, `build_client(&provider, &keys) ->
    Arc<dyn ModelClient>` factory, `ApiKeyRef` resolver.
  - `examples/agent_demo.rs` — runnable end-to-end demo. Reads
    provider config from env vars, runs a turn, writes the JSONL
    conversation log. Builds with `cargo build --example agent_demo
    -p ai` and works against OpenRouter / Ollama / `claude` CLI.
  - 101/101 unit tests pass on the `ai` crate.
- **Phase 0 Track A — substantial progress, not yet complete.**
  Iterated the warp app crate from ~8314 errors down to ~4099 over
  18 rounds of bulk-fix scripts (~50% reduction). Concrete deliverables:
  - 4 bulk-fix scripts repaired ~700 files of orphan multi-line
    `use foo::{...};` blocks left by the perl-strip.
  - `app/src/legacy_stubs.rs` defines ~150 placeholder types covering
    the deleted-cloud surface (SyncId, ServerId,
    CloudObjectTypeAndId, BlocklistAIHistoryModel, TelemetryEvent,
    UserWorkspaces, RemoteServerManager, ContextChipKind, ChipValue,
    LaunchConfig, ChannelState, AgentRunEvent, ApiKeyUid,
    AIConversationId, ServerConversationToken, AgentViewController,
    AgentViewState, ConversationStatus, AgentToolbarItemKind,
    TaskId, ResponseEvent, CloudObjectLocation, GenericCloudObject,
    ServerCloudObject, ShareableLinkError, AmbientAgentViewModel,
    StringModel, DisplaySetting, LoginGatedFeature, JsonSerializer,
    SecretHandle, AgentModeCitation, ServerAIConversationMetadata,
    ReviewComment, JsonModel, GenericStringModel, ServerTime,
    MessageProvider, SizeInfo, PaneTemplateType, NetworkLogView,
    SubmittableTextInput, TipsCompleted, DetectedLinksState,
    AmbientAgentTaskInput, ScheduledAmbientAgent, AgentRun,
    StartAgentExecutionMode, AIAgentExecutionProfileFields,
    AgentEnvironment, ScheduledAgentTaskRunHistory,
    PassiveSuggestionTriggerType, EntrypointType, OutputModelInfo,
    ChannelStateEvent, ServerEnvironment, AmbientAgentDispatchSource,
    DispatchAmbientAgentRequest, ExternalSecret, InitiatedBy,
    ContainingObject, CLIAgentType, PaintContext, MenuEvent,
    EphemeralMessageModel, ContentEditability, TaskStatusUpdate,
    StoredCredentials, SpawnedFutureHandle, VisibleRow, SizeConstraint,
    SharingDialog, SharingAccessLevel, ParsedTemplatableMCPServerResult,
    ImportQueueArgs, CloudStringObject, CloudObjectTelemetryMetadata,
    BlockClient, AuthError, AuthClient, AttachmentInput, RenderState,
    InlineMenuType, GridType, FuzzyMatchResult, FindOptions,
    ArgumentType, AgentInputFooter, AgentConversationEntry, AIApiError,
    WorkspaceDecorationVisibility, UploadIntent, UploadId, UnlinkRunArgs,
    ToolCall, TimedSession, TerminationGracePeriod, StreamFinishedReason,
    SkillReference, ScheduleArgs, ScheduleAndDispatch,
    RunWithStartingSnapshot, RunActionArgs, ResumeConversationArgs,
    RegisterMacroArgs, PrTaskCounts, PathRoot, PassiveSuggestionTrigger,
    InteractionSource, GENERIC_STRING_OBJECT_PREFIX, CodeReviewModel,
    GlobalCodeReviewModel, AgentSdkProvider, AgentManagement,
    AmbientAgentLifecycle, CodebaseIndexingState, AgentMetadata,
    AgentMessage, AgentTaskState, ConversationOptions, AIInputBlock,
    AICommandBlock, AIOutput, AIOutputId, ChannelState,
    PaneViewLocator, ContextChipKind, ChipValue, LaunchConfig,
    AIClient, ApiKeyUid, CellType, GenericStringObjectId,
    AmbientAgentTaskMetadata, UserContextMetadata).
  - `app/src/legacy_macros.rs` provides crate-wide stub macros for
    `safe_warn!`, `safe_debug!`, `safe_info!`, `safe_error!`,
    `report_error!`, `report_if_error!`,
    `send_telemetry_from_ctx!`, `send_telemetry_from_app_ctx!`,
    `send_telemetry_on_executor!`, `send_telemetry_sync_from_app_ctx!`,
    `id!`, `eq!`, `ne!`. Most are no-ops or `log::*` redirects. The
    keymap macros (`id!` etc.) construct `ContextPredicate` via
    absolute `::warpui::keymap::ContextPredicate::*` paths so they
    work without callers importing the type.
  - 600+ files received bulk import additions across ~20 import
    rounds — `warpui::{ViewContext, AppContext, Element, ...}`,
    `warpui::{fonts, keymap, platform, elements,
    ui_components::components, units}`, plus crate-internal lookups
    for `Appearance`, `TerminalView`, `ToastStack`, `PaneGroup`,
    `Dropdown`, `ActionButton`, `Menu`, `MenuItem`, `NakedTheme`,
    `PrimaryTheme`, `SecondaryTheme`, `LLMId`, `AvailableShell`,
    `Range`, `WriterHandles`, `WorkflowViewMode`, `BackingView`,
    `PaneConfiguration`, `PaneEvent`, `RichTextStyles`, etc.
  - 200+ files had dead-module `use` statements + inline references
    stripped (top-level use lines, nested branches inside
    `use crate::{ ... }` blocks, and direct path replacements like
    `crate::cloud_object::Foo` → `crate::legacy_stubs::Foo`).

  Remaining work: ~4099 cargo errors. Each round saves ~80-100; the
  iteration is converging slowly because the remaining errors are
  cascade failures from method calls on stubs (e.g.
  `BlocklistAIHistoryModel::handle(ctx).update(...)` fails because
  `handle()` isn't defined on the stub). To converge from here, the
  stubs need actual method implementations (or no-op equivalents),
  not just type definitions. That's a different kind of work — one
  that probably needs to be done file-by-file by following the
  cargo errors, looking at the original code, and either adding the
  expected methods to the stub or removing the call site.

  At this point ~50% of the original error volume is from these
  method-call cascades. The remaining 50% is more bulk-fixable
  through the same import-add / use-strip patterns we've been
  applying — another 5-10 mechanical rounds should bring those to
  near zero, then the method-cascade work is what's left.
- **Phase 0 Track B / Phase 2 wiring — not started.** This is the
  remaining work. See "What's still needed" below.

  The **app crate (`warp`) does not yet compile**. Two interleaved
  problems remain:

  1. ~409 files in `app/src/` had multi-line `use foo::bar::{ ... };`
     blocks where the perl-pass that deleted cloud-crate `use`
     statements removed only the first line, leaving an orphan
     `};` and dangling identifier list. A follow-up python pass
     auto-closed many of these but introduced its own breakage in
     valid multi-line `use` blocks (mistakenly removing legitimate
     `};` closers when the run-detector ran twice on shifted line
     numbers). The end state still has unclosed-delimiter errors
     surfacing one-at-a-time in `cargo check`.
  2. The `app/src/ai/agent/` subsystem is intrinsically built on
     the deleted hosted-agent proto API and its types are referenced
     by ~140 files throughout `app/src/ai/`. Replacing that subsystem
     with a local agent loop is Phase 2 work; until that lands, even
     the syntactic errors won't translate into a working build.

  Both problems are contained — the syntax breakage is mechanical
  and grep'able; the agent rebuild is the planned Phase 2.
- Phase 1 — not started.
- Phase 2 — not started.
- Phase 3 — not started.

### What's still needed

The Phase 2 *backends* are usable today as standalone library code
(101 unit tests pass). What's missing is the bridge between them and
the existing app's UI.

**Track A — finish the mechanical syntax cleanup.**
Hundreds of multi-line `use foo::{ ... };` blocks across `app/src/`
were partially mutated by the bulk-strip. The remaining breakage
shows in `cargo check -p warp` as:

- `error: this file contains an unclosed delimiter` — the `};` was
  deleted; restore it.
- `error: unexpected closing delimiter: }` — the `use foo::{`
  opening line was deleted; delete the orphan run.

These should be fixed with a real AST-aware tool (e.g. a small
program using `syn`) — the regex detectors attempted in earlier
commits were fragile and caused collateral damage to struct/enum/match
braces. Hand-fixing one at a time as `cargo check` surfaces them is
slow but safe.

The known-broken files include (top of the list):
`app/src/ai/agent/telemetry.rs`,
`app/src/ai/agent_events/mod.rs`,
`app/src/ai/ai_document_view.rs`,
`app/src/ai/blocklist/history_model.rs`,
`app/src/ai/blocklist/suggested_rule_modal.rs`,
`app/src/ai/blocklist/suggestion_chip_view.rs`,
`app/src/ai/mcp/templatable_manager/native.rs`,
`app/src/ai_assistant/mod.rs`,
`app/src/pane_group/mod.rs`,
`app/src/persistence/mod.rs`,
plus many under `app/src/ai/blocklist/`,
`app/src/settings_view/`, `app/src/terminal/`, `app/src/workspace/`.

**Track B — bridge Phase 2 backends into the app.**

- `app/src/ai/agent/mod.rs` (~3,000 lines after the partial strip)
  defines types like `AIAgentExchange`, `AIConversation`,
  `AIConversationId`, `ServerOutputId`, `ServerConversationToken`,
  etc. that are referenced from ~140 other files. Most of these
  were thin wrappers around the deleted hosted-agent proto API.
  The path forward:
    1. Add a new module `app/src/ai/local_agent/` that re-exports
       `ai::model_client::*`, `ai::conversation::*`, and
       `ai::agent_loop::*` for the rest of the app.
    2. Replace `BlocklistAIHistoryModel` with a thin `warpui::Entity`
       wrapper around `ai::conversation::ConversationStore` —
       roughly 100 call sites use it, but most just call
       `add_message`, `current_conversation`, or `set_active_id`,
       which map cleanly to the new store.
    3. For each surviving cloud-agent type referenced in app/src,
       either (a) re-define it as a thin shim around the new types
       or (b) delete the referrer if it's a cloud-only UI surface
       (e.g. cloud-agent capacity modals).
- Wire `provider_settings::AiProvider` into the existing settings
  UI (`app/src/settings_view/ai_page.rs`) — add radio for
  `OpenAiCompatible` vs `ClaudeCli`, text fields for `base_url`,
  `model`, `binary_path`, etc. The settings system already exists
  in `crates/settings/`.
- Hook `build_client(provider, keys)` into a single
  `Arc<dyn ModelClient>` slot stored in `AppState` (see
  `app/src/app_state.rs`), swap on settings changes.
- Replace the call sites that previously POSTed to the deleted
  `/ai/multi-agent` endpoint (in the now-gone
  `app/src/server/server_api.rs:1135-1227`) with calls to
  `agent_loop::run_turn(client, store, conv, user_text, tools,
  system)` — for v1, leave `tools = vec![]` and `system = None`
  so we get a working text-only agent first, then layer in tool
  dispatch in a later commit.

### Recommended order

The honest read after a long session iterating on Track A: the
remaining errors in the app crate are dominated by **missing
imports** (`ViewContext`, `AppContext`, `Element`, etc. plus
deleted-cloud-types like `BlocklistAIHistoryModel`,
`ServerConversationToken`, `AmbientAgentTaskId`, ...) rather than
parse errors. The plan-time estimate of "Track A is mechanical and
fast" was wrong — it's mechanical but slow because each file's
broken import block is subtly different. A clean approach for the
next session:

1. **Pick a target subset.** The app's `app/src/{auth,server,
   cloud_object,workspaces,billing,external_secrets,launch_configs,
   pricing,usage,remote_server}` are *gone*. The agent system
   (`app/src/ai/agent`, `app/src/ai/blocklist`,
   `app/src/ai/agent_sdk`-style files) is still partially in tree
   but mostly orphaned. Decide whether to delete those subtrees
   wholesale and rebuild a minimal local-agent UI on top of the
   Phase 2 foundation, or keep them and stub the missing types.
2. **If keeping/stubbing:** add a single file
   `app/src/ai/agent/legacy_stubs.rs` that defines all the
   referenced types as bare structs/enums with `#[derive(Clone,
   Debug, Default)]`. List drawn from `cargo check -p warp 2>&1 |
   grep -oE 'cannot find type \`[A-Z][a-zA-Z]*\`' | sort | uniq -c |
   sort -rn`. Wire into `agent/mod.rs`. ~1 day.
3. **If deleting:** wholesale-delete `app/src/ai/agent/` and
   `app/src/ai/blocklist/` and all the cloud-coupled UI under
   `app/src/{drive,notebooks,workflows}` that won't have backends
   until Phase 1, then build a minimal new
   `app/src/local_agent/panel.rs` that shows a chat input + scrolling
   transcript using the Phase 2 `ConversationStore` + `run_turn`.
   ~3-5 days but ends with a usable terminal.
4. Either way, the demo binary at `crates/ai/examples/agent_demo.rs`
   is the smoke test for the underlying Phase 2 backend; whatever
   the app integration looks like, it should produce equivalent
   JSONL on disk.

Phase 1 (local-folder Drive replacement) is still pending and is
independent of Phase 2.

The intended branch for the work is `claude/audit-warp-terminal-lNAJT`;
each phase's sub-commits should land here in the order above. Any future
agent should read this file first.
