#![recursion_limit = "256"]

pub mod args;

use std::time::Duration;

use app::{shell, App};
use argon2::Argon2;
use axum::{
    body::Body as AxumBody,
    extract::{Path, State},
    http::Request,
    response::IntoResponse,
    response::Response,
    routing::{get, post},
    Router,
};
use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use axum_session_sqlx::SessionPgPool;
use clap::Parser;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
use moka::future::Cache;
use neo4rs::ConfigBuilder;
use neo4rs::Graph;
use shared::{shared_args::Environment, state::AppState};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use tracing::info;
use tracing_subscriber::{self, fmt::format::FmtSpan};

use crate::args::Args;
use shared::auth_user::AuthSession;
use shared::sql_user::SqlUser;

async fn server_fn_handler(
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

async fn leptos_routes_handler(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    req: Request<AxumBody>,
) -> Response {
    let options_copy = app_state.leptos_options.clone();
    let handler = leptos_axum::render_app_to_stream_in_order_with_context(
        move || {
            info!("Inside provde context for leptos routes handler");
            provide_context(auth_session.clone());
            provide_context(app_state.clone());
        },
        move || shell(options_copy.clone()),
    );
    handler(req).await.into_response()
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

    match args.shared.environment {
        // TODO: In dev it is a bit too verbose. Especially the timestamp field. Find a way to
        // format that.
        Environment::DEV => tracing_subscriber::fmt()
            .pretty()
            .with_target(true)
            .with_span_events(FmtSpan::CLOSE) // Or FmtSpan::ENTER, FmtSpan::CLOSE etc.
            .init(),

        // TODO: Add a sink to disk for logs.
        Environment::PROD => tracing_subscriber::fmt()
            .json()
            .with_span_events(FmtSpan::CLOSE) // Or FmtSpan::ENTER, FmtSpan::CLOSE etc.
            .with_current_span(false)
            .init(),
    };

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

    let cache_ttl: u64 = match args.shared.environment {
        Environment::DEV => 5,
        Environment::PROD => 1800,
    };
    let cache = Cache::builder()
        .max_capacity(100)
        .time_to_live(Duration::from_secs(cache_ttl))
        .build();

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        graph: graph.clone(),
        database_connection_pool: pg_pool.clone(),
        routes: routes.clone(),
        password_handler: Argon2::default(),
        cache: cache.clone(),
        shareable_args: args.shared.clone(),
    };
    let other_state = app_state.clone();
    // Build our application with a route
    let binary = Router::new()
        .route("/api/{*fn_name}", post(server_fn_handler))
        .leptos_routes_with_handler(routes.clone(), get(leptos_routes_handler))
        .fallback(leptos_axum::file_and_error_handler_with_context::<
            AppState,
            _,
        >(
            move || {
                provide_context(other_state.clone());
            },
            shell,
        ))
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
