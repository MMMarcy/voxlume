use entities_lib::entities::subscription::{Subscription, SubscriptionType};
use entities_lib::{Author, Category, Reader, Series};
use sqlx::{FromRow, PgPool};
use tracing::instrument;

use crate::db_ops::AppError;

// TODO:make this return an i64 when done switching to ids.
/// Returns the required SQL table and column names for a given subscription type.
fn get_subscription_sql_info(
    subscription_type: &SubscriptionType,
) -> (&'static str, &'static str, &'static str, &'static str, i64) {
    match subscription_type {
        SubscriptionType::ToAuthor(author) => (
            "user_author_notification",
            "author",
            "author_id",
            "id",
            author.id,
        ),
        SubscriptionType::ToReader(reader) => (
            "user_reader_notification",
            "reader",
            "reader_id",
            "id",
            reader.id,
        ),
        SubscriptionType::ToSeries(series) => (
            "user_series_notification",
            "series",
            "series_id",
            "id",
            series.id,
        ),
        SubscriptionType::ToCategory(category) => (
            "user_category_notification",
            "category",
            "category_id",
            "id",
            category.id,
        ),
        SubscriptionType::ToKeyword(keyword) => (
            "user_keyword_notification",
            "keyword",
            "keyword_id",
            "id",
            keyword.id,
        ),
    }
}

/// Adds a new subscription for a user.
///
/// # Errors
/// Returns an error if the database operation fails.
#[instrument(skip_all)]
pub async fn add_subscription(pool: &PgPool, subscription: Subscription) -> Result<(), AppError> {
    let (join_table, target_table, target_id_col, target_value_col, target_value) =
        get_subscription_sql_info(&subscription.subscription_type);

    let query_str = format!(
        "INSERT INTO public.{join_table} (user_id, {target_id_col})
         SELECT $1, id
         FROM public.{target_table}
         WHERE {target_value_col} = $2
         ON CONFLICT DO NOTHING",
    );

    sqlx::query(&query_str)
        .bind(subscription.user_id)
        .bind(target_value)
        .execute(pool)
        .await
        .map_err(|e| AppError::GenericError(e.to_string()))?;

    Ok(())
}

/// Checks if a specific subscription exists for a user.
///
/// # Errors
/// Returns an error if the database operation fails.
#[instrument(skip_all)]
pub async fn subscription_exists(
    pool: &PgPool,
    subscription: Subscription,
) -> Result<bool, AppError> {
    let (join_table, target_table, target_id_col, target_value_col, target_value) =
        get_subscription_sql_info(&subscription.subscription_type);

    let query_str = format!(
        "SELECT EXISTS (
            SELECT 1
            FROM public.{join_table} jt
            JOIN public.{target_table} tt ON jt.{target_id_col} = tt.id
            WHERE jt.user_id = $1 AND tt.{target_value_col} = $2
        )"
    );

    let exists = sqlx::query_scalar(&query_str)
        .bind(subscription.user_id)
        .bind(target_value)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::GenericError(e.to_string()))?;

    Ok(exists)
}

/// Deletes a subscription for a user.
///
/// # Errors
/// Returns an error if the database operation fails.
#[instrument(skip_all)]
pub async fn delete_subscription(
    pool: &PgPool,
    subscription: Subscription,
) -> Result<(), AppError> {
    let (join_table, target_table, target_id_col, target_value_col, target_value) =
        get_subscription_sql_info(&subscription.subscription_type);

    let query_str = format!(
        "DELETE FROM public.{join_table}
         WHERE user_id = $1 AND {target_id_col} = (
             SELECT id FROM public.{target_table} WHERE {target_value_col} = $2
         )"
    );

    sqlx::query(&query_str)
        .bind(subscription.user_id)
        .bind(target_value)
        .execute(pool)
        .await
        .map_err(|e| AppError::GenericError(e.to_string()))?;

    Ok(())
}

#[allow(clippy::struct_field_names)]
#[derive(FromRow)]
struct SubscriptionRow {
    subscription_id: i64,
    subscription_kind: String,
    subscription_value: String,
}

/// Returns all the subscriptions for the given user.
///
/// # Errors
///   - If the query cannot be performed
///   - If the query produces a `subscription_kind` that is not supported.
#[instrument(skip(pool))]
pub async fn list_subscriptions(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<Subscription>, AppError> {
    // A temporary struct to hold the raw result from the UNION ALL query.
    // The field names must match the column aliases in the SQL query.

    let query_str = r"
        SELECT 
            'author' AS subscription_kind,
            a.name AS subscription_value,
            a.id  as subscription_id
        FROM public.user_author_notification uan
        JOIN public.author a ON uan.author_id = a.id
        WHERE uan.user_id = $1

        UNION ALL

        SELECT 
            'reader' AS subscription_kind, 
            r.name AS subscription_value,
            r.id AS subscription_id
        FROM public.user_reader_notification urn
        JOIN public.reader r ON urn.reader_id = r.id
        WHERE urn.user_id = $1

        UNION ALL

        SELECT 
            'series' AS subscription_kind,
            s.title AS subscription_value,
            s.id AS subscription_id
        FROM public.user_series_notification usn
        JOIN public.series s ON usn.series_id = s.id
        WHERE usn.user_id = $1

        UNION ALL

        SELECT 
            'category' AS subscription_kind,
            s.name AS subscription_value,
            s.id AS subscription_id
        FROM public.user_category_notification ucn
        JOIN public.category s ON ucn.category_id = s.id
        WHERE ucn.user_id = $1
    ";

    // Execute the query and fetch all rows, mapping them to our temporary struct.
    let rows = sqlx::query_as::<_, SubscriptionRow>(query_str)
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::GenericError(e.to_string()))?;

    // Transform the flat list of rows into the structured `Vec<Subscription>`.
    let subscriptions = rows
        .into_iter()
        .map(|row| {
            let subscription_type = match row.subscription_kind.as_str() {
                "author" => SubscriptionType::ToAuthor(Author {
                    id: row.subscription_id,
                    name: row.subscription_value,
                }),
                "reader" => SubscriptionType::ToReader(Reader {
                    id: row.subscription_id,
                    name: row.subscription_value,
                }),
                // Note: The Series struct uses `title`, but our query aliases it to `subscription_value`.
                "series" => SubscriptionType::ToSeries(Series {
                    id: row.subscription_id,
                    title: row.subscription_value,
                }),
                "category" => SubscriptionType::ToCategory(Category {
                    id: row.subscription_id,
                    value: row.subscription_value,
                }),
                // This case should not be reached if the SQL query is correct.
                _ => unreachable!(
                    "Unexpected subscription kind from DB: {}",
                    row.subscription_kind
                ),
            };

            Subscription {
                user_id,
                subscription_type,
            }
        })
        .collect();

    Ok(subscriptions)
}
