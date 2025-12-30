pub mod audiobook_container;
pub mod audiobook_detailed_view;
mod audiobook_grid;
mod audiobook_list;
mod audiobook_table;
mod link_author;
mod link_reader;
mod link_series;
mod tag_category;
mod tag_keyword;

use entities_lib::{AudiobookWithData, GetAudioBookRequestType};
use leptos::prelude::*;

/// Retrieve the audiobooks from the given request type.
///
/// The underlying implementation uses a cache of type `GetAudioBookRequestType ->
/// Vec<AudiobookWithData>` so that reads are cached.
#[server(GetAudiobooks, "/api")]
pub(crate) async fn get_audiobooks(
    request_type: GetAudioBookRequestType,
) -> Result<Vec<AudiobookWithData>, ServerFnError> {
    use shared::auth_user::AuthSession;
    use shared::db_ops::parade::audiobook_ops::get_audiobooks_cached;
    use shared::state::AppState;
    use tracing::info;

    info!("Before getting app state");
    let state = AppState::get_app_state()?;
    let pgpool = state.database_connection_pool;
    let cache = state.audiobooks_cache;
    info!("Gotten app state");
    let auth_session =
        use_context::<AuthSession>().ok_or(ServerFnError::new("Couldn't find auth session"))?;
    let user_id = auth_session.current_user.map_or_else(|| 1i64, |v| v.id);
    let limit_audiobooks = if user_id == 1 {
        state.shareable_args.guest_user_audiobooks_per_homepage
    } else {
        state.shareable_args.user_audiobooks_per_homepage_section
    };
    info!("Gotten here.");
    get_audiobooks_cached(&pgpool, &cache, request_type, limit_audiobooks)
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))
}
