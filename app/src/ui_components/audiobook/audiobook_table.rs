use entities_lib::{AudiobookWithData, SubscriptionType};
use leptos::prelude::*;
use leptos_router::components::A;

use crate::ui_components::audiobook::link_author::AuthorLinks;
use crate::ui_components::audiobook::link_reader::ReaderLinks;
use crate::ui_components::audiobook::link_series::SeriesLink;
use crate::ui_components::subscriptions::subscriptions_panel::SubscriptionPanel;

#[component]
pub fn AudiobookRow(audiobook_with_data: AudiobookWithData) -> impl IntoView {
    let (audiobook, authors, _, _, readers, maybe_series) = audiobook_with_data;
    let fallback_image = "https://placehold.co/64x64?text=X";
    let (img_src, set_img_src) = signal(audiobook.cover_url.clone());

    view! {
        <tr class="audiobook-table-row">
            <td class="cover-cell">
                <A href=format!("/audiobook/{}", &audiobook.id)>
                    <figure class="image is-128x128 table-cover-container">
                        <img
                            src=img_src
                            alt=""
                            class="table-cover-blur"
                        />
                        <img
                            src=img_src
                            on:error=move |_| set_img_src.set(Some(fallback_image.to_string()))
                            alt="Cover"
                            class="table-cover-front"
                        />
                    </figure>
                </A>
            </td>
            <td>
                <A href=format!("/audiobook/{}", &audiobook.id)>
                    <span class="has-text-weight-bold has-text-link">{audiobook.title}</span>
                </A>
            </td>
            <td><AuthorLinks authors=authors/></td>
            <td><ReaderLinks readers=readers limit=2/></td>
            <td><SeriesLink maybe_series=maybe_series/></td>
        </tr>
    }
}

#[component]
#[allow(non_snake_case)]
pub(crate) fn AudioBookCollectionContainer_Table(
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
                                    <i class="fas fa-list"></i>
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
                    <div class="table-container">
                        <table class="table is-fullwidth is-striped is-hoverable">
                            <thead>
                                <tr>
                                    <th>"Cover"</th>
                                    <th>"Title"</th>
                                    <th>"Authors"</th>
                                    <th>"Readers"</th>
                                    <th>"Series"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || audiobooks().unwrap()
                                    key=|v| v.0.id
                                    let(audiobook_with_data)
                                >
                                    <AudiobookRow audiobook_with_data=audiobook_with_data/>
                                </For>
                            </tbody>
                        </table>
                    </div>
                </Show>
            </div>
        </section>
    }
}
