use crate::pages::get_counts;
use crate::ui_components::ads::grid_ad::GridAd;
use crate::ui_components::audiobook::audiobook_container::AudioBookCollectionContainer;
use crate::ui_components::paginator::Paginator;
use entities_lib::{
    GetAudioBookRequestType, Keyword, MetaRequest, MetaResponse, ShareableArgsValues,
};
use leptos::Params;
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[derive(Params, PartialEq)]
struct KeywordParam {
    keyword_id: Option<i64>,
    keyword_name: Option<String>,
    page: Option<u32>,
}

impl TryFrom<&KeywordParam> for Keyword {
    type Error = ();

    fn try_from(value: &KeywordParam) -> Result<Self, Self::Error> {
        if value.keyword_id.is_none() || value.keyword_name.is_none() {
            return Err(());
        }
        Ok(Keyword {
            id: value.keyword_id.unwrap(),
            value: value.keyword_name.clone().unwrap(),
        })
    }
}

#[component]
pub fn KeywordPage() -> impl IntoView {
    let params = use_params::<KeywordParam>();
    let shareable_args = use_context::<ReadSignal<Option<ShareableArgsValues>>>();
    let (get_amount, set_amount) = signal::<Option<u32>>(None);

    let keyword_data = Memo::new(move |_| {
        params.read().as_ref().ok().map_or_else(
            || {
                (
                    Keyword {
                        id: 1,
                        value: "unknown".to_string(),
                    },
                    0,
                )
            },
            |p| (Keyword::try_from(p).ok().unwrap(), p.page.unwrap_or(0)),
        )
    });

    let keyword_entity = move || keyword_data.get().0;
    let current_page = move || keyword_data.get().1;

    let section_title =
        Signal::derive(move || format!("Audiobooks with keyword {}", keyword_entity().value));

    let get_count_op = Resource::new(keyword_entity, move |entity| {
        get_counts(MetaRequest::CountAudiobooksForKeyword(entity))
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
        let entity = keyword_entity();
        format!("/keyword/{}/{}/{{}}", entity.id, entity.value)
    });

    view! {
        <div class="container is-max-desktop is-flex is-justify-content-center">
            <GridAd ad_slot="1117011249" />
        </div>
        <Paginator current_page=Signal::derive(current_page) n_pages=get_amount path=path/>
            <AudioBookCollectionContainer
                title=section_title
                request_type=Signal::derive(move || GetAudioBookRequestType::ByKeyword(
                    keyword_entity(),
                    current_page(),
                ))
                subscription_type=None
            />
        <Paginator current_page=Signal::derive(current_page) n_pages=get_amount path=path/>
    }
}
