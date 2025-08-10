use entities_lib::{
    AudiobookWithData, Author, Category, GetAudioBookRequestType, Keyword, Reader, User,
};
use leptos::{logging, prelude::*};
use leptos_router::components::A;
use web_sys::TouchEvent;

#[server(GetAudiobooks, "/api")]
async fn get_audiobooks(
    request_type: GetAudioBookRequestType,
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
) -> impl IntoView {
    let _user_signal = use_context::<ReadSignal<User>>().unwrap();

    let audiobooks: RwSignal<Option<Vec<AudiobookWithData>>> = RwSignal::new(None);
    let audiobooks_loaded = move || audiobooks.get().is_some();

    let get_audiobooks_resource = Resource::new(
        move || request_type.clone(),
        move |current_request_type| get_audiobooks(current_request_type, 0),
    );

    Effect::new(move || {
        let result = get_audiobooks_resource.get();
        match result {
            Some(Ok(data)) => {
                logging::log!("{}", "Some(Ok(data))");
                audiobooks.set(Some(data));
            }
            Some(Err(e)) => {
                logging::debug_warn!("{:?}", e);
                audiobooks.set(None);
            }
            None => {
                audiobooks.set(None);
            }
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
                        each=move || audiobooks().unwrap()
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
fn ReaderLinks(readers: Vec<Reader>) -> impl IntoView {
    let reader_count = readers.len();
    view! {
        <p class="subtitle is-6">
            {"Read by "}
            {readers
                .into_iter()
                .enumerate()
                .map(|(index, reader)| {
                    let separator = if index < reader_count - 1 { ", " } else { "" };
                    view! {
                        <>
                            <A href=format!("/reader/{}", reader.name)>{reader.name}</A>
                            {separator}
                        </>
                    }
                })
                .collect_view()}
        </p>
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
fn KeywordsTag(keywords: Vec<Keyword>) -> impl IntoView {
    view! {
        <div class="tags are-normal">
            {"Keywords: "}
            {keywords
                .into_iter()
                .map(|keyword| {
                    view! {
                        <A href=format!("/keyword/{}", keyword.value) class:tag=true class:is-light=true >{keyword.value}</A>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
fn CategoriesTag(categories: Vec<Category>) -> impl IntoView {
    view! {
        <div class="tags are-normal">
            {"Categories: "}
            {categories
                .into_iter()
                .map(|category| {
                    view! {
                        <A href=format!("/category/{}", category.value) class:tag=true class:is-info=true >{category.value}</A>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
pub fn AudioBookComponentBox(audiobook_with_data: AudiobookWithData) -> impl IntoView {
    let (audiobook, authors, categories, keywords, readers, _maybe_series) = audiobook_with_data;
    let title = audiobook.title;
    let cover_url = audiobook.cover_url;
    let very_short_description = audiobook.very_short_description;

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
            <ReaderLinks readers=readers />
        </div>
        </div>

        <div class="content">
           { very_short_description }
        <br />
        <br />
        <CategoriesTag categories=categories />
        <br />
        <KeywordsTag keywords=keywords />
        </div>
    </div>
    </div>
    }
}
