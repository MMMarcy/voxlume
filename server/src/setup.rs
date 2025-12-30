use std::sync::Arc;
use std::time::Duration;

use app::App;
use argon2::Argon2;
use axum::Router;
use axum::http::HeaderValue;
use axum::http::header::CACHE_CONTROL;
use axum::routing::{get, post};
use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_auth::{AuthConfig, AuthSessionLayer};
use axum_session_sqlx::SessionPgPool;
use chrono::Duration as ChronoDuration;
use entities_lib::entities::meta_request::{MetaRequest, MetaResponse};
use entities_lib::{AudiobookWithData, Environment, GetAudioBookRequestType};
use gemini_rust::Gemini;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use moka::future::Cache;
use reqwest::{Client, ClientBuilder};
use shared::db_ops::AppError;
use shared::db_ops::parade::get_postgres_connection;
use shared::private_args::Args;
use shared::sql_user::SqlUser;
use shared::state::AppState;
use sqlx::PgPool;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::handlers::{leptos_routes_handler, server_fn_handler};

/// Initializes the application by setting up the state, session store, and router.
///
/// # Arguments
/// * `args` - The command line arguments.
///
/// # Returns
/// A tuple containing the `Router` and the address string to listen on.
pub async fn init_app(args: &Args) -> (Router, String) {
    let (app_state, session_store) = init_app_state(args).await;
    let addr = app_state.leptos_options.site_addr.to_string();
    let router = create_router(&app_state, session_store);
    (router, addr)
}

/// Initializes the application state and session store.
///
/// # Arguments
/// * `args` - The command line arguments.
///
/// # Returns
/// A tuple containing `AppState` and `SessionStore`.
async fn init_app_state(args: &Args) -> (AppState, SessionStore<SessionPgPool>) {
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let (g, g_emb) = init_gemini_models(args);
    let pg_pool = get_postgres_connection(args).await;
    let session_store = init_session_store(&pg_pool, args).await;
    let (audiobook_cache, meta_cache) = init_caches(args);
    let http_client: Client = ClientBuilder::new().build().unwrap();

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        database_connection_pool: pg_pool.clone(),
        routes: routes.clone(),
        password_handler: Argon2::default(),
        audiobooks_cache: audiobook_cache,
        meta_requests_cache: meta_cache,
        shareable_args: args.shared.clone(),
        http_client,
        embedder: Arc::new(g_emb),
        content_generator: Arc::new(g),
    };

    (app_state, session_store)
}

/// Creates the Axum router with all necessary routes and layers.
///
/// # Arguments
/// * `app_state` - The initialized application state.
/// * `session_store` - The initialized session store.
///
/// # Returns
/// The configured Axum `Router`.
fn create_router(app_state: &AppState, session_store: SessionStore<SessionPgPool>) -> Router {
    let leptos_options = &app_state.leptos_options;
    let app_routes = &app_state.routes;
    let pg_pool = &app_state.database_connection_pool;
    let other_state = app_state.clone();

    let pkg_path = format!(
        "{}/{}",
        &leptos_options.site_root, &leptos_options.site_pkg_dir
    );
    let pkg_service = ServeDir::new(pkg_path)
        .precompressed_gzip()
        .precompressed_br();
    let pkg_router =
        Router::new()
            .fallback_service(pkg_service)
            .layer(SetResponseHeaderLayer::if_not_present(
                CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000, immutable"),
            ));

    // Build our application with a route
    let mut router = Router::new()
        .route("/api/{*fn_name}", post(server_fn_handler))
        .leptos_routes_with_handler(app_routes.clone(), get(leptos_routes_handler))
        .fallback(leptos_axum::file_and_error_handler_with_context::<
            AppState,
            _,
        >(
            move || {
                provide_context(other_state.clone());
            },
            app::shell,
        ))
        .layer(
            AuthSessionLayer::<SqlUser, i64, SessionPgPool, PgPool>::new(Some(pg_pool.clone()))
                .with_config(AuthConfig::<i64>::default()),
        )
        .layer(SessionLayer::new(session_store))
        .layer(CompressionLayer::new())
        .with_state(app_state.clone()); // app_state is cloned here because it's used above

    if app_state.shareable_args.environment == Environment::PROD {
        router = router.nest_service(&format!("/{}", &leptos_options.site_pkg_dir), pkg_router);
    }

    router
}

/// Initializes the Gemini models (content generator and embedder).
///
/// # Arguments
/// * `args` - The command line arguments containing API keys and model names.
///
/// # Returns
/// A tuple containing the content generator `Gemini` instance and the embedder `Gemini` instance.
fn init_gemini_models(args: &Args) -> (Gemini, Gemini) {
    let g = Gemini::with_model(
        &args.gemini_api_key,
        args.shared.gemini_extract_html_model_name.clone(),
    )
    .unwrap();
    let g_emb = Gemini::with_model(
        &args.gemini_api_key,
        args.shared.gemini_embedding_model_name.clone(),
    )
    .unwrap();
    (g, g_emb)
}

/// Initializes the session store using `PostgreSQL`.
///
/// # Arguments
/// * `pg_pool` - The `PostgreSQL` connection pool.
/// * `args` - The command line arguments containing session duration configuration.
///
/// # Returns
/// The initialized `SessionStore`.
async fn init_session_store(pg_pool: &PgPool, args: &Args) -> SessionStore<SessionPgPool> {
    let session_config = SessionConfig::default()
        .with_max_age(Some(ChronoDuration::days(
            args.cookie_and_session_duration_days,
        )))
        .with_max_lifetime(ChronoDuration::days(args.cookie_and_session_duration_days))
        .with_table_name("axum_sessions");

    SessionStore::<SessionPgPool>::new(Some(SessionPgPool::from(pg_pool.clone())), session_config)
        .await
        .unwrap()
}

/// Initializes the application caches.
///
/// # Arguments
/// * `args` - The command line arguments containing cache configuration.
///
/// # Returns
/// A tuple containing the audiobook cache and the meta requests cache.
#[allow(clippy::type_complexity)]
fn init_caches(
    args: &Args,
) -> (
    Cache<GetAudioBookRequestType, Result<Vec<AudiobookWithData>, AppError>>,
    Cache<MetaRequest, Result<MetaResponse, AppError>>,
) {
    let audiobook_cache = Cache::builder()
        .max_capacity(args.audiobook_cache_max_capacity)
        .time_to_live(Duration::from_secs(args.audiobook_cache_ttl))
        .build();

    let meta_cache = Cache::builder()
        .max_capacity(args.meta_cache_max_capacity)
        .time_to_live(Duration::from_secs(args.meta_cache_ttl))
        .build();

    (audiobook_cache, meta_cache)
}
