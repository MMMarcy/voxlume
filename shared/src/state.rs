use std::fmt::Debug;

use argon2::Argon2;
use axum::extract::FromRef;
use entities_lib::AudiobookWithData;
use entities_lib::GetAudioBookRequestType;
use leptos::prelude::use_context;
use leptos::prelude::LeptosOptions;
use leptos::prelude::ServerFnError;
use leptos_axum::AxumRouteListing;
use moka::future::Cache;
use neo4rs::Graph;
use sqlx::postgres::PgPool;

use crate::db_trait::DbConnectionLike;
use crate::graph_trait::AppError;
use crate::password_handler::PasswordHandlerLike;
use crate::shared_args::ShareableArgsValues;
/// This takes advantage of Axum's `SubStates` feature by deriving `FromRef`. This is the only way to have more than one
/// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub graph: Graph,
    pub database_connection_pool: PgPool,
    pub routes: Vec<AxumRouteListing>,
    pub password_handler: Argon2<'static>,
    pub cache: Cache<GetAudioBookRequestType, Result<Vec<AudiobookWithData>, AppError>>,
    pub shareable_args: ShareableArgsValues,
}

impl AppState {
    ///
    /// # Errors
    /// If it can't find the appstate.
    pub fn get_app_state() -> Result<AppState, ServerFnError> {
        if let Some(ctx) = use_context::<AppState>() {
            Ok(ctx)
        } else {
            Err(ServerFnError::ServerError(
                "Couldn't get the appstate from the context".into(),
            ))
        }
    }

    ///
    /// # Errors
    /// If the it can't find the database connection.
    pub fn get_database_connection_pool() -> Result<impl DbConnectionLike, ServerFnError> {
        if let Some(ctx) = use_context::<AppState>() {
            Ok(ctx.database_connection_pool)
        } else {
            Err(ServerFnError::ServerError(
                "Couldn't find the context.".into(),
            ))
        }
    }

    /// # Errors
    /// If it can't get the neo4rs graph instance.
    pub fn get_neo4j_conn() -> Result<Graph, ServerFnError> {
        if let Some(graph) = use_context::<Graph>() {
            Ok(graph)
        } else {
            Err(ServerFnError::ServerError(
                "Couldn't find the graph in the context.".into(),
            ))
        }
    }

    /// # Errors
    /// If it can't find the password handler.
    pub fn get_password_handler() -> Result<impl PasswordHandlerLike, ServerFnError> {
        if let Some(ctx) = use_context::<AppState>() {
            Ok(ctx.password_handler)
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
