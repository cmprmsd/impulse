use crate::search::mixer::SearchMixer;
use crate::legacy_stubs::{ExternalSecret};

pub type ExternalSecretSearchMixer = SearchMixer<ExternalSecretSearchItemAction>;

#[derive(Clone, Debug)]
pub enum ExternalSecretSearchItemAction {
    AcceptSecret(ExternalSecret),
}
