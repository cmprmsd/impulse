use crate::search::mixer::SearchMixer;
use crate::legacy_stubs::{SyncId};

pub type EmbeddingSearchMixer = SearchMixer<EmbeddingSearchItemAction>;

#[derive(Clone, Debug)]
pub enum EmbeddingSearchItemAction {
    AcceptWorkflow(SyncId),
    AcceptNotebook(SyncId),
}
