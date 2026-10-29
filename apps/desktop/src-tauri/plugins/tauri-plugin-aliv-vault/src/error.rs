use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("{0}")]
    Invalid(String),
    #[error("blocked by matter policy: {0}")]
    PolicyBlocked(String),
}

pub type Result<T> = std::result::Result<T, VaultError>;

// Commands return errors to the webview as plain strings.
impl Serialize for VaultError {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
