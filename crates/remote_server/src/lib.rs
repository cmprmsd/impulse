//! Stub for the deleted `remote_server` proto crate. Provides the minimum
//! types referenced by call sites in app/src.

pub mod proto {
    #[derive(Debug, Clone, Default)]
    pub struct ReadFileContextRequest {
        pub files: Vec<ReadFileContextFile>,
        pub max_file_bytes: Option<u32>,
        pub max_batch_bytes: Option<u32>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct ReadFileContextFile {
        pub path: String,
        pub line_ranges: Vec<LineRange>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct LineRange {
        pub start: u32,
        pub end: u32,
    }

    pub mod file_context_proto {
        #[derive(Debug, Clone)]
        pub enum Content {
            TextContent(String),
            BinaryContent(Vec<u8>),
        }
    }
}

pub mod setup {
    #[derive(Debug, Clone)]
    pub enum PreinstallStatus {
        Ok,
        Unsupported { reason: UnsupportedReason },
    }

    #[derive(Debug, Clone)]
    pub enum UnsupportedReason {
        GlibcTooOld { required: String, found: String },
        UnsupportedOs,
        UnsupportedArch,
    }
}

pub mod client {
    pub struct RemoteServerClient;

    impl RemoteServerClient {
        pub async fn read_file_context(
            &self,
            _req: super::proto::ReadFileContextRequest,
        ) -> anyhow::Result<ReadFileContextResponse> {
            Err(anyhow::anyhow!("RemoteServerClient is a stub; not implemented in OSS fork"))
        }
    }

    pub struct ReadFileContextResponse {
        pub file_contexts: Vec<FileContextProto>,
        pub failed_files: Vec<FailedFile>,
    }

    pub struct FileContextProto {
        pub file_name: String,
        pub content: Option<super::proto::file_context_proto::Content>,
        pub line_range_start: Option<u32>,
        pub line_range_end: Option<u32>,
        pub last_modified_epoch_millis: Option<u64>,
        pub line_count: u32,
    }

    pub struct FailedFile {
        pub path: String,
        pub error: Option<FailedFileError>,
    }

    pub struct FailedFileError {
        pub message: String,
    }
}
