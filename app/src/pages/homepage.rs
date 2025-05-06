use entities_lib::{AudioBook, AudiobookWithData, Author, Category, Keyword, Reader, Series, User};
use leptos::{logging, prelude::*};

use crate::ui_components::audiobook_container::AudioBookComponentBox;

#[server(GetMostRecentAudiobooks, "/api")]
async fn get_recent_audiobooks() -> Result<Vec<AudiobookWithData>, ServerFnError> {
    use shared::graph_trait::get_most_recent_audiobooks_with_data;
    use shared::state::AppState;
    use tracing::info;

    let graph = AppState::get_neo4j_conn()?;
    get_most_recent_audiobooks_with_data(&graph)
        .await
        .map_err(|e| ServerFnError::new(format!("{:?}", e)))
}

#[component]
pub fn HomePage() -> impl IntoView {
    let _user_signal = use_context::<ReadSignal<User>>().unwrap();
    let audiobooks: RwSignal<Option<Vec<AudiobookWithData>>> = RwSignal::new(None);
    let audiobooks_loaded = move || audiobooks.get().is_some();
    let get_audiobooks_op = OnceResource::new(get_recent_audiobooks());
    Effect::new(move || match get_audiobooks_op.get() {
        Some(Ok(data)) => {
            audiobooks.set(Some(data));
        }
        Some(Err(e)) => {
            logging::debug_warn!("{:?}", e);
        }
        None => {
            logging::debug_warn!("get_audiobooks_op is None")
        }
    });
    view! {
        <Show
            when=audiobooks_loaded
            fallback=move|| view! {"loading"} >
            // TODO: audiobook should have a key that is Copy
            <For
                each=move || audiobooks.get().unwrap()
                key=|v| v.0.path.clone()
                let(audiobook_with_data)
            >
               <AudioBookComponentBox audiobook_with_data=audiobook_with_data />
            </For>
        </Show>
    }
}
