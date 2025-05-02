#![recursion_limit = "256"]

pub mod args;

// use axum::http::header::HeaderMap;
use app::{shell, App};
use argon2::Argon2;
use args::Environment;
use axum::{
    body::Body as AxumBody,
    extract::{Path, State},
    http::Request,
    response::IntoResponse,
    response::Response,
    routing::get,
    Router,
};
use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use axum_session_sqlx::SessionPgPool;
use clap::Parser;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
use neo4rs::ConfigBuilder;
use neo4rs::Graph;
use shared::state::AppState;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use tracing::info;
use tracing_subscriber;

use crate::args::Args;
use shared::auth_user::{AuthSession, SqlUser};

async fn server_fn_handler(
    State(app_state): State<AppState>,
    auth_session: AuthSession,
    path: Path<String>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    info!("{:?}", path);

    handle_server_fns_with_context(
        move || {
            provide_context(auth_session.clone());
            provide_context(app_state.pg_pool.clone());
            provide_context(app_state.clone());
            provide_context(Argon2::default());
        },
        request,
    )
    .await
}

async fn leptos_routes_handler(
    auth_session: AuthSession,
    state: State<AppState>,
    req: Request<AxumBody>,
) -> Response {
    let State(app_state) = state.clone();

    let handler = leptos_axum::render_route_with_context(
        app_state.routes.clone(),
        move || {
            provide_context(auth_session.clone());
            provide_context(app_state.pg_pool.clone());
            provide_context(app_state.graph.clone());
        },
        move || shell(app_state.leptos_options.clone()),
    );
    handler(state, req).await.into_response()
}

async fn get_neo4j_connection(args: &Args) -> Graph {
    info!("Connecting to NEO4J");
    let neo4j_config = ConfigBuilder::default()
        .uri(args.neo4j_url.clone())
        .user(args.neo4j_username.clone())
        .password(args.neo4j_password.clone())
        .max_connections(16)
        .fetch_size(200)
        .build()
        .unwrap();
    Graph::connect(neo4j_config).await.unwrap()
}

async fn get_postgres_connection(args: &Args) -> PgPool {
    let postgres_conn_str = format!(
        "postgres://{username}:{password}@{host}/voxlume",
        username = args.postgres_username.clone(),
        password = args.postgres_password.clone(),
        host = args.postgres_url.clone()
    );
    info!("Connecting to Postgres");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_conn_str)
        .await
        .unwrap();

    let migrations_result = sqlx::migrate!("../model/migrations").run(&pool).await;
    match migrations_result {
        Ok(_) => info!("Migrations ran successfully"),
        Err(err) => info!("Migrations failed. Error: {}", err),
    }
    pool
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.environment {
        Environment::DEV => tracing_subscriber::fmt().compact().init(),
        Environment::PROD => tracing_subscriber::fmt()
            .json()
            .with_current_span(false)
            .init(),
    }

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    let graph = get_neo4j_connection(&args).await;
    let pg_pool = get_postgres_connection(&args).await;
    let session_config = SessionConfig::default().with_table_name("axum_sessions");
    let auth_config = AuthConfig::<i64>::default();
    let session_store = SessionStore::<SessionPgPool>::new(
        Some(SessionPgPool::from(pg_pool.clone())),
        session_config,
    )
    .await
    .unwrap();

    let app_state = AppState {
        leptos_options,
        graph,
        pg_pool: pg_pool.clone(),
        routes: routes.clone(),
    };
    // Build our application with a route
    let binary = Router::new()
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .layer(
            AuthSessionLayer::<SqlUser, i64, SessionPgPool, PgPool>::new(Some(pg_pool.clone()))
                .with_config(auth_config),
        )
        .layer(SessionLayer::new(session_store))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("listening on http://{}", &addr);
    axum::serve(listener, binary.into_make_service())
        .await
        .unwrap();
}
