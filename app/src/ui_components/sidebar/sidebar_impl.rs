use entities_lib::entities::meta_request::{MetaRequest, MetaResponse};
use leptos::prelude::*;
use leptos_router::components::A;

/// Fetches metadata from the server based on the provided request.
///
/// This function acts as a bridge to the backend, retrieving lists of categories,
/// authors, series, or readers. It leverages caching via `get_meta_cached`.
#[server(GetMetadata, "/api")]
pub async fn get_metadata(request: MetaRequest) -> Result<MetaResponse, ServerFnError> {
    use shared::db_ops::parade::meta_ops::get_meta_cached;
    use shared::state::AppState;

    let state = AppState::get_app_state()?;
    let pgpool = state.database_connection_pool;
    let cache = state.meta_requests_cache;

    get_meta_cached(&pgpool, &cache, request)
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))
}

/// Defines the sorting order for the items in a sidebar section.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SortOrder {
    /// Sort items alphabetically (A-Z).
    Alphabetical,
    /// Sort items by the count of associated audiobooks (High-Low).
    ByCount,
}

/// Handles the rendering of the list content, including filtering and pagination.
#[component]
fn SidebarListContent<MapF>(
    /// The response from the server containing metadata.
    response: Result<MetaResponse, ServerFnError>,
    /// Function to map the server response to displayable items.
    mapper: MapF,
    /// The current search query for filtering items.
    #[prop(into)]
    search_query: Signal<String>,
    /// The current page number for pagination.
    #[prop(into)]
    current_page: Signal<usize>,
    /// Setter for the current page number.
    #[prop(into)]
    set_current_page: WriteSignal<usize>,
    /// Callback triggered when a link in the list is clicked.
    #[prop(into)]
    on_link_click: Callback<()>,
) -> impl IntoView
where
    MapF: Fn(MetaResponse) -> Vec<(String, String)> + Copy + Send + 'static, {
    let page_size = 15;

    match response {
        Ok(data) => {
            let items = mapper(data);

            // Memoize the filtering to track search_query changes.
            // This moves `items` into the closure, owning it.
            let filtered_items_memo = Memo::new(move |_| {
                let query = search_query.get().to_lowercase();
                if query.is_empty() {
                    items.clone()
                } else {
                    items
                        .iter()
                        .filter(|(_, label)| label.to_lowercase().contains(&query))
                        .cloned()
                        .collect()
                }
            });

            view! {
                {move || {
                    // Use .with() to avoid cloning the entire vector
                    filtered_items_memo
                        .with(|filtered_items| {
                            let total_items = filtered_items.len();
                            let max_page = if total_items > 0 {
                                (total_items - 1) / page_size
                            } else {
                                0
                            };
                            let page = usize::min(current_page.get(), max_page);
                            let start = page * page_size;
                            let end = usize::min(start + page_size, total_items);

                            // Clone only the items for the current page
                            let page_items = if start < total_items {
                                filtered_items[start..end].to_vec()
                            } else {
                                vec![]
                            };

                            view! {
                                <For
                                    each=move || page_items.clone()
                                    key=|item| item.0.clone()
                                    let(child)
                                >
                                    <li>
                                        <A href=child.0 on:click=move |_| on_link_click.run(())>
                                            {child.1}
                                        </A>
                                    </li>
                                </For>
                                <Show when=move || { total_items > page_size }>
                                    <div class="level is-mobile mt-2 is-size-7">
                                        <div class="level-left">
                                            <button
                                                class="button is-small is-ghost p-1"
                                                disabled=move || page == 0
                                                on:click=move |_| set_current_page
                                                    .update(|p| *p = p.saturating_sub(1))
                                            >
                                                <span class="icon">
                                                    <i class="fas fa-chevron-left"></i>
                                                </span>
                                            </button>
                                        </div>
                                        <div class="level-item">
                                            <span>
                                                {format!("{}/{}", page + 1, max_page + 1)}
                                            </span>
                                        </div>
                                        <div class="level-right">
                                            <button
                                                class="button is-small is-ghost p-1"
                                                disabled=move || { page >= max_page }
                                                on:click=move |_| {
                                                    set_current_page.update(|p| *p += 1);
                                                }
                                            >
                                                <span class="icon">
                                                    <i class="fas fa-chevron-right"></i>
                                                </span>
                                            </button>
                                        </div>
                                    </div>
                                </Show>
                            }
                        })
                }}
            }
            .into_any()
        }
        Err(_e) => view! {
            <li class="has-text-danger is-size-7">"Error"</li>
        }
        .into_any(),
    }
}

