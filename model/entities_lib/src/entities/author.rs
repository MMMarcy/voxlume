//! Author stuff

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Author {
    pub id: i64,
    pub name: String,
}
