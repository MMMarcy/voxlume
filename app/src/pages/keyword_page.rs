use crate::ui_components::audiobook_container::AudioBookCollectionContainer;
use entities_lib::{GetAudioBookRequestType, Keyword};
use leptos::logging;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn KeywordPage() -> impl IntoView {
    let params = use_params_map();
    let maybe_keyword = move || params.read().get("keyword");
    let section_title = move || {
        logging::log!("{}", "title changed");
        format!(
            "Audiobooks with keyword {}",
            maybe_keyword().unwrap_or_else(|| "".into())
        )
    };

    view! {
        <Title text=section_title() />
        <div class="section">
             <AudioBookCollectionContainer
                title=section_title()
                request_type=GetAudioBookRequestType::ByKeyword(Keyword {value: maybe_keyword().unwrap()})
            />
        </div>
    }
}
