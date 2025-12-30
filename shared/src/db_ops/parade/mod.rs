use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

use crate::private_args::Args;

pub mod audiobook_ops;
pub mod meta_ops;
pub mod notifications;
pub mod save_ops;
pub mod search_ops;
pub mod subscription_ops;

/// Creates a `PGPool` or panics.
///
/// # Panics
/// If the connection pool can't be established.
pub async fn get_postgres_connection(args: &Args) -> PgPool {
    let postgres_conn_str = format!(
        "postgres://{username}:{password}@{host}/voxlume",
        username = args.postgres_username.clone(),
        password = args.postgres_password.clone(),
        host = args.postgres_url.clone()
    );
    info!("Connecting to Postgres");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&postgres_conn_str)
        .await
        .unwrap()
}

// TODO: Add the migrations somewhere else.
//
// let migrations_result = sqlx::migrate!("../model/migrations").run(&pool).await;
// match migrations_result {
//     Ok(()) => info!("Migrations ran successfully"),
//     Err(err) => info!("Migrations failed. Error: {}", err),
// }