/// A generic component that renders a collapsible, searchable, and paginated list section in the sidebar.
///
/// This component handles:
/// - Fetching data using the provided `req_factory`.
/// - Mapping the response to a list of (URL, Label) tuples using `mapper`.
/// - Client-side filtering based on a search query.
/// - Client-side pagination of the filtered results.
/// - Toggling sort order and section visibility.
///
/// # Type Parameters
/// * `ReqF` - A factory function that creates a `MetaRequest` based on the current `SortOrder`.
/// * `MapF` - A function that maps the `MetaResponse` to a vector of `(String, String)` tuples (URL, Label).
#[component]
fn SidebarListSection<ReqF, MapF>(
    /// The title displayed in the section header.
    title: &'static str,
    /// Function to generate the data fetching request.
    req_factory: ReqF,
    /// Function to map the server response to displayable items.
    mapper: MapF,
    /// Callback triggered when a link in the list is clicked (e.g., to close the sidebar on mobile).
    #[prop(into)]
    on_link_click: Callback<()>,
) -> impl IntoView
where
    ReqF: Fn(SortOrder) -> MetaRequest + Copy + Send + Sync + 'static,
    MapF: Fn(MetaResponse) -> Vec<(String, String)> + Copy + Send + 'static, {
    // -- State --
    let (sort_order, set_sort_order) = signal(SortOrder::Alphabetical);
    let (is_collapsed, set_is_collapsed) = signal(false);

    // Search and Pagination State
    let (is_search_visible, set_is_search_visible) = signal(false);
    let (search_query, set_search_query) = signal(String::new());
    let (current_page, set_current_page) = signal(0);

    // -- Data Fetching --
    // Refetches data whenever `sort_order` changes.
    let resource = Resource::new(
        move || sort_order.get(),
        move |order| {
            let req = req_factory(order);
            get_metadata(req)
        },
    );

    // -- Handlers --

    // Toggles between Alphabetical and ByCount sorting.
    let toggle_sort = move |_| {
        set_sort_order.update(|s| {
            *s = match s {
                SortOrder::Alphabetical => SortOrder::ByCount,
                SortOrder::ByCount => SortOrder::Alphabetical,
            }
        });
    };

    // Reset pagination when search query or sort order changes to avoid "empty" pages.
    Effect::new(move |_| {
        search_query.track();
        sort_order.track();
        set_current_page.set(0);
    });

    view! {
        <div class="mb-4">
            // -- Section Header --
            <div class="level is-mobile mb-2">
                <div class="level-left">
                    <p class="menu-label mb-0">{title}</p>
                </div>
                // Buttons container: Search, Sort, Collapse
                <div class="is-flex">
                    <button
                        class="button is-small is-ghost p-1"
                        on:click=move |_| {
                            set_is_search_visible.update(|v| *v = !*v);
                            if !is_search_visible.get() {
                                set_search_query.set(String::new());
                            }
                        }
                        title="Search"
                    >
                        <span class="icon is-small">
                            <i class="fas fa-magnifying-glass"></i>
                        </span>
                    </button>
                    <button
                        class="button is-small is-ghost p-1"
                        on:click=toggle_sort
                        title="Toggle Sort Order"
                    >
                        <span class="icon is-small">
                            <i class=move || {
                                match sort_order.get() {
                                    SortOrder::Alphabetical => "fas fa-arrow-down-a-z",
                                    SortOrder::ByCount => "fas fa-arrow-down-wide-short",
                                }
                            }></i>
                        </span>
                    </button>
                    <button
                        class="button is-small is-ghost p-1"
                        on:click=move |_| set_is_collapsed.update(|c| *c = !*c)
                        title="Toggle Visibility"
                    >
                        <span class="icon is-small">
                            <i class=move || {
                                if is_collapsed.get() {
                                    "fas fa-chevron-right"
                                } else {
                                    "fas fa-chevron-down"
                                }
                            }></i>
                        </span>
                    </button>
                </div>
            </div>

            // -- Search Input --
            <Show when=move || is_search_visible.get()>
                <div class="field mb-2 mx-1">
                    <div class="control has-icons-left">
                        <input
                            class="input is-small"
                            type="text"
                            placeholder="Filter..."
                            prop:value=move || search_query.get()
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        />
                        <span class="icon is-small is-left">
                            <i class="fas fa-filter"></i>
                        </span>
                    </div>
                </div>
            </Show>

            // -- List Items --
            <ul class="menu-list" class:is-hidden=move || is_collapsed.get()>
                <Transition fallback=move || {
                    view! { <li class="is-size-7 has-text-grey">"Loading..."</li> }
                }>
                    {move || {
                        resource
                            .get()
                            .map(|response| {
                                view! {
                                    <SidebarListContent
                                        response=response
                                        mapper=mapper
                                        search_query=search_query
                                        current_page=current_page
                                        set_current_page=set_current_page
                                        on_link_click=on_link_click
                                    />
                                }
                            })
                    }}

                </Transition>
            </ul>
        </div>
    }
}

