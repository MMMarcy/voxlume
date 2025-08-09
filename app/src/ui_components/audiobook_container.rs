use entities_lib::{AudiobookWithData, Author, GetAudioBookRequestType, User};
use leptos::{logging, prelude::*};
use leptos_router::components::A;
use web_sys::TouchEvent;

#[server(GetAudiobooks, "/api")]
async fn get_audiobooks(
    request_type: GetAudioBookRequestType,
    maybe_author: Option<Author>,
    page: u16,
) -> Result<Vec<AudiobookWithData>, ServerFnError> {
    use shared::auth_user::AuthSession;
    use shared::graph_trait::get_audiobooks_cached;
    use shared::state::AppState;
    use tracing::info;

    info!("Before getting app state");
    let state = AppState::get_app_state()?;
    let graph = state.graph;
    let cache = state.cache;
    info!("Gotten app state");
    let auth_session =
        use_context::<AuthSession>().ok_or(ServerFnError::new("Couldn't find auth session"))?;
    let user_id = auth_session.current_user.map_or_else(|| 1i64, |v| v.id);
    let limit_audiobooks = if user_id == 1 {
        state.shareable_args.guest_user_audiobooks_per_homepage
    } else {
        state.shareable_args.user_audiobooks_per_homepage_section
    };
    info!("Gotten here.");
    get_audiobooks_cached(
        &graph,
        &cache,
        Some(user_id),
        maybe_author,
        request_type,
        limit_audiobooks,
        page,
    )
    .await
    .map_err(|e| ServerFnError::new(format!("{:?}", e)))
}

#[component]
pub fn AudioBookCollectionContainer(
    title: String,
    request_type: GetAudioBookRequestType,
    maybe_author: Option<Author>,
) -> impl IntoView {
    let _user_signal = use_context::<ReadSignal<User>>().unwrap();

    let audiobooks: RwSignal<Option<Vec<AudiobookWithData>>> = RwSignal::new(None);
    let audiobooks_loaded = move || audiobooks.get().is_some();
    let get_audiobooks_op =
        OnceResource::new_blocking(get_audiobooks(request_type, maybe_author, 0));
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
        if !_user_signal().is_guest() {
            set_is_hover.update(|v| *v = !*v);
        }
    };
    let on_touch_hanlder = move |_: TouchEvent| {
        set_is_hover.update(|v| *v = !*v);
    };
    view! {
        <div class="container is-fluid" style="margin-top: 1rem">
            <p class="title">{title}</p>
            <div
                class="columns" class:is-multiline=move || _user_signal().is_guest()
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
                    <div class="column is-one-sixth is-one-fourth-desktop is-one-third-tablet is-full-mobile">
                        <AudioBookComponentBox audiobook_with_data=audiobook_with_data />
                    </div>
                    </For>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn AuthorLinks(authors: Vec<Author>) -> impl IntoView {
    let author_count = authors.len();
    view! {
        <p class="subtitle is-6">
            {"By "}
            {authors
                .into_iter()
                .enumerate()
                .map(|(index, author)| {
                    let separator = if index < author_count - 1 { ", " } else { "" };
                    view! {
                        <>
                            <A href=format!("/author/{}", author.name)>{author.name}</A>
                            {separator}
                        </>
                    }
                })
                .collect_view()}
        </p>
    }
}

#[component]
pub fn AudioBookComponentBox(audiobook_with_data: AudiobookWithData) -> impl IntoView {
    let (audiobook, authors, _categories, _keywords, _readers, _maybe_series) = audiobook_with_data;
    let title = audiobook.title;
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
            <AuthorLinks authors=authors />
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
