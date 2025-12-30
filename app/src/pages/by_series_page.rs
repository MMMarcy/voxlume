use crate::pages::get_counts;
use crate::ui_components::ads::grid_ad::GridAd;
use crate::ui_components::audiobook::audiobook_container::AudioBookCollectionContainer;
use crate::ui_components::paginator::Paginator;
use entities_lib::{
    GetAudioBookRequestType, MetaRequest, MetaResponse, Series, ShareableArgsValues,
    SubscriptionType,
};
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
// You don't need to import Params twice
// use leptos_router::params::Params;

#[derive(Params, PartialEq, Clone)]
struct SeriesParam {
    series_id: Option<i64>,
    series_title: Option<String>,
    page: Option<u32>,
}

impl TryFrom<&SeriesParam> for Series {
    // ...existing code...
    type Error = ();

    fn try_from(value: &SeriesParam) -> Result<Self, Self::Error> {
        // Use a match or if-let for cleaner parsing
        if let (Some(id), Some(title)) = (value.series_id, value.series_title.as_ref()) {
            Ok(Series {
                id,
                title: title.clone(),
            })
        } else {
            Err(())
        }
    }
}

#[component]
pub fn BySeriesPage() -> impl IntoView {
    let params = use_params::<SeriesParam>();
    let shareable_args = use_context::<ReadSignal<Option<ShareableArgsValues>>>();
    let (get_amount, set_amount) = signal::<Option<u32>>(None);

    let series_data = Memo::new(move |_| {
        params.with(|p| {
            p.as_ref().ok().map_or_else(
                || {
                    (
                        Series {
                            id: 1,
                            title: "unknown".to_string(),
                        },
                        0,
                    )
                },
                |p_struct| {
                    (
                        Series::try_from(p_struct).ok().unwrap(),
                        p_struct.page.unwrap_or(0),
                    )
                },
            )
        })
    });

    let series_entity = move || series_data.get().0;
    let current_page = move || series_data.get().1;

    let section_title = move || format!("Audiobooks of series: {}", series_entity().title);

    let get_count_op = Resource::new(series_entity, move |entity| {
        get_counts(MetaRequest::CountAudiobooksInSeries(entity))
    });

    Effect::new(move || {
        let result = get_count_op.get();

        if let Some(Ok(MetaResponse::Count(amount))) = result
            && let Some(Some(args)) = shareable_args.get()
        {
            let pages = amount / args.guest_user_audiobooks_per_homepage;
            set_amount(Some(pages));
        }
    });

    let path = Memo::new(move |_| {
        let entity = series_entity();
        format!("/series/{}/{}/{{}}", entity.id, entity.title)
    });

    view! {
        <div class="container is-max-desktop is-flex is-justify-content-center">
            <GridAd ad_slot="1117011249" />
        </div>
        <Paginator current_page=Signal::derive(current_page) n_pages=get_amount path=path/>
        <AudioBookCollectionContainer
            title=Signal::derive(section_title)
            request_type=Signal::derive(move || GetAudioBookRequestType::BySeries(series_entity(), current_page()))
            subscription_type=Some(Signal::derive(move || SubscriptionType::ToSeries(series_entity())))
        />
        <Paginator current_page=Signal::derive(current_page) n_pages=get_amount path=path/>
    }
}
