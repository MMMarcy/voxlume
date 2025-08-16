use crate::ui_components::audiobook_container::AudioBookCollectionContainer;
use entities_lib::{entities::user::User, GetAudioBookRequestType, Reader};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn ReaderPage() -> impl IntoView {
    let params = use_params_map();
    let _user_signal = use_context::<ReadSignal<User>>().unwrap();
    let maybe_reader = move || params.read().get("reader");
    let section_title = move || {
        format!(
            "Audiobooks read by {}",
            maybe_reader().unwrap_or_else(String::new)
        )
    };
    unsafe {
        view! {
            <Title text=section_title />
            <div class="section">
                 <AudioBookCollectionContainer
                    title=Signal::derive(section_title)
                    request_type=Signal::derive(move || GetAudioBookRequestType::ByReader(
                Reader { name: maybe_reader().unwrap_unchecked()}, 0))

                />
            </div>
        }
    }
}
