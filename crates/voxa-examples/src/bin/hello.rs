#![forbid(unsafe_code)]

use voxa_core::logging::{init_default_logging, LogLevel, LogRecord, LogSink, TracingLogSink};
use voxa_examples::hello_message;
use voxa_types::{ErrorCategory, Result, SessionId, VoxaError};

fn main() -> Result<()> {
    let session = SessionId::new("hello-session").map_err(|error| {
        VoxaError::new(
            ErrorCategory::Validation,
            "VOXA-EXM-001",
            "hello session identifier must be valid",
        )
        .with_source(error)
    })?;

    init_default_logging()?;
    init_default_logging()?;

    let record = LogRecord::new(LogLevel::Info, "runtime.ready")?
        .with_session(session.clone())
        .with_field("example", "hello")?;
    TracingLogSink.emit(&record);

    println!("{}", hello_message(&session));
    Ok(())
}
