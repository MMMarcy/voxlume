use std::fmt::Debug;

use argon2::Argon2;
use axum::extract::FromRef;
use leptos::prelude::use_context;
use leptos::prelude::LeptosOptions;
use leptos::prelude::ServerFnError;
use leptos_axum::AxumRouteListing;
use neo4rs::Graph;
use sqlx::postgres::PgPool;
/// This takes advantage of Axum's `SubStates` feature by deriving `FromRef`. This is the only way to have more than one
/// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub graph: Graph,
    pub pg_pool: PgPool,
    pub routes: Vec<AxumRouteListing>,
    pub argon2_params: Argon2<'static>,
}

impl AppState {
    pub fn get_db_conn() -> Result<PgPool, ServerFnError> {
        if let Some(ctx) = use_context::<AppState>() {
            Ok(ctx.pg_pool)
        } else {
            Err(ServerFnError::ServerError(
                "Couldn't find the context.".into(),
            ))
        }
    }
    pub fn get_neo4j_conn() -> Result<Graph, ServerFnError> {
        if let Some(ctx) = use_context::<AppState>() {
            Ok(ctx.graph)
        } else {
            Err(ServerFnError::ServerError(
                "Couldn't find the context.".into(),
            ))
        }
    }

    pub fn get_argon2_params() -> Result<Argon2<'static>, ServerFnError> {
        if let Some(ctx) = use_context::<AppState>() {
            Ok(ctx.argon2_params)
        } else {
            Err(ServerFnError::ServerError(
                "Couldn't find the context.".into(),
            ))
        }
    }
}
impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppState {{...}}")
    }
}
