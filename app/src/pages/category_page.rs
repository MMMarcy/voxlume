use crate::ui_components::audiobook_container::AudioBookCollectionContainer;
use entities_lib::{entities::user::User, Category, GetAudioBookRequestType};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn CategoryPage() -> impl IntoView {
    let params = use_params_map();
    let _user_signal = use_context::<ReadSignal<User>>().unwrap();
    let maybe_category = move || params.read().get("category");
    let section_title = || {
        format!(
            "Audiobooks with category {}",
            maybe_category().unwrap_or_else(|| "".into())
        )
    };
    unsafe {
        view! {
            <Title text=section_title() />
            <div class="section">
                 <AudioBookCollectionContainer
                    title=section_title()
                    request_type=GetAudioBookRequestType::ByCategory(Category {value: maybe_category().unwrap_unchecked()})
                />
            </div>
        }
    }
}
