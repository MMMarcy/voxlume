use entities_lib::{AudiobookWithData, SubscriptionType};
use leptos::prelude::*;
use leptos_router::components::A;

use crate::ui_components::audiobook::link_author::AuthorLinks;
use crate::ui_components::audiobook::link_reader::ReaderLinks;
use crate::ui_components::audiobook::link_series::SeriesLink;
use crate::ui_components::audiobook::tag_category::CategoriesTag;
use crate::ui_components::audiobook::tag_keyword::KeywordsTag;
use crate::ui_components::subscriptions::subscriptions_panel::SubscriptionPanel;

#[component]
pub fn AudiobookListItem(audiobook_with_data: AudiobookWithData) -> impl IntoView {
    let (audiobook, authors, categories, keywords, readers, maybe_series) = audiobook_with_data;
    let fallback_image = "https://placehold.co/128x128?text=No+Cover";
    let (img_src, set_img_src) = signal(audiobook.cover_url.clone());

    view! {
        <div class="box mb-4">
            <article class="media">
                <figure class="media-left">
                    <A href=format!("/audiobook/{}", &audiobook.id)>
                        <p class="image is-96x96">
                             <img
                                src=img_src
                                on:error=move |_| set_img_src.set(Some(fallback_image.to_string()))
                                alt="Cover"
                                style="object-fit: cover; border-radius: 4px;"
                            />
                        </p>
                    </A>
                </figure>
                <div class="media-content">
                    <div class="content">
                        <p>
                            <A href=format!("/audiobook/{}", &audiobook.id)>
                                <strong class="title is-5">{audiobook.title}</strong>
                            </A>
                            <br/>
                            <div class="is-size-7 mb-2 mt-1">
                                <span class="mr-3"><AuthorLinks authors=authors/></span>
                                <span class="mr-3"><ReaderLinks readers=readers limit=5/></span>
                                <span><SeriesLink maybe_series=maybe_series/></span>
                            </div>
                            <span class="is-size-7" style="display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;">
                                {audiobook.very_short_description}
                            </span>
                        </p>
                    </div>
                    <div class="level is-mobile mb-1">
                        <div class="level-left">
                             <CategoriesTag categories=categories limit=10/>
                        </div>
                    </div>
                    <div class="level is-mobile">
                        <div class="level-left">
                             <KeywordsTag keywords=keywords limit=10/>
                        </div>
                    </div>
                </div>
            </article>
        </div>
    }
}

#[component]
#[allow(non_snake_case)]
pub(crate) fn AudioBookCollectionContainer_List(
    title: Signal<String>,
    audiobooks: Signal<Option<Vec<AudiobookWithData>>>,
    subscription_type: Option<Signal<SubscriptionType>>,
) -> impl IntoView {
    let audiobooks_loaded = move || audiobooks.get().is_some();

    view! {
        <section class="section">
            <div class="container">
                 <div class="level mb-5">
                    <div class="level-left">
                        <div class="level-item">
                            <h2 class="title is-3">
                                <span class="icon is-medium has-text-link mr-3">
                                    <i class="fas fa-th-list"></i>
                                </span>
                                {title}
                            </h2>
                        </div>
                    </div>
                    <div class="level-right">
                        <SubscriptionPanel sub_type=subscription_type/>
                    </div>
                </div>

                <Show when=audiobooks_loaded fallback=move || view! {
                    <div class="has-text-centered py-6">
                        <span class="icon is-large has-text-grey-light">
                            <i class="fas fa-circle-notch fa-spin fa-3x"></i>
                        </span>
                    </div>
                }>
                    <div class="content">
                         <For
                            each=move || audiobooks().unwrap()
                            key=|v| v.0.id
                            let(audiobook_with_data)
                        >
                            <AudiobookListItem audiobook_with_data=audiobook_with_data/>
                        </For>
                    </div>
                </Show>
            </div>
        </section>
    }
}
