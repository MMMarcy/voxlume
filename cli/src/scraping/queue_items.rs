use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum QueueTask {
    ParseSubmissionPage(String),
    ParseAudiobookPage {
        url: String,
        submission_date: String,
    },
}
