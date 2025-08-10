use crate::ui_components::audiobook_container::AudioBookCollectionContainer;
use entities_lib::{entities::user::User, Author, GetAudioBookRequestType};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn AuthorPage() -> impl IntoView {
    let params = use_params_map();
    let _user_signal = use_context::<ReadSignal<User>>().unwrap();
    let maybe_author = move || params.read().get("author");
    let section_title = move || {
        format!(
            "Audiobooks by {}",
            maybe_author().unwrap_or_else(|| "".into())
        )
    };
    unsafe {
        view! {
            <Title text=section_title() />
            <div class="section">
                 <AudioBookCollectionContainer
                    title=Signal::derive(move || section_title())
                    request_type=Signal::derive(move || GetAudioBookRequestType::ByAuthor(Author {name: maybe_author().unwrap_unchecked()}))
                />
            </div>
        }
    }
}
