use crate::legacy_stubs::{AIConversation, ResponseEvent};

// Cloud-only feature stub: shared session replay was used to reconstruct events
// for Warp's cloud-shared sessions. Local-only fork returns empty.
pub fn reconstruct_response_events_from_conversations(
    _conversations: &[AIConversation],
) -> Vec<ResponseEvent> {
    Vec::new()
}
