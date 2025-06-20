use crate::ui_components::audiobook_container::AudioBookCollectionContainer;
use entities_lib::entities::user::User;
use leptos::{logging, prelude::*};

#[component]
pub fn HomePage() -> impl IntoView {
    let _user_signal = use_context::<ReadSignal<User>>().unwrap();

    view! {
        <div class="section">
             <AudioBookCollectionContainer title={String::from("Test")} />
        </div>
    }
}
