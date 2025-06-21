use crate::ui_components::audiobook_container::AudioBookCollectionContainer;
use entities_lib::{entities::user::User, GetAudioBookRequestType};
use leptos::{logging, prelude::*};

#[island]
pub fn HomePage() -> impl IntoView {
    let _user_signal = use_context::<ReadSignal<User>>().unwrap();

    view! {
        <div class="section">
             <AudioBookCollectionContainer
                title={String::from("Most recent")}
                request_type=GetAudioBookRequestType::MostRecent
            />
        </div>
    }
}
