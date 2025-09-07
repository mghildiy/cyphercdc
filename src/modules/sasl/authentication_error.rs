
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum AuthenticationError {
    UnsupportedMechanism(String),
    ScramPreparationFailed(String),
    ClientKeyGenerationFailed(String),
    IllegalState(String),
    ConnectionFailed(String),
    SASLAuthenticationFailed(String),
    GenericError(String),
}

impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AuthenticationError::UnsupportedMechanism(msg) => write!(f, "Unsupported mechanism: {}", msg),
            AuthenticationError::ScramPreparationFailed(msg) => write!(f, "SCRAM preparation failed: {}", msg),
            AuthenticationError::ClientKeyGenerationFailed(msg) => write!(f, "Client key generation failed: {}", msg),
            AuthenticationError::IllegalState(msg) => write!(f, "Missing details: {}", msg),
            AuthenticationError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            AuthenticationError::SASLAuthenticationFailed(msg) => write!(f, "SASL authentication failed: {}", msg),
            AuthenticationError::GenericError(msg) => write!(f, "Generic error: {}", msg),
        }
    }
}