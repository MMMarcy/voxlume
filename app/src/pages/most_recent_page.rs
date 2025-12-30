use crate::pages::get_counts;
use crate::ui_components::audiobook::audiobook_container::AudioBookCollectionContainer;
use crate::ui_components::paginator::Paginator;
use entities_lib::{GetAudioBookRequestType, MetaRequest, MetaResponse, ShareableArgsValues};
use leptos::logging;
use leptos::prelude::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[derive(Params, PartialEq, Debug, Clone)]
struct MostRecentParam {
    page: Option<u32>,
}

impl From<&MostRecentParam> for u32 {
    fn from(value: &MostRecentParam) -> Self {
        if let Some(n) = value.page {
            return n;
        }
        1u32
    }
}

#[component]
pub fn MostRecentPage() -> impl IntoView {
    let params = use_params::<MostRecentParam>();

    let shareable_args = use_context::<ReadSignal<Option<ShareableArgsValues>>>()
        .expect("ShareableArgs context must be provided");

    let (get_amount, set_amount) = signal::<Option<u32>>(None);
    let page = Memo::new(move |_| match params.read().as_ref() {
        Ok(v) => u32::from(v),
        Err(_) => 1u32,
    });

    let get_count_op = OnceResource::new(async move {
        logging::debug_warn!("Performing request for most recent");
        get_counts(MetaRequest::CountAllAudiobooks).await
    });

    Effect::new(move || {
        let result = get_count_op.get();
        if let Some(Err(e)) = result.clone() {
            logging::debug_error!("Error: {e:?}"); // This prints requests argument not provided.
        }
        let args_opt = shareable_args.get();

        if let Some(Ok(MetaResponse::Count(amount))) = result
            && let Some(args) = args_opt
        {
            let per_page = args.guest_user_audiobooks_per_homepage;
            let pages = if per_page > 0 {
                amount.div_ceil(per_page)
            } else {
                0
            };
            logging::debug_warn!("Setting most recent pages to: {}", pages);
            set_amount(Some(pages));
        }
    });

    let path = Memo::new(move |_| "/most-recent/{}".to_string());

    view! {
        <Paginator current_page=page n_pages=get_amount path=path/>
            <AudioBookCollectionContainer
                title=Signal::derive(move || String::from("Most recent"))
                request_type=Signal::derive(move || GetAudioBookRequestType::MostRecent(page()))
                subscription_type=None
            />
        <Paginator current_page=page n_pages=get_amount path=path/>
    }
}
