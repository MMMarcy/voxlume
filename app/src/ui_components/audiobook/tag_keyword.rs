use entities_lib::Keyword;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn KeywordsTag(keywords: Vec<Keyword>, limit: u8) -> impl IntoView {
    let has_values = !keywords.is_empty();
    let limit = limit as usize;
    let has_more_values_than_limit = keywords.len() > limit;

    view! {
        <Show when=move || has_values fallback=|| view! { <></> }>
            <div class="tags are-normal">
                {"Keywords: "}
                {keywords
                    .clone()
                    .into_iter()
                    .take(limit)
                    .map(|keyword| {
                        view! {
                            <A
                                href=format!("/keyword/{}/{}/1", keyword.id, keyword.value)
                                class:tag=true
                                class:is-white=true
                            >
                                {keyword.value}
                            </A>
                        }
                    })
                    .collect_view()}
                <Show when=move || has_more_values_than_limit >
                    "..."
                </Show>
            </div>
        </Show>
    }
}
