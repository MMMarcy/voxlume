use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;
/// This takes advantage of Axum's `SubStates` feature by deriving `FromRef`. This is the only way to have more than one
/// item in Axum's State. Leptos requires you to have leptosOptions in your State struct for the leptos route handlers
#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub state_str: String, // pub pg_pool: PgPool,
}

// pub fn get_db_conn() -> Result<PgPool, ServerFnError> {
//     use_context::<AppState>()
//         .map(|s| s.pg_pool)
//         .ok_or_else(|| ServerFnError::ServerError("DB connection missing.".into()))
// }
//
// pub fn get_neo4j_conn() -> Result<Graph, ServerFnError> {
//     use_context::<AppState>()
//         .map(|s| s.graph)
//         .ok_or_else(|| ServerFnError::ServerError("Neo4j connection missing.".into()))
// }
