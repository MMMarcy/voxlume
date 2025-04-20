use serde::{Deserialize, Serialize};

/// User entity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    /// The timestamp when the change list item was created.
    pub username: String,
    /// The timestamp when the change list item was last updated.
    pub password_hash: String,

    /// Salt
    pub salt_used: String,

    /// Timestamp of last access.
    pub last_access: u64,
}
