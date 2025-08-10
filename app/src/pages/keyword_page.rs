use crate::ui_components::audiobook_container::AudioBookCollectionContainer;
use entities_lib::{entities::user::User, GetAudioBookRequestType, Keyword};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn KeywordPage() -> impl IntoView {
    let params = use_params_map();
    let _user_signal = use_context::<ReadSignal<User>>().unwrap();
    let maybe_keyword = move || params.read().get("keyword");
    let section_title = || {
        format!(
            "Audiobooks with keyword {}",
            maybe_keyword().unwrap_or_else(|| "".into())
        )
    };

    unsafe {
        view! {
            <Title text=section_title() />
            <div class="section">
                 <AudioBookCollectionContainer
                    title=section_title()
                    request_type=GetAudioBookRequestType::ByKeyword(Keyword {value: maybe_keyword().unwrap_unchecked()})
                />
            </div>
        }
    }
}
