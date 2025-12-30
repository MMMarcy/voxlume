use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// This enum maps directly to the PostgreSQL `notification_reason` enum.
// The `sqlx::Type` derive macro is crucial for this mapping.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[cfg_attr(
    feature = "ssr",
    sqlx(type_name = "notification_reason", rename_all = "snake_case")
)]
pub enum NotificationReason {
    MatchSeries,
    MatchKeyword,
    MatchCategory,
    MatchAuthor,
    MatchReader,
}

// This struct represents a single row in the `user_notification` table.
// The `sqlx::FromRow` macro allows `sqlx` to build this struct from a query result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct UserNotification {
    pub user_id: i64,
    pub audiobook_id: i64,
    pub created_at: DateTime<Utc>,
    // `sqlx` automatically handles mapping a PG array to a Rust Vec.
    pub reasons: Vec<NotificationReason>,
    pub has_been_seen: bool,
}

impl UserNotification {
    pub fn format_reasons(&self) -> String {
        let mut s = String::new();
        s.push_str("<ol>");
        if self.reasons.contains(&NotificationReason::MatchSeries) {
            s.push_str("<li>Matches a series you follow</li>");
        }
        if self.reasons.contains(&NotificationReason::MatchAuthor) {
            s.push_str("<li>Matches an author you follow</li>");
        }
        if self.reasons.contains(&NotificationReason::MatchCategory) {
            s.push_str("<li>Matches a category you follow</li>");
        }
        if self.reasons.contains(&NotificationReason::MatchKeyword) {
            s.push_str("<li>Matches a keyword you follow</li>");
        }
        s.push_str("</ol>");

        s
    }
}

impl Display for UserNotification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id: {}, audiobook_id: {}, created_at: {}, reasons: {:?}, seen: {}",
            self.user_id, self.audiobook_id, self.created_at, self.reasons, self.has_been_seen
        )
    }
}

impl PartialEq for UserNotification {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id && self.audiobook_id == other.audiobook_id
    }
}
