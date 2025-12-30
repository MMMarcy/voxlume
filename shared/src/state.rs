use std::fmt::Debug;
use std::sync::Arc;

use argon2::Argon2;
use axum::extract::FromRef;
use entities_lib::entities::meta_request::{MetaRequest, MetaResponse};
use entities_lib::{AudiobookWithData, GetAudioBookRequestType};
use leptos::prelude::{LeptosOptions, ServerFnError, use_context};
use leptos_axum::AxumRouteListing;
use moka::future::Cache;
use reqwest::Client;
use sqlx::postgres::PgPool;

use crate::db_ops::AppError;
use crate::db_trait::DbConnectionLike;
use crate::password_handler::PasswordHandlerLike;
use crate::utils::gemini::{GeminiContentEmbedderLike, GeminiContentGeneratorLike};
use entities_lib::ShareableArgsValues;
/// This takes advantage of Axum's `SubStates` feature by deriving `FromRef`. This is the only way to have more than one
/// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub database_connection_pool: PgPool,
    pub routes: Vec<AxumRouteListing>,
    pub password_handler: Argon2<'static>,
    pub audiobooks_cache: Cache<GetAudioBookRequestType, Result<Vec<AudiobookWithData>, AppError>>,
    pub meta_requests_cache: Cache<MetaRequest, Result<MetaResponse, AppError>>,
    pub shareable_args: ShareableArgsValues,
    pub http_client: Client,
    pub embedder: Arc<dyn GeminiContentEmbedderLike>,
    pub content_generator: Arc<dyn GeminiContentGeneratorLike>,
}

impl AppState {
    ///
    /// # Errors
    /// If it can't find the appstate.
    pub fn get_app_state() -> Result<AppState, ServerFnError> {
        match use_context::<AppState>() {
            Some(ctx) => Ok(ctx),
            _ => Err(ServerFnError::ServerError(
                "Couldn't get the appstate from the context".into(),
            )),
        }
    }

    ///
    /// # Errors
    /// If the it can't find the database connection.
    pub fn get_database_connection_pool() -> Result<impl DbConnectionLike, ServerFnError> {
        match use_context::<AppState>() {
            Some(ctx) => Ok(ctx.database_connection_pool),
            _ => Err(ServerFnError::ServerError(
                "Couldn't find the context.".into(),
            )),
        }
    }

    /// # Errors
    /// If it can't find the password handler.
    pub fn get_password_handler() -> Result<impl PasswordHandlerLike, ServerFnError> {
        match use_context::<AppState>() {
            Some(ctx) => Ok(ctx.password_handler),
            _ => Err(ServerFnError::ServerError(
                "Couldn't find the context.".into(),
            )),
        }
    }
}
impl Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppState {{...}}")
    }
}
