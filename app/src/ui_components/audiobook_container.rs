use entities_lib::{AudioBook, AudiobookWithData, Author, Category, Keyword, Reader, Series};
use leptos::prelude::*;

#[component]
pub fn AudioBookComponentBox(audiobook_with_data: AudiobookWithData) -> impl IntoView {
    view! {
        <p>{ audiobook_with_data.0.title }</p>
        <p>{ audiobook_with_data.1.first().unwrap().name.clone() }</p>
    }
}
