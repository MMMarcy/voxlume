use crate::pages::get_counts;
use crate::ui_components::ads::grid_ad::GridAd;
use crate::ui_components::audiobook::audiobook_container::AudioBookCollectionContainer;
use crate::ui_components::paginator::Paginator;
use entities_lib::{
    Author, GetAudioBookRequestType, MetaRequest, MetaResponse, ShareableArgsValues,
    SubscriptionType,
};
use leptos::Params;
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[derive(Params, PartialEq)]
struct AuthorParam {
    author_id: Option<i64>,
    author_name: Option<String>,
    page: Option<u32>,
}

impl TryFrom<&AuthorParam> for Author {
    type Error = ();

    fn try_from(value: &AuthorParam) -> Result<Self, Self::Error> {
        if value.author_id.is_none() || value.author_name.is_none() {
            return Err(());
        }
        Ok(Author {
            id: value.author_id.unwrap(),
            name: value.author_name.clone().unwrap(),
        })
    }
}

#[component]
pub fn AuthorPage() -> impl IntoView {
    let params = use_params::<AuthorParam>();
    let shareable_args = use_context::<ReadSignal<Option<ShareableArgsValues>>>();
    let (get_amount, set_amount) = signal::<Option<u32>>(None);

    let author_data = Memo::new(move |_| {
        params.read().as_ref().ok().map_or_else(
            || {
                (
                    Author {
                        id: 1,
                        name: "unknown".to_string(),
                    },
                    0,
                )
            },
            |p| (Author::try_from(p).ok().unwrap(), p.page.unwrap_or(0)),
        )
    });

    let author_entity = move || author_data.get().0;
    let current_page = move || author_data.get().1;

    let section_title = Signal::derive(move || format!("Audiobooks by {}", author_entity().name));

    let get_count_op = Resource::new(author_entity, move |entity| {
        get_counts(MetaRequest::CountAudiobooksForAuthor(entity))
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
        let entity = author_entity();
        format!("/author/{}/{}/{{}}", entity.id, entity.name)
    });

    view! {
        <div class="container is-max-desktop is-flex is-justify-content-center">
            <GridAd ad_slot="1117011249" />
        </div>
        <Paginator current_page=Signal::derive(current_page) n_pages=get_amount path=path/>
            <AudioBookCollectionContainer
                title=section_title
                request_type=Signal::derive(move || GetAudioBookRequestType::ByAuthor(author_entity(), current_page()))
                subscription_type=Some(Signal::derive(move || {
                    SubscriptionType::ToAuthor(author_entity())
                }))
            />
        <Paginator current_page=Signal::derive(current_page) n_pages=get_amount path=path/>
    }
}
