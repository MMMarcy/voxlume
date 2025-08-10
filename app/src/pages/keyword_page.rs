use crate::ui_components::audiobook_container::AudioBookCollectionContainer;
use entities_lib::{GetAudioBookRequestType, Keyword};
use leptos::logging;
use leptos::prelude::*;
use leptos::Params;
use leptos_meta::Title;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[derive(Params, PartialEq)]
struct KeywordParam {
    keyword: Option<String>,
}

#[component]
pub fn KeywordPage() -> impl IntoView {
    let params = use_params::<KeywordParam>();
    let keyword = move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|p| p.keyword.clone())
            .unwrap()
    };
    let section_title = move || {
        logging::log!("{}", "title changed");
        format!("Audiobooks with keyword {}", keyword())
    };

    view! {
        <Title text=section_title() />
        <div class="section">
             <AudioBookCollectionContainer
                title=Signal::derive(move || format!("Audiobooks with keyword {}", keyword()))
                request_type=Signal::derive(move || GetAudioBookRequestType::ByKeyword(Keyword {value: keyword()}))
            />
        </div>
    }
}
