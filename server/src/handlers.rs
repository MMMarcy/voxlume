use app::shell;
use axum::body::Body as AxumBody;
use axum::extract::{Path, State};
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use leptos::prelude::provide_context;
use leptos_axum::handle_server_fns_with_context;
use shared::auth_user::AuthSession;
use shared::state::AppState;
use tracing::info;

pub async fn server_fn_handler(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    path: Path<String>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    info!("{:?}", path);

    handle_server_fns_with_context(
        move || {
            info!("Inside provide context for server fn handler");
            provide_context(auth_session.clone());
            provide_context(app_state.clone());
        },
        request,
    )
    .await
}

pub async fn leptos_routes_handler(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    req: Request<AxumBody>,
) -> Response {
    let options_copy = app_state.leptos_options.clone();
    // Changed from render_app_to_stream_in_order_with_context to render_app_to_stream_with_context
    let handler = leptos_axum::render_app_to_stream_with_context(
        move || {
            info!("Inside provde context for leptos routes handler");
            provide_context(auth_session.clone());
            provide_context(app_state.clone());
        },
        move || shell(options_copy.clone()),
    );
    handler(req).await.into_response()
}
