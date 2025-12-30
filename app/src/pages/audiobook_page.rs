use entities_lib::{AudiobookWithData, GetAudioBookRequestType};
use leptos::prelude::*;
use leptos::{Params, logging};
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

use crate::ui_components::ads::grid_ad::GridAd;
use crate::ui_components::audiobook::audiobook_detailed_view::AudioBookDetailedViewComponent;

#[derive(Params, PartialEq)]
struct AudiobookIdParam {
    audiobook_id: Option<String>,
}

#[server(GetAudiobookById, "/api")]
async fn get_audiobook_by_id(
    request_type: GetAudioBookRequestType,
) -> Result<Option<AudiobookWithData>, ServerFnError> {
    use shared::db_ops::parade::audiobook_ops::get_audiobooks_cached;
    use shared::state::AppState;
    use tracing::debug;

    let state = AppState::get_app_state()?;
    debug!("Gotten app state");
    let pgpool = state.database_connection_pool;
    let cache = state.audiobooks_cache;
    get_audiobooks_cached(
        &pgpool,
        &cache,
        request_type,
        1, // Getting only one audiobook since we are retrieving by id.
    )
    .await
    .map(|audiobooks_vec| audiobooks_vec.into_iter().next())
    .map_err(|e| ServerFnError::new(format!("{e:?}")))
}

#[component]
pub fn AudiobookDetailedView() -> impl IntoView {
    let params = use_params::<AudiobookIdParam>();
    let audiobook_id = move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|p| p.audiobook_id.clone())
            .unwrap()
    };

    let audiobook: RwSignal<Option<AudiobookWithData>> = RwSignal::new(None);
    let audiobook_loaded = Signal::derive(move || audiobook.get().is_some());

    let get_audiobook_resource = Resource::new(audiobook_id, move |audiobook_id| {
        get_audiobook_by_id(GetAudioBookRequestType::ById(audiobook_id))
    });

    Effect::new(move || {
        let result = get_audiobook_resource.get();
        match result {
            Some(Ok(data)) => {
                logging::debug_warn!("Found audiobook with id {}", audiobook_id());
                audiobook.set(data);
            }
            Some(Err(e)) => {
                logging::debug_warn!("{:?}", e);
                audiobook.set(None);
            }
            None => {
                audiobook.set(None);
            }
        }
    });

    view! {
        <Show when=audiobook_loaded fallback=move || view! { "Loading" }>
                <AudioBookDetailedViewComponent audiobook_with_data=audiobook.get().unwrap() />
        </Show>
        <div class="container is-max-desktop is-flex is-justify-content-center">
            <GridAd ad_slot="1117011249" />
        </div>

    }
}
