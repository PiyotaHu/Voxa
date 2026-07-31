#![forbid(unsafe_code)]
//! Dependency-light public value and error types for Voxa.

mod error;
mod id;
mod time;

pub use error::{ErrorCategory, ErrorContext, Result, VoxaError};
pub use id::{IdentifierError, NodeId, SessionId, StreamId, TraceId};
pub use time::{SequenceId, Timestamp};
