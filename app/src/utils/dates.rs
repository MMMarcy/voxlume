use chrono::{DateTime, Utc};

// TODO: This should be moved to server side to avoid shipping chrono to the client.
pub fn print_date(date: DateTime<Utc>) -> String {
    format!("{}", date.format("%d/%m/%Y %H:%M"))
}
