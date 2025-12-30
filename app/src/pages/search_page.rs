use crate::ui_components::ads::grid_ad::GridAd;
use crate::ui_components::audiobook::audiobook_container::AudioBookCollectionContainer;
use entities_lib::{GetAudioBookRequestType, SearchQuery};
use leptos::prelude::*;
use leptos::{Params, logging};
use leptos_meta::Title;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[allow(clippy::unused_async)]
#[server(SearchAudiobooks, "/api")]
async fn search_audiobooks_server_fn(search_query: SearchQuery) -> Result<Vec<i64>, ServerFnError> {
    use shared::db_ops::parade::search_ops::search_audiobooks;
    use shared::state::AppState;
    use tracing::{debug, info};

    debug!("Before getting app state");
    let state = AppState::get_app_state()?;
    debug!("Gotten app state");
    let _limit_audiobooks = state.shareable_args.max_search_results;
    debug!("Pulled what we needed out of the context.");
    info!("Gotten results for query {:?}", search_query);
    search_audiobooks(
        &state.database_connection_pool,
        &search_query,
        &state.shareable_args,
        state.embedder,
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[derive(Params, PartialEq)]
struct SearchParam {
    search_string: Option<String>,
}

#[component]
pub fn SearchPage() -> impl IntoView {
    let params = use_params::<SearchParam>();
    let search_query = move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|p| p.search_string.clone())
            .unwrap()
    };
    let section_title = move || format!("Search results: {}", search_query());
    let audiobook_ids: RwSignal<Option<Vec<i64>>> = RwSignal::new(None);

    let get_audiobooks_ids_resource = Resource::new(search_query, move |query| {
        let sq = SearchQuery {
            search_string: query,
        };
        search_audiobooks_server_fn(sq)
    });

    Effect::new(move || {
        let result = get_audiobooks_ids_resource.get();
        match result {
            Some(Ok(data)) => {
                logging::debug_warn!("Found {} audiobook ids", &data.len());
                audiobook_ids.set(Some(data));
            }
            Some(Err(e)) => {
                logging::debug_warn!("{:?}", e);
                audiobook_ids.set(None);
            }
            None => {
                audiobook_ids.set(None);
            }
        }
    });

    view! {
        <div class="container is-max-desktop is-flex is-justify-content-center">
            <GridAd ad_slot="1117011249" />
        </div>
        <Title text=section_title />
        <Show
            when=move || audiobook_ids.get().is_some()
            fallback=move || view! { <p>"loading"</p> }
        >
            <Show
                when=move || !audiobook_ids.get().unwrap().is_empty()
                fallback=move || view! { <p>No audiobooks found.</p> }
            >
                <AudioBookCollectionContainer
                    title=Signal::derive(move || {
                        format!("Search results for query: {}", search_query())
                    })
                    request_type=Signal::derive(move || GetAudioBookRequestType::ByIdList(
                        audiobook_ids.get().unwrap(),
                    ))
                    subscription_type=None
                />
            </Show>

        </Show>
    }
}
