// pub mod fileserv;

// use axum::http::header::HeaderMap;
use axum::{
    body::Body as AxumBody, extract::State, http::Request, response::IntoResponse, routing::get,
    Router,
};
use leptos::prelude::*;
use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};

// use crate::fileserv::file_and_error_handler;
use app::{shell, App};
// use server_utils::entities_impl::user_impl::SqlUser;
// use server_utils::entities_impl::user_impl::UserAuthSession;
// use shared::state::AppState;

use leptos::logging;
use shared::state::AppState;
// #[derive(Debug, Parser)]
// #[command(version, about, long_about = None)]
// struct Args {
//     /// Username for postgres.
//     #[arg(long)]
//     postgres_username: String,
//
//     /// Username for postgres.
//     #[arg(long)]
//     postgres_password: String,
//
//     /// URL for postgres.
//     #[arg(long)]
//     postgres_url: String,
//
//     /// Postgres database to use.
//     #[arg(long)]
//     postgres_db: String,
//
//     /// Postgres max concurrent connections.
//     #[arg(long, default_value_t = 5)]
//     postgres_max_connections: u32,
// }

// async fn server_fn_handler(
//     State(app_state): State<AppState>,
//     // auth_session: UserAuthSession,
//     _path: Path<String>,
//     _headers: HeaderMap,
//     _raw_query: RawQuery,
//     request: Request<AxumBody>,
// ) -> impl IntoResponse {
//     handle_server_fns_with_context(
//         move || {
//             // provide_context(auth_session.clone());
//             provide_context(app_state.clone());
//         },
//         request,
//     )
//     .await
//
async fn server_fn_handler(
    State(app_state): State<AppState>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    dbg!(&request);

    handle_server_fns_with_context(
        move || {
            provide_context(app_state.state_str.clone());
        },
        request,
    )
    .await
}

#[tokio::main]
async fn main() {
    // let args = Args::parse();
    // let config: OllamaServerConfig = read_confuguration("config.json").unwrap();

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    // info!("Connecting to PSQL...");

    // let db = get_postgres_pool(&args).await;
    // let session_config = SessionConfig::default().with_table_name("sessions_table");
    // let session_store = SessionStore::<SessionPgPool>::new(Some(db.clone().into()), session_config)
    //   .await
    //     .unwrap();
    // let auth_config = AuthConfig::<i64>::default();

    let app_state = AppState {
        leptos_options,
        state_str: String::from("Test"),
    };
    // build our application with a route
    let binary = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes(&app_state, routes, {
            let app_state = app_state.clone();
            move || shell(app_state.leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<LeptosOptions, _>(
            shell,
        ))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, binary.into_make_service())
        .await
        .unwrap();
}
