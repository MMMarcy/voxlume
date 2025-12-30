use entities_lib::entities::notifications::UserNotification;
use sqlx::PgPool;

/// Gets all the notifications for a given `user_id`.
///
/// # Errors
///   - If there is a problem with the query.
pub async fn list_user_notifications(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<UserNotification>, sqlx::Error> {
    let notifications: Vec<UserNotification> = sqlx::query_as(
        r"
            SELECT user_id, audiobook_id, created_at, reasons, has_been_seen
            FROM public.user_notification
            WHERE user_id = $1
            ORDER BY created_at DESC
        ",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(notifications)
}

/// Tells whether the user has unseen notifications.
///
/// # Errors
///   - If the query resulted in an error.
pub async fn has_unseen_notifications(pool: &PgPool, user_id: i64) -> Result<bool, sqlx::Error> {
    let has_unseen_notifications: bool = sqlx::query_scalar(
        r"
            SELECT EXISTS (
                SELECT 1
                FROM public.user_notification AS nt
                WHERE nt.user_id = $1
                AND nt.has_been_seen = FALSE
            );
        ",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(has_unseen_notifications)
}

/// Delete a notification.
///
/// # Errors
///   - If there is a problem deleting.
///   - If the notification can't be found
pub async fn delete_notification(
    pool: &PgPool,
    notification: UserNotification,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query(
        r"
            DELETE FROM public.user_notification
            WHERE user_id = $1 AND audiobook_id = $2
        ",
    )
    .bind(notification.user_id)
    .bind(notification.audiobook_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

/// Mark the notification as seen
///
/// # Errors
///   - If there is a problem updating the notification
///   - If the notification can't be found
pub async fn mark_notification_as_seen(
    pool: &PgPool,
    notification: UserNotification,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query(
        r"
            UPDATE public.user_notification
            SET has_been_seen = TRUE
            WHERE user_id = $1 AND audiobook_id = $2
        ",
    )
    .bind(notification.user_id)
    .bind(notification.audiobook_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

/// Mark all notifications as seen for a user
///
/// # Errors
///   - If there is a problem updating the notifications
pub async fn mark_all_notifications_as_seen(
    pool: &PgPool,
    user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
            UPDATE public.user_notification
            SET has_been_seen = TRUE
            WHERE user_id = $1
        ",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete all notifications for a user
///
/// # Errors
///   - If there is a problem deleting the notifications
pub async fn delete_all_user_notifications(pool: &PgPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
            DELETE FROM public.user_notification
            WHERE user_id = $1
        ",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}
