use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum GetAudioBookRequestType {
    MostRecent = 0,
    ByAuthor = 1,
}
