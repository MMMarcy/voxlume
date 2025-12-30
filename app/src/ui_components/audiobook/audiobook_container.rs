use super::audiobook_grid::AudioBookCollectionContainer_Grid;
use super::audiobook_list::AudioBookCollectionContainer_List;
use super::audiobook_table::AudioBookCollectionContainer_Table;
use entities_lib::entities::audiobook_display::AudiobookDisplayMode;
use entities_lib::{GetAudioBookRequestType, SubscriptionType};
use leptos::leptos_dom::{debug_error, debug_log};
use leptos::prelude::*;

use super::get_audiobooks;

#[component]
pub fn AudioBookCollectionContainer(
    title: Signal<String>,
    request_type: Signal<GetAudioBookRequestType>,
    subscription_type: Option<Signal<SubscriptionType>>,
) -> impl IntoView {
    let display_mode = use_context::<ReadSignal<AudiobookDisplayMode>>().unwrap();

    // Use standard Resource. SsrMode::Async + Suspense ensures the server waits.
    let audiobooks_resource = Resource::new(move || request_type.get(), get_audiobooks);

    // Derived signal that extracts the data
    let audiobooks = Signal::derive(move || {
        audiobooks_resource.get().and_then(|res| match res {
            Ok(data) => {
                debug_log!("Found {} audiobooks", &data.len());
                Some(data)
            }
            Err(e) => {
                debug_error!("{:?}", e);
                None
            }
        })
    });

    view! {
        <Suspense fallback=move || view! {
            <div class="has-text-centered py-6">
                <span class="icon is-large has-text-grey-light">
                    <i class="fas fa-circle-notch fa-spin fa-3x"></i>
                </span>
            </div>
        }>
            {move || {
                // We access the signal here to register dependency with Suspense
                let _ = audiobooks.get();

                match display_mode() {
                    AudiobookDisplayMode::TableLike => view! {
                        <AudioBookCollectionContainer_Table
                            title=title
                            audiobooks=audiobooks
                            subscription_type=subscription_type
                        />
                    }.into_any(),
                    AudiobookDisplayMode::ListLike => view! {
                        <AudioBookCollectionContainer_List
                            title=title
                            audiobooks=audiobooks
                            subscription_type=subscription_type
                        />
                    }.into_any(),
                    AudiobookDisplayMode::GridLike => view! {
                        <AudioBookCollectionContainer_Grid
                            title=title
                            audiobooks=audiobooks
                            subscription_type=subscription_type
                        />
                    }.into_any(),
                }
            }}
        </Suspense>
    }
}
