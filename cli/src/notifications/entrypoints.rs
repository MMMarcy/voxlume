use std::error::Error;
use std::time::Duration;

use shared::db_ops::parade::get_postgres_connection;
use shared::db_ops::pgmq::get_pgmq_queue;
use shared::private_args::Args;
use sqlx::{PgPool, Row};
use tokio::time::sleep;
use tracing::{debug, info};

use crate::queue_messages::IngestedAudiobookMessage;

async fn insert_or_update_notification(
    pgpool: &PgPool,
    user_ids: Vec<i64>,
    audiobook_id: &i64,
    reason_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pgpool.begin().await?;
    for user_id in user_ids {
        sqlx::query(
            r"
            INSERT INTO public.user_notification (user_id, audiobook_id, reasons)
            VALUES ($1, $2, ARRAY[$3]::notification_reason[])
            ON CONFLICT (user_id, audiobook_id)
            DO UPDATE SET
            reasons = (
                SELECT array_agg(DISTINCT reason)
                FROM unnest(user_notification.reasons || EXCLUDED.reasons) AS t(reason)
            ); 
    ",
        )
        .bind(user_id)
        .bind(audiobook_id)
        .bind(reason_type)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

async fn get_entities_related_to_audiobook(
    pgpool: &PgPool,
    table_name: &str,
    target_column_name: &str,
    audiobook_id: &i64,
) -> Result<Vec<i64>, Box<dyn Error>> {
    let query = format!(
        r"
        SELECT {target_column_name}
        FROM {table_name}
        WHERE audiobook_id = $1
        "
    );
    let res: Vec<i64> = sqlx::query_scalar(&query)
        .bind(audiobook_id)
        .fetch_all(pgpool)
        .await?;
    debug!(
        "Found {} entities {table_name} related to audiobook with id {audiobook_id}",
        res.len()
    );
    Ok(res)
}

async fn get_user_ids_interested_to_entity(
    pgpool: &PgPool,
    table_name: &str,
    target_column_name: &str,
    entity_id: i64,
) -> Result<Vec<i64>, Box<dyn Error>> {
    let query = format!(
        r"
        SELECT user_id
        FROM {table_name}
        WHERE {target_column_name} = $1    "
    );
    let res: Vec<i64> = sqlx::query_scalar(&query)
        .bind(entity_id)
        .fetch_all(pgpool)
        .await?;
    debug!(
        "Found {} users subscribed to entity {table_name} with id {entity_id}",
        res.len()
    );
    Ok(res)
}

async fn get_user_interested_to_entitities_and_update_notifications(
    pgpool: &PgPool,
    join_table_name: &str,
    join_table_column_name: &str,
    notification_table_name: &str,
    notification_table_column_name: &str,
    notification_reason: &str,
    audiobook_id: &i64,
) -> Result<(), Box<dyn Error>> {
    let entities_ids = get_entities_related_to_audiobook(
        pgpool,
        join_table_name,
        join_table_column_name,
        audiobook_id,
    )
    .await?;
    for entity_id in entities_ids {
        let interested_users = get_user_ids_interested_to_entity(
            pgpool,
            notification_table_name,
            notification_table_column_name,
            entity_id,
        )
        .await?;
        insert_or_update_notification(pgpool, interested_users, audiobook_id, notification_reason)
            .await?;
    }
    Ok(())
}
/// Loops over the notification queues and shows them to users.
///
/// # Errors
///     - If the pgmq queue can't be created.
///
/// # Panics
///     - It shouldn't panic despite the `unwrap` in the code.
pub async fn handle_notifications(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let pgpool = get_postgres_connection(&args).await;
    let queue = get_pgmq_queue(&args, &args.pgmq_notifications_queue_name).await?;
    let notifications_queue_name = args.pgmq_notifications_queue_name.clone();

    loop {
        let maybe_audiobook_message = queue
            .pop::<IngestedAudiobookMessage>(&notifications_queue_name)
            .await?;

        // If the subscription is none it means the queue is currently empty and so
        // we just wait a bit before trying to pop a message again.
        if maybe_audiobook_message.is_none() {
            debug!("No message in queue. Looping without taking any action.");
            sleep(Duration::from_secs(60)).await;
            continue;
        }
        debug!("Audiobook id message retrieved from queue.");
        let audiobook_id: i64 = maybe_audiobook_message.unwrap().message.audiobook_id;

        // At this point we need to define what happens here. The logic performs as follows:
        // 1. Get the series id, keywords, ... from that id.
        // 2. For each of those elements, check in the notification tables which users is
        //    subscribed to that particular entity.
        // 3. For each user found, create a notification entry if not already there.

        // 1.1 - Series
        let maybe_series_id: Option<Option<i64>> =
            sqlx::query_scalar("SELECT series_id FROM audiobook where id = $1")
                .bind(audiobook_id)
                .fetch_optional(&pgpool)
                .await?;
        if let Some(Some(series_id)) = maybe_series_id {
            let rows = sqlx::query(
                "SELECT user_id FROM public.user_series_notification WHERE series_id = $1",
            )
            .bind(series_id)
            .fetch_all(&pgpool)
            .await?;

            let user_ids: Vec<i64> = rows
                .iter()
                .map(|row| row.get::<i64, _>("user_id"))
                .collect();
            insert_or_update_notification(&pgpool, user_ids, &audiobook_id, "match_series").await?;
        } else {
            debug!(
                "No series id associated with audiobook with id {}",
                &audiobook_id
            );
        }

        // Authors
        info!(
            "Pushing notifications of authors of audiobook {}",
            audiobook_id
        );
        get_user_interested_to_entitities_and_update_notifications(
            &pgpool,
            "audiobook_author",
            "author_id",
            "user_author_notification",
            "author_id",
            "match_author",
            &audiobook_id,
        )
        .await?;

        // Readers
        info!(
            "Pushing notifications of readers of audiobook {}",
            audiobook_id
        );
        get_user_interested_to_entitities_and_update_notifications(
            &pgpool,
            "audiobook_reader",
            "reader_id",
            "user_reader_notification",
            "reader_id",
            "match_reader",
            &audiobook_id,
        )
        .await?;

        // Categories
        info!(
            "Pushing notifications of categories of audiobook {}",
            audiobook_id
        );
        get_user_interested_to_entitities_and_update_notifications(
            &pgpool,
            "audiobook_category",
            "category_id",
            "user_category_notification",
            "category_id",
            "match_category",
            &audiobook_id,
        )
        .await?;

        // Keywords
        info!(
            "Pushing notifications of keywords of audiobook {}",
            audiobook_id
        );
        get_user_interested_to_entitities_and_update_notifications(
            &pgpool,
            "audiobook_keyword",
            "keyword_id",
            "user_keyword_notification",
            "keyword_id",
            "match_keyword",
            &audiobook_id,
        )
        .await?;
    }
}

/// Sends a test notification to the notification queues.
///
/// # Errors
///   - If it cannot create the queue
///   - If it cannot get the notirication queue.
pub async fn handle_send_test_notifications(
    audiobook_id: i64,
    args: Args,
) -> Result<(), Box<dyn Error>> {
    let queue = get_pgmq_queue(&args, &args.pgmq_notifications_queue_name).await?;
    let notifications_queue_name = args.pgmq_notifications_queue_name.clone();

    let message = IngestedAudiobookMessage::new(audiobook_id);
    queue.send(&notifications_queue_name, &message).await?;
    Ok(())
}
