use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestedAudiobookMessage {
    pub audiobook_id: i64,
}

impl IngestedAudiobookMessage {
    #[must_use]
    pub fn new(audiobook_id: i64) -> Self {
        IngestedAudiobookMessage { audiobook_id }
    }
}
