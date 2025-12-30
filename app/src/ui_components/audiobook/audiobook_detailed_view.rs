use entities_lib::{AudiobookWithData, ShareableArgsValues};
use leptos::prelude::*;

use crate::ui_components::audiobook::link_author::AuthorLinks;
use crate::ui_components::audiobook::link_reader::ReaderLinks;
use crate::ui_components::audiobook::link_series::SeriesLink;
use crate::ui_components::audiobook::tag_category::CategoriesTag;
use crate::ui_components::audiobook::tag_keyword::KeywordsTag;

#[component]
pub fn AudioBookDetailedViewComponent(audiobook_with_data: AudiobookWithData) -> impl IntoView {
    let (audiobook, authors, categories, keywords, readers, maybe_series) = audiobook_with_data;
    let title = audiobook.title;
    let cover_url = audiobook.cover_url;
    let description = audiobook.description;
    let path = audiobook.path;

    let (img_src, set_img_src) = signal(cover_url);

    let shared_args = use_context::<ReadSignal<Option<ShareableArgsValues>>>()
        .expect("Shared args signal should be provided");

    let mirrors = move || {
        let args_opt = shared_args.get();
        if let Some(args) = args_opt {
            let path_suffix = if let Some(idx) = path.find("://") {
                if let Some(slash_idx) = path[idx + 3..].find('/') {
                    &path[idx + 3 + slash_idx..]
                } else {
                    "/"
                }
            } else {
                path.as_str()
            };

            args.audiobookbay_extensions
                .iter()
                .enumerate()
                .map(|(i, ext)| {
                    let label = format!("Mirror {}", i + 1);
                    let url = format!(
                        "https://{}.{}{}",
                        args.audiobookbay_domain, ext, path_suffix
                    );
                    let class = if i == 0 {
                        "button is-primary"
                    } else {
                        "button is-light"
                    };
                    (label, url, class)
                })
                .collect::<Vec<_>>()
        } else {
            vec![("Mirror 1".to_string(), path.clone(), "button is-primary")]
        }
    };

    // Define your fallback image (local path or external URL)
    let fallback_image = "https://placehold.co/400x400?text=Cover+not+found";
    view! {
        <section class="section">
            <div class="container">
                // Added is-multiline to allow wrapping
                <div class="columns is-vcentered is-variable is-8 is-multiline">

                    // Cover Image Column
                    // is-full-mobile: Forces full width on mobile
                    <div class="column is-full-mobile is-one-third-desktop is-4-tablet">
                        <figure class="image" style="box-shadow: 0 0.5em 1em -0.125em rgba(10, 10, 10, 0.1), 0 0px 0 1px rgba(10, 10, 10, 0.02); border-radius: 8px; overflow: hidden;">
                            <img
                                src=img_src
                                alt=format!("Cover for {}", title)
                                on:error=move |_| set_img_src.set(Some(fallback_image.to_string()))
                                style="object-fit: cover;" />
                        </figure>
                    </div>

                    // Details Column
                    // is-full-mobile: Forces full width on mobile
                    <div class="column is-full-mobile">
                        <h1 class="title is-2 is-size-3-mobile">{title}</h1>

                        <div class="mb-4">
                            <AuthorLinks authors=authors />
                            <ReaderLinks readers=readers limit=3 />
                            <SeriesLink maybe_series=maybe_series />
                        </div>

                        <div class="content my-5" inner_html=description />

                        <div class="mb-4">
                            <CategoriesTag categories=categories limit=10 />
                        </div>
                        <div class="mb-5">
                            <KeywordsTag keywords=keywords limit=10 />
                        </div>

                        <div class="buttons">
                             <For
                                each=mirrors
                                key=|item| item.1.clone()
                                children=move |(label, url, class_name)| {
                                    view! {
                                        <a href=url target="_blank" class=class_name>
                                            <strong>{label}</strong>
                                        </a>
                                    }
                                }
                            />
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}