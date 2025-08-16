use crate::ui_components::audiobook_container::AudioBookCollectionContainer;
use entities_lib::{GetAudioBookRequestType, Series};
use leptos::prelude::*;
use leptos::Params;
use leptos_meta::Title;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[derive(Params, PartialEq)]
struct SeriesParam {
    series: Option<String>,
}

#[component]
pub fn BySeriesPage() -> impl IntoView {
    let params = use_params::<SeriesParam>();
    let series = move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|p| p.series.clone())
            .unwrap()
    };
    let section_title = move || format!("Audiobooks of series: {}", series());

    view! {
        <Title text=section_title />
        <div class="section">
             <AudioBookCollectionContainer
                title=Signal::derive(move || format!("Audiobooks of series: {}", series()))
                request_type=Signal::derive(move || GetAudioBookRequestType::BySeries(Series {title: series()}, 0))
            />
        </div>
    }
}
