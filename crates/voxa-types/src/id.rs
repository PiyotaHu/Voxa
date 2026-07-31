use std::{error::Error, fmt, str::FromStr};

/// The reason an identifier value was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentifierError {
    /// The identifier has no bytes.
    Empty,
    /// The identifier exceeds the maximum length of 255 UTF-8 bytes.
    TooLong,
    /// The identifier begins or ends with whitespace.
    LeadingOrTrailingWhitespace,
    /// The identifier contains an ASCII control character.
    ContainsControlCharacter,
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("identifier must not be empty"),
            Self::TooLong => formatter.write_str("identifier must be at most 255 bytes"),
            Self::LeadingOrTrailingWhitespace => {
                formatter.write_str("identifier must not have leading or trailing whitespace")
            }
            Self::ContainsControlCharacter => {
                formatter.write_str("identifier must not contain ASCII control characters")
            }
        }
    }
}

impl Error for IdentifierError {}

fn validate_identifier(value: &str) -> Result<(), IdentifierError> {
    if value.is_empty() {
        return Err(IdentifierError::Empty);
    }
    if value.len() > 255 {
        return Err(IdentifierError::TooLong);
    }
    if value.trim() != value {
        return Err(IdentifierError::LeadingOrTrailingWhitespace);
    }
    if value.chars().any(|character| character.is_ascii_control()) {
        return Err(IdentifierError::ContainsControlCharacter);
    }

    Ok(())
}

macro_rules! identifier_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<str>);

        impl $name {
            /// Creates an identifier after validating its value.
            pub fn new(value: impl Into<Box<str>>) -> Result<Self, IdentifierError> {
                let value = value.into();
                validate_identifier(&value)?;
                Ok(Self(value))
            }

            /// Returns the identifier value.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = IdentifierError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }
    };
}

identifier_type!(
    /// Identifies a node in a Voxa graph.
    ///
    /// ```compile_fail
    /// use voxa_types::{NodeId, SessionId};
    ///
    /// fn needs_node(_: NodeId) {}
    ///
    /// let session = SessionId::new("session-1").unwrap();
    /// needs_node(session);
    /// ```
    NodeId
);
identifier_type!(
    /// Identifies a Voxa session.
    SessionId
);
identifier_type!(
    /// Identifies a stream within a session.
    StreamId
);
identifier_type!(
    /// Identifies a trace.
    TraceId
);

#[cfg(test)]
mod tests {
    use super::{NodeId, SessionId, StreamId, TraceId};

    #[test]
    fn identifiers_validate_and_round_trip() {
        let node: NodeId = "asr.primary".parse().expect("valid node id");
        assert_eq!(node.as_str(), "asr.primary");
        assert_eq!(node.to_string(), "asr.primary");
        assert!(SessionId::new("").is_err());
        assert!(StreamId::new(" audio ").is_err());
        assert!(TraceId::new("trace\n1").is_err());
    }
}
