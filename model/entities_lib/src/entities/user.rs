use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use sqlx::FromRow;

pub const GUEST_USER_ID: i64 = 1;

/// User entity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct User {
    /// Id
    pub id: i64,

    /// The timestamp when the change list item was created.
    pub username: String,

    /// Timestamp of last access.
    pub last_access: i64,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: GUEST_USER_ID,
            username: String::from("Guest"),
            last_access: 0_i64,
        }
    }
}

impl User {
    pub fn is_guest(&self) -> bool {
        self.id == GUEST_USER_ID
    }
}
