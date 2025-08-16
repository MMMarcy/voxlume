use crate::ui_components::audiobook_container::AudioBookCollectionContainer;
use entities_lib::{Category, GetAudioBookRequestType};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn CategoryPage() -> impl IntoView {
    let params = use_params_map();
    let maybe_category = move || params.read().get("category");
    let section_title = move || {
        format!(
            "Audiobooks with category {}",
            maybe_category().unwrap_or_else(String::new)
        )
    };

    // Note: The `unwrap()` call below will panic if the "category" param is missing.
    // It's safer to handle the `None` case, for example, by conditionally
    // rendering the component or showing a "Not Found" message.
    view! {
        <Title text=section_title />
        <div class="section">
             <AudioBookCollectionContainer
                title=Signal::derive(section_title)
                request_type=Signal::derive(move || GetAudioBookRequestType::ByCategory(Category {value: maybe_category().unwrap()}, 0))
            />
        </div>
    }
}
