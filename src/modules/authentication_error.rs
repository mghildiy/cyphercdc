
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum AuthenticationError {
    UnsupportedMechanism(String),
    ScramPreparationFailed(String),
}

impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AuthenticationError::UnsupportedMechanism(msg) => write!(f, "Unsupported mechanism: {}", msg),
            AuthenticationError::ScramPreparationFailed(msg) => write!(f, "SCRAM preparation failed: {}", msg),
        }
    }
}