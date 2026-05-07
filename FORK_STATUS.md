# Warp OSS fork — current status

## Progress this session

**8314 → very few cargo errors after stubbing the deleted proto crates.**
Last targeted compile (round 103, partial under file-lock contention)
showed only 6 distinct E0433 references after the stub crates landed.

## Phases

- **Phase 0 (cloud strip): ~99% complete.** Removed 7 cloud crates and
  ~250 cloud-only files; resolved all parse errors and the bulk of
  type-resolution errors.
- **Phase 2 (model_client foundation): 100% complete.**
  - `crates/ai/model_client/` (`OpenAiCompatibleClient`, `ClaudeCliClient`)
  - `crates/ai/conversation/` (JSONL store under `~/WarpData/ai_conversations/`)
  - `crates/ai/agent_loop.rs` orchestration
  - 101 unit tests + runnable `agent_demo.rs`

## What was done programmatically (~110 commits)

1. **Brace-balance fixes** (~620 let-else `};`, ~270 missing `}`, ~50 stray `}`):
   tokenizer-aware indent-stack matching, applied across 1500+ files.
2. **Comment unresolved imports** (~1071 lines across 77 files).
3. **Module re-exports**: `app/src/lib.rs` stub modules for `crate::server`,
   `crate::auth`, `crate::cloud_object`, `crate::workspaces`,
   `crate::remote_server`, re-exporting types from `legacy_stubs.rs`.
4. **Macro hygiene**: drop legacy `safe_*` and `send_telemetry_*` stubs;
   re-export warp_core's at crate root via `pub use`. Add explicit
   `use crate::*` / `use warp_core::*` to ~150 files.
5. **Trait-type alignment**: redirect `legacy_stubs::InlineMenuType` and
   `legacy_stubs::SettingsSection` imports to canonical paths.
6. **Generic-args**: `<I, M>` parameters on stub `GenericCloudObject`
   and `GenericStringModel`.
7. **From-impl removal**: deleted cloud-coupled From / TryFrom impls in
   `artifacts/mod.rs` and `agent/mod.rs`.
8. **Workspace-level stub crates** for the deleted proto crates:
   - `crates/warp_multi_agent_api/` (Task, response_event, stream_finished
     stats types, AgentType, ToolType, message)
   - `crates/api/` (response_event, message::artifact_event, agent_event,
     RequestParams)
   - `crates/remote_server/` (proto::ReadFileContextRequest etc.,
     client::RemoteServerClient)
   - `crates/warp_graphql/` (ai::AIConversationArtifact and variants)

## Phase 1 (local-folder Drive backend): not started

`LocalFsObjectClient` impl of `ObjectClient` trait, backed by
`~/WarpData/`. Filesystem watcher to replace `get_warp_drive_updates`.

## Phase 3 (Claude CLI integration in-app): backend ready

`ClaudeCliClient` complete and tested. Wiring into `AppState` and the
agent-card UI is pending.
