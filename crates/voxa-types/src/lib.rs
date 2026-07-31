#![forbid(unsafe_code)]
//! Dependency-light public value and error types for Voxa.

mod id;
mod time;

pub use id::{IdentifierError, NodeId, SessionId, StreamId, TraceId};
pub use time::{SequenceId, Timestamp};
