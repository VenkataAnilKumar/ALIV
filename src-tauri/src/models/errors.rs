use thiserror::Error;

#[derive(Debug, Error)]
pub enum AlivError {
    #[error("database error: {0}")]
    Database(String),
    #[error("python worker error: {0}")]
    PythonBridge(String),
    #[error("safety block: {0}")]
    SafetyBlock(String),
    #[error("validation error: {0}")]
    Validation(String),
}
