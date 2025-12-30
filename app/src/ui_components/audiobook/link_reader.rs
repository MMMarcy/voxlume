use entities_lib::Reader;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn ReaderLinks(readers: Vec<Reader>, limit: u8) -> impl IntoView {
    let reader_count = readers.len();
    let has_values = !readers.is_empty();
    let limit = limit as usize;
    let has_more_values_than_limit = reader_count > limit;
    let displayed_count = reader_count.min(limit);

    view! {
        <Show when=move || has_values fallback=|| view! { "Unknow reader" }>
            <p class="subtitle is-6">
                {"Read by "}
                {readers.clone()
                    .into_iter()
                    .take(limit)
                    .enumerate()
                    .map(|(index, reader)| {
                        let separator = if index < displayed_count - 1 { ", " } else { "" };
                        view! {
                            <>
                                <A href=format!(
                                    "/reader/{}/{}/1",
                                    reader.id,
                                    reader.name,
                                )>{reader.name}</A>
                                {separator}
                            </>
                        }
                    })
                    .collect_view()}
                <Show when=move || has_more_values_than_limit >
                    " ..."
                </Show>
            </p>
        </Show>
    }
}
