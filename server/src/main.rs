// use axum::http::header::HeaderMap;
use app::{shell, App};
use axum::{
    body::Body as AxumBody, extract::State, http::Request, response::IntoResponse, routing::get,
    Router,
};
use clap::Parser;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
use log::info;
use neo4rs::ConfigBuilder;
use neo4rs::Graph;
use shared::state::AppState;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Username for postgres.
    #[arg(long)]
    neo4j_username: String,

    /// Username for postgres.
    #[arg(long)]
    neo4j_password: String,

    /// URL for postgres.
    #[arg(long)]
    neo4j_url: String,
}

async fn server_fn_handler(
    State(_app_state): State<AppState>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            // provide_context(app_state.state_str.clone());
        },
        request,
    )
    .await
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    info!("Connecting to NEO4J");
    let config = ConfigBuilder::default()
        .uri(args.neo4j_url)
        .user(args.neo4j_username)
        .password(args.neo4j_password)
        .max_connections(16)
        .fetch_size(200)
        .build()
        .unwrap();
    let graph = Graph::connect(config).await.unwrap();
    let app_state = AppState {
        leptos_options,
        graph,
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
    info!("listening on http://{}", &addr);
    axum::serve(listener, binary.into_make_service())
        .await
        .unwrap();
}
