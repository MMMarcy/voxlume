use entities_lib::Author;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn AuthorLinks(authors: Vec<Author>) -> impl IntoView {
    let author_count = authors.len();
    let has_values = !authors.is_empty();
    view! {
        <Show when=move || has_values fallback=|| view! { "Unknown authors" }>
            <p class="subtitle is-6">
                {"By "}
                {authors
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(|(index, author)| {
                        let separator = if index < author_count - 1 { ", " } else { "" };
                        view! {
                            <>
                                <A href=format!(
                                    "/author/{}/{}/1",
                                    author.id,
                                    author.name,
                                )>{author.name}</A>
                                {separator}
                            </>
                        }
                    })
                    .collect_view()}
            </p>
        </Show>
    }
}
