pub(crate) mod about_page;
pub(crate) mod audiobook_page;
pub(crate) mod author_page;
pub(crate) mod by_series_page;
pub(crate) mod category_page;
pub(crate) mod homepage;
pub(crate) mod keyword_page;
pub(crate) mod login;
pub(crate) mod logout;
pub(crate) mod most_recent_page;
pub(crate) mod notifications;
pub(crate) mod reader_page;
pub(crate) mod register;
pub(crate) mod roadmap_page;
pub(crate) mod search_page;
pub(crate) mod subscriptions;

use entities_lib::{MetaRequest, MetaResponse};
use leptos::prelude::*;
use leptos::server_fn::codec::Json;

#[server(GetCounts, "/api", input = Json)]
pub(super) async fn get_counts(request: MetaRequest) -> Result<MetaResponse, ServerFnError> {
    use shared::db_ops::parade::meta_ops::get_meta_cached;
    use shared::state::AppState;

    let state = AppState::get_app_state()?;
    let db_pool = state.database_connection_pool;
    let cache = state.meta_requests_cache;
    get_meta_cached(&db_pool, &cache, request)
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))
}
