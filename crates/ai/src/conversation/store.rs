//! Disk-backed conversation store.
//!
//! Reads and appends [`ConversationEvent`]s as JSONL under
//! `<root>/ai_conversations/<id>.jsonl`.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::conversation::{Conversation, ConversationEvent, ConversationId};

/// On-disk conversation store rooted at a single directory.
///
/// `root` is typically `<WarpData>/ai_conversations`.
#[derive(Debug, Clone)]
pub struct ConversationStore {
    root: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("conversation {0} has no Header event (corrupt log)")]
    MissingHeader(ConversationId),
    #[error("conversation log is empty: {0}")]
    Empty(PathBuf),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl ConversationStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path_for(&self, id: ConversationId) -> PathBuf {
        self.root.join(format!("{id}.jsonl"))
    }

    fn ensure_root(&self) -> Result<(), StoreError> {
        fs::create_dir_all(&self.root).map_err(|e| StoreError::Io {
            path: self.root.clone(),
            source: e,
        })
    }

    /// Persist a freshly-created [`Conversation`]'s header to disk.
    /// Returns an error if a file for this id already exists.
    pub fn create(&self, conversation: &Conversation) -> Result<(), StoreError> {
        self.ensure_root()?;
        let path = self.path_for(conversation.id);
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(|e| StoreError::Io {
                path: path.clone(),
                source: e,
            })?;
        for ev in &conversation.events {
            write_event(&mut f, ev, &path)?;
        }
        Ok(())
    }

    /// Append one event to an existing conversation log.
    pub fn append(
        &self,
        id: ConversationId,
        event: &ConversationEvent,
    ) -> Result<(), StoreError> {
        self.ensure_root()?;
        let path = self.path_for(id);
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| StoreError::Io {
                path: path.clone(),
                source: e,
            })?;
        write_event(&mut f, event, &path)
    }

    /// Read one conversation back into memory.
    pub fn load(&self, id: ConversationId) -> Result<Conversation, StoreError> {
        let path = self.path_for(id);
        let f = File::open(&path).map_err(|e| StoreError::Io {
            path: path.clone(),
            source: e,
        })?;
        let reader = BufReader::new(f);
        let mut events = Vec::new();
        for (lineno, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| StoreError::Io {
                path: path.clone(),
                source: e,
            })?;
            if line.trim().is_empty() {
                continue;
            }
            let event: ConversationEvent = serde_json::from_str(&line).map_err(|e| {
                StoreError::Io {
                    path: path.clone(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("line {}: {e}", lineno + 1),
                    ),
                }
            })?;
            events.push(event);
        }
        if events.is_empty() {
            return Err(StoreError::Empty(path));
        }
        let header = events.first().ok_or_else(|| StoreError::Empty(path.clone()))?;
        let (id, created_at, title, provider, model) = match header {
            ConversationEvent::Header {
                id,
                created_at,
                title,
                provider,
                model,
            } => (
                *id,
                *created_at,
                title.clone(),
                provider.clone(),
                model.clone(),
            ),
            _ => return Err(StoreError::MissingHeader(id)),
        };
        Ok(Conversation {
            id,
            created_at,
            title,
            provider,
            model,
            events,
        })
    }

    /// List all known conversation ids by scanning the root directory.
    pub fn list_ids(&self) -> Result<Vec<ConversationId>, StoreError> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }
        let mut ids = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(|e| StoreError::Io {
            path: self.root.clone(),
            source: e,
        })? {
            let entry = entry.map_err(|e| StoreError::Io {
                path: self.root.clone(),
                source: e,
            })?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s,
                None => continue,
            };
            if let Ok(uuid) = uuid::Uuid::parse_str(stem) {
                ids.push(ConversationId(uuid));
            }
        }
        Ok(ids)
    }
}

fn write_event<W: Write>(
    w: &mut W,
    ev: &ConversationEvent,
    path: &Path,
) -> Result<(), StoreError> {
    let line = serde_json::to_string(ev)?;
    writeln!(w, "{line}").map_err(|e| StoreError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::{Conversation, ConversationEvent, ProviderTag};
    use chrono::Utc;
    use tempfile::tempdir;

    #[test]
    fn create_append_and_load_round_trips() {
        let dir = tempdir().expect("tmpdir");
        let store = ConversationStore::new(dir.path().to_path_buf());

        let conv = Conversation::new(ProviderTag::OpenAiCompatible, "gpt-4o-mini");
        let id = conv.id;
        store.create(&conv).expect("create");

        store
            .append(
                id,
                &ConversationEvent::UserMessage {
                    ts: Utc::now(),
                    text: "hello".into(),
                },
            )
            .expect("append user");
        store
            .append(
                id,
                &ConversationEvent::AssistantMessage {
                    ts: Utc::now(),
                    blocks: vec![crate::model_client::ChatBlock::Text {
                        text: "hi".into(),
                    }],
                },
            )
            .expect("append assistant");

        let loaded = store.load(id).expect("load");
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.events.len(), 3);
        assert_eq!(loaded.provider, ProviderTag::OpenAiCompatible);
        assert_eq!(loaded.model, "gpt-4o-mini");
    }

    #[test]
    fn list_ids_finds_new_files() {
        let dir = tempdir().expect("tmpdir");
        let store = ConversationStore::new(dir.path().to_path_buf());
        let conv1 = Conversation::new(ProviderTag::ClaudeCli, "claude");
        let conv2 = Conversation::new(ProviderTag::OpenAiCompatible, "gpt-4o-mini");
        store.create(&conv1).expect("create 1");
        store.create(&conv2).expect("create 2");
        let mut ids = store.list_ids().expect("list");
        ids.sort_by_key(|id| id.0);
        let mut want = vec![conv1.id, conv2.id];
        want.sort_by_key(|id| id.0);
        assert_eq!(ids, want);
    }

    #[test]
    fn missing_file_errors() {
        let dir = tempdir().expect("tmpdir");
        let store = ConversationStore::new(dir.path().to_path_buf());
        let id = ConversationId::new();
        let err = store.load(id).expect_err("must error on missing file");
        assert!(matches!(err, StoreError::Io { .. }));
    }
}
