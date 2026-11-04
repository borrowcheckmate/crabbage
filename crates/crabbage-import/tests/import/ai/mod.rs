//! AI client: the request sent to the chat endpoint, the draft read from its answer, error
//! messages, and that the API key never shows up in them. Every server is on loopback.

mod client;
mod errors;
mod harness;
mod parse;

pub(crate) use harness::*;
