# Warp OSS fork — current status

## Progress summary

- **Phase 0 (cloud strip): ~99% complete.**
  - Removed 7 cloud crates (firebase, graphql, warp_server_client,
    managed_secrets, warp_graphql_schema, remote_server, warp_multi_agent_api)
    and ~250 cloud-only files.
  - **8314 → ~50 cargo errors** (>99% reduction). All parse errors clear.
- **Phase 2 (model_client foundation): 100% complete.** New `crates/ai`
  module ships:
  - `ModelClient` trait with provider-agnostic `ChatRequest` / `ChatDelta`
  - `OpenAiCompatibleClient` (POSTs `/v1/chat/completions` with SSE)
  - `ClaudeCliClient` (spawns `claude --output-format stream-json`)
  - `ConversationStore` (JSONL persistence under `~/WarpData/ai_conversations/`)
  - `agent_loop::run_turn` orchestration
  - `provider_settings::AiProvider` enum + `build_client`
  - `examples/agent_demo.rs` end-to-end runnable demo
  - 101 unit tests passing.

## What was done programmatically (in batches)

1. **Brace-balance fixes** (~620 let-else `};`, ~270 missing `}`, ~50 stray `}`):
   tokenizer-aware scan + indent-stack matching, applied across 1500+ files.
2. **Comment unresolved imports** (~1071 lines across 77 files).
3. **Module re-exports**: stub modules under `app/src/lib.rs` for
   `crate::server`, `crate::auth`, `crate::cloud_object`, `crate::workspaces`,
   `crate::remote_server` re-exporting types from `legacy_stubs.rs`.
4. **Macro hygiene**: drop legacy `safe_*` and `send_telemetry_*` stubs in
   favour of warp_core's; re-export them at crate root via `pub use`.
   Add explicit `use crate::*` / `use warp_core::*` to ~150 files where
   the macros were used without imports.
5. **Trait-type alignment**: redirect `legacy_stubs::InlineMenuType` and
   `legacy_stubs::SettingsSection` imports to their canonical paths in
   the impl files (E0326/E0053 fixes).
6. **Generic-args**: add `<I, M>` parameters to stub `GenericCloudObject`
   and `GenericStringModel` to satisfy 18 E0107 errors.
7. **From impls cleanup**: remove `From<api::*>`, `From<warp_graphql::*>`,
   and diff_hunk_api/warp_multi_agent_api conversion impls in
   artifacts/mod.rs and agent/mod.rs (their input crates were deleted).

## Compile status

`cargo check -p warp`:
```
8314 → ~50 errors (99.4% reduction)
```

Remaining E[xxxx] codes (latest round):
- E0432/E0433: ~10 unresolved imports in deep cloud-coupled call sites
- E0107/E0223: a couple of generic-arg / ambiguous associated type issues

Most are in files that should be deleted entirely (cloud-only
shared_session, ambient_agent, conversation_details_panel features).

## Phase 1 (local-folder Drive backend): not started

`LocalFsObjectClient` impl of `ObjectClient` trait, backed by
`~/WarpData/`.

## Phase 3 (Claude CLI integration): backend ready, app integration pending

`ClaudeCliClient` is complete and tested in isolation (3 tests).
Wiring into `AppState` and the agent-card UI is pending.
