use crate::ai::blocklist::block::CommentElementState;
// (cloud-only deleted) use crate::code_review;
use std::collections::{HashMap, HashSet};

#[derive(Copy, Clone)]
pub(super) struct Props<'a> {
    pub(super) comments: &'a HashMap<CommentId, CommentElementState>,
    pub(super) addressed_comment_ids: &'a HashSet<CommentId>,
}
