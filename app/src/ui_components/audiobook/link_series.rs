use entities_lib::Series;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn SeriesLink(maybe_series: Option<Series>) -> impl IntoView {
    maybe_series.map(|series| {
        view! {
            {"Part of \""}
            <A href=format!("/series/{}/{}/1", series.id, series.title)>{series.title.clone()}</A>
            {"\" series"}
        }
    })
}
