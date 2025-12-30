use std::time::Duration;

use shared::db_ops::parade::get_postgres_connection;
use shared::private_args::Args;
use tokio::time::sleep;

use tracing::info;

///
/// # Errors
/// If refreshing the view or adding the index fails.
pub async fn refresh_search_index(
    args: Args,
    every_n_seconds: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let pg_pool = get_postgres_connection(&args).await;
    sqlx::query(
        r"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_audiobook_search_view_unique_id 
            ON public.audiobook_search_view (audiobook_id);
    ",
    )
    .execute(&pg_pool)
    .await?;

    loop {
        sqlx::query("REFRESH MATERIALIZED VIEW CONCURRENTLY public.audiobook_search_view")
            .execute(&pg_pool)
            .await?;

        info!(
            "Refreshed materiealized view. Sleeping for {} hours now.",
            every_n_seconds / 3600
        );
        sleep(Duration::from_secs(every_n_seconds.into())).await;
    }
}
