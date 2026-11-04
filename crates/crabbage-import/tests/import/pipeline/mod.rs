//! Recipe completion: the reading of a draft without AI, the merge of the AI's draft and the whole
//! `complete` flow. The AI server is on loopback.

mod complete;
mod from_draft;
mod harness;
mod merge;
mod photos;
mod server;

pub(crate) use harness::*;
