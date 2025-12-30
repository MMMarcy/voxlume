use crate::pages::get_counts;
use crate::ui_components::ads::grid_ad::GridAd;
use crate::ui_components::audiobook::audiobook_container::AudioBookCollectionContainer;
use crate::ui_components::paginator::Paginator;
use entities_lib::{
    Category, GetAudioBookRequestType, MetaRequest, MetaResponse, ShareableArgsValues,
};
use leptos::logging;
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[derive(Params, PartialEq, Debug)]
struct CategoryParam {
    category_id: Option<i64>,
    category_name: Option<String>,
    page: Option<u32>,
}

impl TryFrom<&CategoryParam> for Category {
    type Error = ();

    fn try_from(value: &CategoryParam) -> Result<Self, Self::Error> {
        logging::debug_log!("{:?}", value);
        if value.category_id.is_none() || value.category_name.is_none() {
            return Err(());
        }
        Ok(Category {
            id: value.category_id.unwrap(),
            value: value.category_name.clone().unwrap(),
        })
    }
}

#[component]
pub fn CategoryPage() -> impl IntoView {
    let params = use_params::<CategoryParam>();
    let shareable_args = use_context::<ReadSignal<Option<ShareableArgsValues>>>();
    let (get_amount, set_amount) = signal::<Option<u32>>(None);

    // Memoize the parsing of category data.
    // This ensures we only re-parse when route params actually change.
    let category_data = Memo::new(move |_| {
        params.read().as_ref().ok().map_or_else(
            || {
                (
                    Category {
                        id: 1,
                        value: "unknown".to_string(),
                    },
                    1,
                )
            },
            |p| (Category::try_from(p).ok().unwrap(), p.page.unwrap_or(0)),
        )
    });

    // Helper closures to access the memoized data
    let category_entity = move || category_data.get().0;
    let current_page = move || category_data.get().1;

    let section_title = move || format!("Audiobooks with category {}", category_entity().value);

    // Resource now depends on the stable memoized category
    let get_count_op = Resource::new(category_entity, move |cat| {
        logging::debug_warn!("Performing request");
        get_counts(MetaRequest::CountAudiobooksForCategory(cat))
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

    // Make path reactive so Paginator updates when category changes
    let path = Memo::new(move |_| {
        let cat = category_entity();
        format!("/category/{}/{}/{{}}", cat.id, cat.value)
    });

    view! {

        <div class="container is-max-desktop is-flex is-justify-content-center">
            <GridAd ad_slot="1117011249" />
        </div>
        <Paginator current_page=Signal::derive(current_page) n_pages=get_amount path=path/>
            <AudioBookCollectionContainer
                title=Signal::derive(section_title)
                request_type=Signal::derive(move || {
                    GetAudioBookRequestType::ByCategory(category_entity(), current_page())
                })
                subscription_type=None
            />
        <Paginator current_page=Signal::derive(current_page) n_pages=get_amount path=path />
    }
}
