use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum Language {
    Unknown = 0,

    English = 1,

    Italian = 2,
}

#[derive(Clone, Copy, Deserialize, EnumString)]
pub enum RelationshipNames {
    Written,
}
