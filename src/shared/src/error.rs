use std::fmt;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Domain error: {0}")]
    Domain(String),

    #[error("Infrastructure error: {0}")]
    Infrastructure(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("WebRTC error: {0}")]
    WebRTC(String),

    #[error("Command execution error: {0}")]
    CommandExecution(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Infrastructure(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Infrastructure(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Configuration(format!("JSON error: {}", err))
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Configuration(format!("TOML deserialization error: {}", err))
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::Configuration(format!("TOML serialization error: {}", err))
    }
}

#[derive(Debug)]
pub struct AppError {
    pub message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}
