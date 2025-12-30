use crate::pages::get_counts;
use crate::ui_components::ads::grid_ad::GridAd;
use crate::ui_components::audiobook::audiobook_container::AudioBookCollectionContainer;
use crate::ui_components::paginator::Paginator;
use entities_lib::{
    GetAudioBookRequestType, MetaRequest, MetaResponse, Reader, ShareableArgsValues,
    SubscriptionType,
};
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[derive(Params, PartialEq)]
struct ReaderParam {
    reader_id: Option<i64>,
    reader_name: Option<String>,
    page: Option<u32>,
}

impl TryFrom<&ReaderParam> for Reader {
    // ...existing code...
    type Error = ();

    fn try_from(value: &ReaderParam) -> Result<Self, Self::Error> {
        if value.reader_id.is_none() || value.reader_name.is_none() {
            return Err(());
        }
        Ok(Reader {
            id: value.reader_id.unwrap(),
            name: value.reader_name.clone().unwrap(),
        })
    }
}

#[component]
pub fn ReaderPage() -> impl IntoView {
    let params = use_params::<ReaderParam>();
    let shareable_args = use_context::<ReadSignal<Option<ShareableArgsValues>>>();
    let (get_amount, set_amount) = signal::<Option<u32>>(None);

    let reader_data = Memo::new(move |_| {
        params.read().as_ref().ok().map_or_else(
            || {
                (
                    Reader {
                        id: 1,
                        name: "unknown".to_string(),
                    },
                    0,
                )
            },
            |p| (Reader::try_from(p).ok().unwrap(), p.page.unwrap_or(0)),
        )
    });

    let reader_entity = move || reader_data.get().0;
    let current_page = move || reader_data.get().1;

    let section_title =
        Signal::derive(move || format!("Audiobooks read by {}", reader_entity().name));

    let get_count_op = Resource::new(reader_entity, move |entity| {
        get_counts(MetaRequest::CountAudiobooksForReader(entity))
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
        let entity = reader_entity();
        format!("/reader/{}/{}/{{}}", entity.id, entity.name)
    });

    view! {
        <div class="container is-max-desktop is-flex is-justify-content-center">
            <GridAd ad_slot="1117011249" />
        </div>
        <Paginator current_page=Signal::derive(current_page) n_pages=get_amount path=path/>
            <AudioBookCollectionContainer
                title=section_title
                request_type=Signal::derive(move || GetAudioBookRequestType::ByReader(reader_entity(), current_page()))
                subscription_type=Some(Signal::derive(move || {
                    SubscriptionType::ToReader(reader_entity())
                }))
            />
        <Paginator current_page=Signal::derive(current_page) n_pages=get_amount path=path/>
    }
}
