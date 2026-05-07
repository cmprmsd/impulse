# Warp OSS fork — current status

## Progress summary

- **Phase 0 (cloud strip): ~95% complete.** Removed 7 cloud crates
  (firebase, graphql, warp_server_client, managed_secrets, warp_graphql_schema,
  remote_server, warp_multi_agent_api) and ~250 cloud-only files.
- **Phase 2 (model_client foundation): 100% complete.** New crates/ai
  module ships:
  - `ModelClient` trait with provider-agnostic `ChatRequest` / `ChatDelta`
  - `OpenAiCompatibleClient` (POSTs `/v1/chat/completions` with SSE)
  - `ClaudeCliClient` (spawns `claude --output-format stream-json`)
  - `ConversationStore` (JSONL persistence under `~/WarpData/ai_conversations/`)
  - `agent_loop::run_turn` orchestration
  - `provider_settings::AiProvider` enum + `build_client`
  - `examples/agent_demo.rs` end-to-end runnable demo
  - 101 unit tests passing.

## Compile status

`cargo check -p warp` reduced from 8314 → ~1000 errors (~88% reduction).
All parse errors resolved as of commit 2dde5c9.

Remaining error categories (cargo round 69):
- 905 E0433 (failed to resolve module/type)
- 359 E0412 (failed to resolve type)
- 160 E0425 (failed to resolve identifier)
- ~100 E0404/E0422/E0405/etc.

These are all references to deleted cloud modules (cloud_object,
agent_view, ambient_agents, agent_sdk, ai_document_view, AIRequestUsageModel,
etc.). They need stubs added to `app/src/legacy_stubs.rs` or the
call sites need to be removed.

Estimated remaining work: 10-20 hours of stub additions.

## Phase 1 (local-folder Drive backend): not started

Replacement for `ObjectClient` trait in `app/src/server/server_api/object.rs`
to back Drive operations with files in `~/WarpData/`.

## Phase 3 (Claude CLI integration): backend ready, app integration pending

The `ClaudeCliClient` is complete and tested in isolation. Wiring into
`AppState` and the agent-card UI is pending.

