use entities_lib::{AudiobookWithData, SubscriptionType, User};
use leptos::prelude::*;
use web_sys::TouchEvent;

use crate::ui_components::subscriptions::subscriptions_panel::SubscriptionPanel;

use leptos_router::components::A;

use crate::ui_components::audiobook::link_author::AuthorLinks;
use crate::ui_components::audiobook::link_reader::ReaderLinks;
use crate::ui_components::audiobook::link_series::SeriesLink;
use crate::ui_components::audiobook::tag_category::CategoriesTag;
use crate::ui_components::audiobook::tag_keyword::KeywordsTag;

#[allow(clippy::too_many_lines)]
#[component]
pub fn AudioBookComponentBox(audiobook_with_data: AudiobookWithData) -> impl IntoView {
    let (audiobook, authors, categories, keywords, readers, maybe_series) = audiobook_with_data;
    let title = audiobook.title;
    let cover_url = audiobook.cover_url;
    let very_short_description = audiobook.very_short_description;
    let fallback_image = "https://placehold.co/400x400?text=Cover+not+found";

    let (img_src, set_img_src) = signal(cover_url);

    view! {
        <div class="card audiobook-card">
            <div class="card-image">
                <A href=format!("/audiobook/{}", &audiobook.id)>
                    <figure class="image audiobook-cover-container">
                        <img
                            src=img_src
                            alt=""
                            class="audiobook-cover-blur"
                        />
                        <img
                            src=img_src
                            on:error=move |_| set_img_src.set(Some(fallback_image.to_string()))
                            alt="Audiobook cover"
                            class="audiobook-cover-front"
                        />
                    </figure>
                </A>
            </div>

            <div class="card-content audiobook-content">
                <div class="media mb-4">
                    <div class="media-content audiobook-info">
                        <A href=format!("/audiobook/{}", &audiobook.id)>
                            <p class="title is-5 has-text-weight-bold mb-2 audiobook-title">
                                {title}
                            </p>
                        </A>
                        <div class="is-size-7">
                            <AuthorLinks authors=authors/>
                            <ReaderLinks readers=readers limit=3/>
                            <SeriesLink maybe_series=maybe_series/>
                        </div>
                    </div>
                </div>

                <div class="content is-size-7">
                    <p class="audiobook-description">
                        {very_short_description}
                    </p>
                    <div class="mt-3">
                        <CategoriesTag categories=categories limit=5/>
                        <div class="mt-2">
                            <KeywordsTag keywords=keywords limit=5/>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub(crate) fn AudioBookCollectionContainer_Grid(
    title: Signal<String>,
    audiobooks: Signal<Option<Vec<AudiobookWithData>>>,
    subscription_type: Option<Signal<SubscriptionType>>,
) -> impl IntoView {
    let user_signal = use_context::<RwSignal<Option<User>>>().unwrap();
    let audiobooks_loaded = move || audiobooks.get().is_some();
    let (is_hover, set_is_hover) = signal(false);
    let on_hover_handler = move |_| {
        if user_signal().is_some() {
            set_is_hover.update(|v| *v = !*v);
        }
    };
    let on_touch_hanlder = move |_: TouchEvent| {
        set_is_hover.update(|v| *v = !*v);
    };
    view! {
        <section class="section">
            <div class="container">
                <div class="level mb-5">
                    <div class="level-left">
                        <div class="level-item">
                            <h2 class="title is-3">
                                <span class="icon is-medium has-text-link mr-3">
                                    <i class="fas fa-book"></i>
                                </span>
                                {title}
                            </h2>
                        </div>
                    </div>
                    <div class="level-right">
                        <SubscriptionPanel sub_type=subscription_type/>
                    </div>
                </div>

                <div
                    class="columns is-multiline"
                    class:overlow_x_visible=is_hover
                    on:mouseenter=on_hover_handler
                    on:mouseleave=on_hover_handler
                    on:touchstart=on_touch_hanlder
                    // on:touchend=on_touch_hanlder
                    on:touchcancel=on_touch_hanlder
                >

                    <Show when=audiobooks_loaded fallback=move || view! {
                        <div class="column is-full has-text-centered py-6">
                            <span class="icon is-large has-text-grey-light">
                                <i class="fas fa-circle-notch fa-spin fa-3x"></i>
                            </span>
                        </div>
                    }>
                        // TODO: audiobook should have a key that is Copy
                        <For
                            each=move || audiobooks().unwrap()
                            key=|v| v.0.path.clone()
                            let(audiobook_with_data)
                        >
                            <div class="column is-12-mobile is-6-tablet is-4-desktop is-3-widescreen">
                                <AudioBookComponentBox audiobook_with_data=audiobook_with_data/>
                            </div>
                        </For>
                    </Show>

                </div>
            </div>
        </section>
    }
}
