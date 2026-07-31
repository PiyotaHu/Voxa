#![forbid(unsafe_code)]
//! Consumer-facing examples for the public Voxa crates.

use voxa_types::SessionId;

/// Formats the readiness message emitted by the hello example.
pub fn hello_message(session: &SessionId) -> String {
    format!("Voxa runtime ready: {}", session.as_str())
}

#[cfg(test)]
mod tests {
    use super::hello_message;
    use voxa_types::SessionId;

    #[test]
    fn hello_message_contains_typed_session_id() {
        let session = SessionId::new("hello-session").unwrap();
        assert_eq!(hello_message(&session), "Voxa runtime ready: hello-session");
    }
}
