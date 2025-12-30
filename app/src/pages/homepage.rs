use crate::ui_components::audiobook::audiobook_container::AudioBookCollectionContainer;
use entities_lib::GetAudioBookRequestType;
use leptos::prelude::*;

use crate::ui_components::ads::grid_ad::GridAd;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <section class="hero is-info is-medium">
            <div class="hero-body">
                <div class="container has-text-centered">
                    <p class="title is-1">"Welcome to Voxlume"</p>
                    <p class="subtitle is-3">"Discover your next favorite story"</p>
                </div>
            </div>
        </section>

        <section class="section">
            <div class="container content has-text-centered">
                <p class="is-size-5">
                    "Voxlume is a sane, privacy-focused, and ad-supported audiobook search store."
                    "We believe in providing a less spammy experience where you can easily find "
                    "new books you're interested in, without constant upsells."
                    "Tired of platforms that push purchases over discovery? Voxlume is built for you."
                </p>
                <p class="is-size-6">
                    "We do not host any of the files."
                </p>
            </div>
        </section>

        <div class="container is-max-desktop is-flex is-justify-content-center">
            <GridAd ad_slot="1117011249" />
        </div>

        <div class="section">
            <AudioBookCollectionContainer
                title=Signal::derive(move || String::from("Fresh Arrivals"))
                request_type=Signal::derive(move || GetAudioBookRequestType::MostRecent(1))
                subscription_type=None
            />
        </div>

    }
}
