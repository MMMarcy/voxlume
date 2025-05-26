use std::{rc::Rc, sync::Arc};

use entities_lib::{
    entities::{audiobook, author},
    AudioBook, AudiobookWithData, Author, Category, Keyword, Reader, Series,
};
use leptos::{logging::debug_warn, prelude::*};

#[component]
pub fn AudioBookComponentBox(audiobook_with_data: AudiobookWithData) -> impl IntoView {
    let (audiobook, authors, categories, keywords, readers, maybe_series) = audiobook_with_data;
    let title = audiobook.title;
    let author = authors[0].name.clone();
    let cover_url = audiobook.cover_url;
    // let description = audiobook.description;
    let very_short_description = audiobook.very_short_description;

    // let audiobook = Arc::new(audiobook);
    debug_warn!("Test update");
    view! {
    <div class="cell">
        <div class="card">
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
            <time datetime="2016-1-1">11:09 PM - 1 Jan 2016</time>
            </div>
        </div>
        </div>
    </div>
        }
}
