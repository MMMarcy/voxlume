use leptos::prelude::*;
use leptos_router::components::A;

#[component]
fn NavButton(
    #[prop(into)] href: Signal<String>,
    #[prop(into)] disabled: Signal<bool>,
    // 1. Tip: Use `into` and `optional` for better flexibility (String is usually better than &'static str for classes)
    #[prop(into, optional)] class_name: String,
    children: Children,
) -> impl IntoView {
    view! {
        <A
            href=move || if disabled.get() { "#".to_string() } else { href.get() }

            // 2. Assign the variable directly here
            attr:class=class_name

            // You can still use reactive classes alongside the static one!
            class:disabled-style=move || disabled.get()

            attr:disabled=move || disabled.get().then_some("")
            style:pointer-events=move || if disabled.get() { "none" } else { "auto" }
            style:opacity=move || if disabled.get() { "0.5" } else { "1" }
        >
            {children()}
        </A>
    }
}

#[component]
fn PageLink(
    #[prop(into)] href: Signal<String>,
    page: u32,
    #[prop(into)] is_current: Signal<bool>,
) -> impl IntoView {
    view! {
        <li>
            <A
                href=href
                attr:class=move || if is_current.get() { "pagination-link is-current" } else { "pagination-link" }
                attr:aria-label=format!("Goto page {}", page)
                attr:aria-current=move || if is_current.get() { "page" } else { "false" }
            >
                {page}
            </A>
        </li>
    }
}

#[component]
fn Ellipsis() -> impl IntoView {
    view! {
        <li><span class="pagination-ellipsis">"..."</span></li>
    }
}

#[component]
fn PaginatorContent(
    #[prop(into)] current_page: Signal<u32>,
    #[prop(into)] total_pages: Signal<u32>,
    #[prop(into)] path: Signal<String>, // Changed from Arc<String> to Signal<String>
) -> impl IntoView {
    let radius = 2;

    // 1. Prepare Signals OUTSIDE the view.
    // We use path.get() so these update when the path signal changes.

    let prev_href = Signal::derive(move || {
        path.get()
            .replace("{}", &(current_page.get() - 1).to_string())
    });

    let next_href = Signal::derive(move || {
        path.get()
            .replace("{}", &(current_page.get() + 1).to_string())
    });

    let first_href = Signal::derive(move || path.get().replace("{}", "1"));

    let last_href =
        Signal::derive(move || path.get().replace("{}", &total_pages.get().to_string()));

    // 2. Logic for logic/disabling
    let prev_disabled = move || current_page.get() <= 1;
    let next_disabled = move || current_page.get() >= total_pages.get();

    // 3. Middle pages logic
    let middle_pages = move || {
        let current = current_page.get();
        let total = total_pages.get();
        let path_str = path.get(); // Get the current path string

        let start = if current > radius + 1 {
            current - radius
        } else {
            2
        };
        let end = if current + radius < total {
            current + radius
        } else {
            total - 1
        };

        (start..=end)
            .filter(move |&p| p > 1 && p < total)
            .map(move |p| {
                // Simple string replace here
                let href = path_str.replace("{}", &p.to_string());
                view! {
                    <PageLink href=href page=p is_current={p == current} />
                }
            })
            .collect_view()
    };

    let (is_mobile, set_is_mobile) = signal(false);

    Effect::new(move || {
        let w = window();
        let Ok(width_val) = w.inner_width() else {
            return;
        };
        if let Some(width) = width_val.as_f64() {
            set_is_mobile.set(width < 769.0); // Bulma's mobile breakpoint is 768px
        }
    });

    view! {
        <nav class="pagination is-centered" role="navigation" aria-label="pagination">
            <NavButton
                href=prev_href
                disabled=Signal::derive(prev_disabled)
                class_name="pagination-previous"
            >
                {move || if is_mobile.get() { "Prev" } else { "Previous" }}
            </NavButton>

            <NavButton
                href=next_href
                disabled=Signal::derive(next_disabled)
                class_name="pagination-next"
            >
                {move || if is_mobile.get() { "Next" } else { "Next page" }}
            </NavButton>

            <Show when=move || !is_mobile.get()>
                <ul class="pagination-list">
                    <PageLink
                        href=first_href
                        page=1
                        is_current=Signal::derive(move || current_page.get() == 1)
                    />

                    <Show when=move || {current_page.get() > radius + 2}>
                        <Ellipsis />
                    </Show>

                    {middle_pages}

                    <Show when=move || {current_page.get() + radius + 1 < total_pages.get()}>
                        <Ellipsis />
                    </Show>

                    <Show when=move || {total_pages.get() > 1}>
                        <PageLink
                            href=last_href
                            page=total_pages.get()
                            is_current=Signal::derive(move || { current_page.get() == total_pages.get() })
                        />
                    </Show>
                </ul>
            </Show>
        </nav>
    }
}

#[component]
pub fn Paginator(
    #[prop(into)] current_page: Signal<u32>,
    #[prop(into)] n_pages: Signal<Option<u32>>,
    #[prop(into)] path: Signal<String>, // Changed from Arc<String>
) -> impl IntoView {
    view! {
        <Show when=move || n_pages.get().is_some_and(|n| n > 0)>
            <PaginatorContent
                current_page=current_page
                total_pages=Signal::derive(move || n_pages.get().unwrap_or(0))
                path=path
            />
        </Show>
    }
}
