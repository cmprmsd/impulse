# Warp OSS fork — current status

## Progress this session

**8314 → ~50 cargo errors (>99% reduction)** in `cargo check -p warp`,
addressed across 100+ commits to branch `claude/audit-warp-terminal-lNAJT`.

## Phases

- **Phase 0 (cloud strip): ~99% complete.** Removed 7 cloud crates and
  ~250 cloud-only files; resolved all parse errors and the bulk of
  type-resolution errors.
- **Phase 2 (model_client foundation): 100% complete.**
  - `crates/ai/model_client/` (`OpenAiCompatibleClient`, `ClaudeCliClient`)
  - `crates/ai/conversation/` (JSONL store under `~/WarpData/ai_conversations/`)
  - `crates/ai/agent_loop.rs` orchestration
  - 101 unit tests + runnable `agent_demo.rs`

## What was done programmatically (in batches, ~100 commits)

1. **Brace-balance fixes** (~620 let-else `};`, ~270 missing `}`, ~50 stray `}`):
   tokenizer-aware indent-stack matching, applied across 1500+ files.
2. **Comment unresolved imports** (~1071 lines across 77 files).
3. **Module re-exports**: stub modules under `app/src/lib.rs` for
   `crate::server`, `crate::auth`, `crate::cloud_object`, `crate::workspaces`,
   `crate::remote_server` re-exporting types from `legacy_stubs.rs`.
4. **Macro hygiene**: drop legacy `safe_*` and `send_telemetry_*` stubs
   in favour of warp_core's; re-export them at crate root via `pub use`.
   Add explicit `use crate::*` / `use warp_core::*` to ~150 files where
   the macros were used without imports.
5. **Trait-type alignment**: redirect `legacy_stubs::InlineMenuType` and
   `legacy_stubs::SettingsSection` imports to canonical paths.
6. **Generic-args**: add `<I, M>` parameters to stub `GenericCloudObject`
   and `GenericStringModel` to satisfy E0107 errors.
7. **From impls cleanup**: remove `From<api::*>`, `From<warp_graphql::*>`,
   and diff_hunk_api/warp_multi_agent_api conversions in artifacts and
   agent (their input crates were deleted).
8. **Stubs**: `crate::ai::agent_sdk`, `agent_management`,
   `conversation_details_panel`; `crate::terminal::view::ambient_agent`;
   `ai::index::full_source_code_embedding::manager`.

## Remaining

A handful of errors in deep cloud-coupled call sites (orchestration,
conversation_details_panel internals, etc.). These likely warrant
deletion of entire modules rather than further stubbing.

## Phase 1 (local-folder Drive backend): not started

`LocalFsObjectClient` impl of `ObjectClient` trait, backed by
`~/WarpData/`. Filesystem watcher to replace `get_warp_drive_updates`
subscription.

## Phase 3 (Claude CLI integration in-app): backend ready

`ClaudeCliClient` is complete and tested in isolation. Wiring into
`AppState` and the agent-card UI is pending.
