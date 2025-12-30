use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use clap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(clap::ValueEnum))]
pub enum Environment {
    DEV,
    PROD,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(clap::Args))]
pub struct ShareableArgsValues {
    #[cfg_attr(feature = "ssr", clap(value_enum))]
    #[cfg_attr(feature = "ssr", arg(long, default_value_t = Environment::DEV))]
    pub environment: Environment,

    #[cfg_attr(feature = "ssr", arg(long, default_value_t = 24))]
    pub guest_user_audiobooks_per_homepage: u32,

    #[cfg_attr(feature = "ssr", arg(long, default_value_t = 8))]
    pub user_audiobooks_per_homepage_section: u32,

    #[cfg_attr(feature = "ssr", arg(long, default_value_t = 24))]
    pub max_search_results: i32,

    #[cfg_attr(feature = "ssr", arg(long, default_value_t = String::from("models/gemini-2.5-flash-lite")))]
    pub gemini_extract_html_model_name: String,

    #[cfg_attr(feature = "ssr", arg(long, default_value_t = String::from("models/gemini-2.5-flash")))]
    pub gemini_text_generation_model_name: String,

    #[cfg_attr(feature = "ssr", arg(long, default_value_t = String::from("models/gemini-embedding-001")))]
    pub gemini_embedding_model_name: String,

    #[cfg_attr(feature = "ssr", arg(long, default_value_t = 768))]
    pub gemini_embeddings_size: u16,

    #[cfg_attr(feature = "ssr", arg(long, default_value_t = String::from("audiobookbay")))]
    pub audiobookbay_domain: String,

    #[cfg_attr(
        feature = "ssr",
        arg(long, value_delimiter = ',', default_value = "is,lu")
    )]
    pub audiobookbay_extensions: Vec<String>,
}
