//! Replaceable structured logging for runtime-facing Voxa services.

use std::sync::OnceLock;

use voxa_types::{ErrorCategory, NodeId, Result, SessionId, VoxaError};

static DEFAULT_LOGGING: OnceLock<()> = OnceLock::new();

/// A severity for a structured Voxa log record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogLevel {
    /// An unrecoverable or user-visible failure.
    Error,
    /// A recoverable condition that needs attention.
    Warn,
    /// A normal runtime lifecycle event.
    Info,
    /// Diagnostic information useful during development.
    Debug,
    /// Highly detailed diagnostic information.
    Trace,
}

/// A structured event that can be emitted through any [`LogSink`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogRecord {
    level: LogLevel,
    event_name: Box<str>,
    session: Option<SessionId>,
    node: Option<NodeId>,
    fields: Vec<(Box<str>, Box<str>)>,
}

impl LogRecord {
    /// Creates a record with its severity and stable event name.
    pub fn new(level: LogLevel, event_name: impl Into<Box<str>>) -> Self {
        Self {
            level,
            event_name: event_name.into(),
            session: None,
            node: None,
            fields: Vec::new(),
        }
    }

    /// Attaches the session associated with the event.
    pub fn with_session(mut self, session: SessionId) -> Self {
        self.session = Some(session);
        self
    }

    /// Attaches the graph node associated with the event.
    pub fn with_node(mut self, node: NodeId) -> Self {
        self.node = Some(node);
        self
    }

    /// Adds an ordered, non-sensitive field to the record.
    ///
    /// This is fallible so callers cannot accidentally log reserved field names.
    pub fn with_field(
        mut self,
        name: impl Into<Box<str>>,
        value: impl Into<Box<str>>,
    ) -> Result<Self> {
        let name = name.into();
        if is_reserved_field(&name) {
            return Err(VoxaError::new(
                ErrorCategory::Validation,
                "VOXA-LOG-001",
                "log field name is reserved",
            ));
        }

        self.fields.push((name, value.into()));
        Ok(self)
    }

    /// Returns the record severity.
    pub const fn level(&self) -> LogLevel {
        self.level
    }

    /// Returns the stable event name.
    pub fn event_name(&self) -> &str {
        &self.event_name
    }

    /// Returns the associated session, if any.
    pub fn session(&self) -> Option<&SessionId> {
        self.session.as_ref()
    }

    /// Returns the associated graph node, if any.
    pub fn node(&self) -> Option<&NodeId> {
        self.node.as_ref()
    }

    /// Returns the record fields in insertion order.
    pub fn fields(&self) -> &[(Box<str>, Box<str>)] {
        &self.fields
    }
}

/// Receives structured log records without coupling callers to a logging backend.
pub trait LogSink: Send + Sync {
    /// Emits a structured record.
    fn emit(&self, record: &LogRecord);
}

/// A [`LogSink`] implementation backed by the `tracing` ecosystem.
#[derive(Clone, Copy, Debug, Default)]
pub struct TracingLogSink;

impl LogSink for TracingLogSink {
    fn emit(&self, record: &LogRecord) {
        match record.level() {
            LogLevel::Error => tracing::error!(
                event = %record.event_name(),
                session = ?record.session(),
                node = ?record.node(),
                fields = ?record.fields(),
                "Voxa event"
            ),
            LogLevel::Warn => tracing::warn!(
                event = %record.event_name(),
                session = ?record.session(),
                node = ?record.node(),
                fields = ?record.fields(),
                "Voxa event"
            ),
            LogLevel::Info => tracing::info!(
                event = %record.event_name(),
                session = ?record.session(),
                node = ?record.node(),
                fields = ?record.fields(),
                "Voxa event"
            ),
            LogLevel::Debug => tracing::debug!(
                event = %record.event_name(),
                session = ?record.session(),
                node = ?record.node(),
                fields = ?record.fields(),
                "Voxa event"
            ),
            LogLevel::Trace => tracing::trace!(
                event = %record.event_name(),
                session = ?record.session(),
                node = ?record.node(),
                fields = ?record.fields(),
                "Voxa event"
            ),
        }
    }
}

/// Initializes the default `tracing` formatter once without replacing an existing subscriber.
pub fn init_default_logging() -> Result<()> {
    DEFAULT_LOGGING.get_or_init(|| {
        let _ = tracing_subscriber::fmt().try_init();
    });

    Ok(())
}

fn is_reserved_field(name: &str) -> bool {
    ["payload", "authorization", "private_extension"]
        .iter()
        .any(|reserved| name.eq_ignore_ascii_case(reserved))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::{LogLevel, LogRecord, LogSink};
    use voxa_types::{NodeId, SessionId};

    #[derive(Default)]
    struct CollectSink {
        records: Mutex<Vec<LogRecord>>,
    }

    impl LogSink for CollectSink {
        fn emit(&self, record: &LogRecord) {
            self.records
                .lock()
                .expect("collecting sink lock")
                .push(record.clone());
        }
    }

    #[test]
    fn custom_sink_receives_structured_record() {
        let sink = CollectSink::default();
        let record = LogRecord::new(LogLevel::Info, "runtime.started")
            .with_session(SessionId::new("session-1").expect("valid session"))
            .with_field("worker_count", "2")
            .expect("safe field");

        sink.emit(&record);

        assert_eq!(
            sink.records
                .lock()
                .expect("collecting sink lock")
                .as_slice(),
            &[record]
        );
    }

    #[test]
    fn record_preserves_identity_and_field_insertion_order() {
        let record = LogRecord::new(LogLevel::Warn, "runtime.degraded")
            .with_session(SessionId::new("session-1").expect("valid session"))
            .with_node(NodeId::new("asr.primary").expect("valid node"))
            .with_field("attempt", "2")
            .expect("safe field")
            .with_field("reason", "timeout")
            .expect("safe field");

        assert_eq!(record.level(), LogLevel::Warn);
        assert_eq!(record.event_name(), "runtime.degraded");
        assert_eq!(record.session().map(SessionId::as_str), Some("session-1"));
        assert_eq!(record.node().map(NodeId::as_str), Some("asr.primary"));
        assert_eq!(
            record.fields(),
            &[
                (Box::<str>::from("attempt"), Box::<str>::from("2")),
                (Box::<str>::from("reason"), Box::<str>::from("timeout"),)
            ]
        );
    }

    #[test]
    fn rejects_payload_field_to_prevent_sensitive_logging() {
        let error = LogRecord::new(LogLevel::Info, "runtime.started")
            .with_field("payload", "audio bytes")
            .expect_err("payload must be rejected");

        assert_eq!(error.code(), "VOXA-LOG-001");
    }

    #[test]
    fn rejects_authorization_field_to_prevent_sensitive_logging() {
        let error = LogRecord::new(LogLevel::Info, "runtime.started")
            .with_field("authorization", "Bearer secret")
            .expect_err("authorization must be rejected");

        assert_eq!(error.code(), "VOXA-LOG-001");
    }

    #[test]
    fn rejects_private_extension_field_to_prevent_sensitive_logging() {
        let error = LogRecord::new(LogLevel::Info, "runtime.started")
            .with_field("private_extension", "secret")
            .expect_err("private extension must be rejected");

        assert_eq!(error.code(), "VOXA-LOG-001");
    }

    #[test]
    fn default_logging_initialization_is_idempotent() {
        super::init_default_logging().expect("first initialization");
        super::init_default_logging().expect("second initialization");
    }
}