/// The main sidebar component for the application's "Explorer" view.
///
/// It renders a collapsible sidebar menu containing sections for:
/// - Categories
/// - Authors
/// - Series
/// - Readers
///
/// Handles automatic collapsing on small screens (mobile/tablet) when a link is clicked.
#[allow(clippy::too_many_lines)]
#[component]
pub fn ExplorerSidebar(
    /// Controls the visibility of the sidebar.
    #[prop(into)]
    is_open: Signal<bool>,
    /// Callback to toggle the `is_open` state.
    #[prop(into)]
    toggle: Callback<()>,
) -> impl IntoView {
    // Helper to create default page/limit.
    // We set a very high limit to fetch "all" items, relying on client-side pagination.
    let default_pl = || (0, 100_000);

    let handle_nav_click = move || {
        let win_width = window().inner_width().ok().and_then(|w| w.as_f64());

        if let Some(w) = win_width {
            // Collapse if screen is smaller than 1024px (Mobile/Tablet)
            if w < 1024.0 {
                toggle.run(());
            }
        }
    };

    view! {
        <aside class="menu">
            <div class="level is-mobile mb-4">
                <div class="level-left">
                    <Show when=move || is_open.get()>
                        <p class="menu-label is-size-6 has-text-weight-bold mb-0">"Explorer"</p>
                    </Show>

                    <Show when=move || !is_open.get()>
                        <button
                            class="button is-small p-1 sidebar-toggle-button mt-3"
                            on:click=move |_| toggle.run(())
                            title="Expand Sidebar"
                        >
                            <span class="icon">
                                <i class="fas fa-bars"></i>
                            </span>
                        </button>
                    </Show>
                </div>
                <div class="level-right">
                    <Show when=move || is_open.get()>
                        <button
                            class="button is-small p-1 sidebar-toggle-button mt-2"
                            on:click=move |_| toggle.run(())
                            title="Collapse Sidebar"
                        >
                            <span class="icon">
                                <i class="fas fa-circle-arrow-left"></i>
                            </span>
                        </button>
                    </Show>
                </div>
            </div>

            <div class:is-hidden=move || !is_open.get()>
                <SidebarListSection
                    title="Categories"
                    req_factory=move |sort| {
                        let (p, l) = default_pl();
                        match sort {
                            SortOrder::Alphabetical => MetaRequest::CategoriesAlphabetically(p, l),
                            SortOrder::ByCount => MetaRequest::CategoriesByPublishedAudiobooks(p, l),
                        }
                    }
                    mapper=move |res| {
                        if let MetaResponse::Categories(list) = res {
                            list.into_iter().map(|c| (format!("/category/{}/{}/1", c.id, c.value), c.value)).collect()
                        } else { vec![] }
                    }
                    on_link_click=handle_nav_click
                />

                <SidebarListSection
                    title="Authors"
                    req_factory=move |sort| {
                        let (p, l) = default_pl();
                        match sort {
                            SortOrder::Alphabetical => MetaRequest::AuthorsAlphabetically(p, l),
                            SortOrder::ByCount => MetaRequest::AuthorsByPublishedAudiobooks(p, l),
                        }
                    }
                    mapper=move |res| {
                        if let MetaResponse::Authors(list) = res {
                            list.into_iter().map(|a| (format!("/author/{}/{}/1", a.id, a.name), a.name)).collect()
                        } else { vec![] }
                    }
                    on_link_click=handle_nav_click
                />

                <SidebarListSection
                    title="Series"
                    req_factory=move |sort| {
                        let (p, l) = default_pl();
                        match sort {
                            SortOrder::Alphabetical => MetaRequest::SeriesAlphabetically(p, l),
                            SortOrder::ByCount => MetaRequest::SeriesBySubscriber(p, l),
                        }
                    }
                    mapper=move |res| {
                        if let MetaResponse::Series(list) = res {
                            list.into_iter().map(|s| (format!("/series/{}/{}/1", s.id, s.title), s.title)).collect()
                        } else { vec![] }
                    }
                    on_link_click=handle_nav_click
                />

                <SidebarListSection
                    title="Readers"
                    req_factory=move |sort| {
                        let (p, l) = default_pl();
                        match sort {
                            SortOrder::Alphabetical => MetaRequest::ReadersAlphabetically(p, l),
                            SortOrder::ByCount => MetaRequest::ReaderByPublishedAudiobooks(p, l),
                        }
                    }
                    mapper=move |res| {
                        if let MetaResponse::Readers(list) = res {
                            list.into_iter().map(|r| (format!("/reader/{}/{}/1", r.id, r.name), r.name)).collect()
                        } else { vec![] }
                    }
                    on_link_click=handle_nav_click
                />
            </div>
        </aside>
    }
}
