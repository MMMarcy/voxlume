use std::{rc::Rc, sync::Arc};

use entities_lib::{
    entities::{audiobook, author},
    AudioBook, AudiobookWithData, Author, Category, Keyword, Reader, Series,
};
use leptos::{logging, prelude::*};
use leptos::{logging::debug_warn, prelude::*};
use web_sys::TouchEvent;

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
pub fn AudioBookCollectionContainer(title: String) -> impl IntoView {
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
    let (is_hover, set_is_hover) = signal(false);
    let on_hover_handler = move |_| {
        set_is_hover.update(|v| *v = !*v);
    };
    let on_touch_hanlder = move |e: TouchEvent| {
        // e.prevent_default();
        set_is_hover.update(|v| *v = !*v);
    };
    view! {
        <div class="container is-fluid" style="margin-top: 1rem">
            <div
                class="columns"
                class:overlow_x_visible=is_hover
                on:mouseenter=on_hover_handler
                on:mouseleave=on_hover_handler
                on:touchstart=on_touch_hanlder
                // on:touchend=on_touch_hanlder
                on:touchcancel=on_touch_hanlder
            >
                <Show
                    when=audiobooks_loaded
                    fallback=move|| view! {"loading"} >
                    // TODO: audiobook should have a key that is Copy
                    <For
                        each=move || audiobooks.get().unwrap()
                        key=|v| v.0.path.clone()
                        let(audiobook_with_data)
                    >
                    <div class="column is-one-fifth-fullhd is-one-fourth-desktop is-one-third-tablet is-full-mobile">
                        <AudioBookComponentBox audiobook_with_data=audiobook_with_data />
                    </div>
                    </For>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn AudioBookComponentBox(audiobook_with_data: AudiobookWithData) -> impl IntoView {
    let (audiobook, authors, categories, keywords, readers, maybe_series) = audiobook_with_data;
    let title = audiobook.title;
    let author = authors[0].name.clone();
    let cover_url = audiobook.cover_url;
    // let description = audiobook.description;
    let very_short_description = audiobook.very_short_description;

    // let audiobook = Arc::new(audiobook);
    view! {
    <div class="card is-equal-height-card">
    <div class="card-image">
        <figure class="image is-4by3">
        <img
            src={ cover_url }
            alt="Placeholder image"
        />
        </figure>
    </div>
    <div class="card-content">
        <div class="media">
        <div class="media-content">
            <p class="title is-4">{ title }</p>
            <p class="subtitle is-6">By { author }</p>
        </div>
        </div>

        <div class="content">
           { very_short_description }
        <br />
        // <time datetime="2016-1-1">11:09 PM - 1 Jan 2016</time>
        </div>
    </div>
    </div>
    }
}
