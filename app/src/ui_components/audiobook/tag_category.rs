use entities_lib::Category;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn CategoriesTag(categories: Vec<Category>, limit: u8) -> impl IntoView {
    let has_values = !categories.is_empty();
    let limit = limit as usize;
    let has_more_values_than_limit = categories.len() > limit;

    view! {
        <Show when=move || has_values fallback=|| view! { <></> }>
            <div class="tags are-normal">
                {"Categories: "}
                {categories
                    .clone()
                    .into_iter()
                    .take(limit)
                    .map(|category| {
                        view! {
                            <A
                                href=format!("/category/{}/{}/1", category.id, category.value)
                                class:tag=true
                                class:is-info=true
                            >
                                {category.value}
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
