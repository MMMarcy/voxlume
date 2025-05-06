use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::instrument;
use tracing::{debug, warn};

use crate::sql_user::SqlUser;

/// Represents a connection to a database that can perform user-related operations.
/// This trait is designed to be used with dependency injection at some point.
#[async_trait]
pub trait DbConnectionLike: Send + Sync {
    /// Retrieves a user from the database based on their username.
    ///
    /// # Arguments
    ///
    /// * `username` - A reference to a string-like type representing the username to search for.
    ///
    /// # Returns
    ///
    /// An `Option<SqlUser>` which is `Some(SqlUser)` if the user is found, or `None` otherwise.
    async fn get_user_by_username(&self, username: &(impl AsRef<str> + Sync)) -> Option<SqlUser>;

    /// Retrieves a user from the database based on their ID.
    ///
    /// # Arguments
    ///
    /// * `userid` - The ID of the user to search for.
    ///
    /// # Returns
    ///
    /// A `Result<Option<SqlUser>>`. `Ok(Some(SqlUser))` if the user is found,
    /// `Ok(None)` if the user is not found, or an `Err` if a database error occurs.
    /// Note: If `userid` is 1, it returns a default "Guest" user without hitting the database.
    async fn get_user_by_id(&self, userid: i64) -> Result<Option<SqlUser>>;

    /// Inserts a new user into the database.
    ///
    /// # Arguments
    ///
    /// * `user` - The `SqlUser` object containing the user data to insert.
    ///
    /// # Returns
    ///
    /// A `Result<()>` which is `Ok(())` on successful insertion, or an `Err` if a database error occurs.
    async fn insert_user(&self, user: SqlUser) -> Result<()>;
}

/// Implementation of `DbConnectionLike` for a PostgreSQL connection pool (`sqlx::PgPool`).
#[async_trait]
impl DbConnectionLike for PgPool {
    /// Fetches a user by username from the PostgreSQL database.
    /// Returns `None` if the user is not found or if a database error occurs during the query.
    #[instrument(skip_all)]
    async fn get_user_by_username(&self, username: &(impl AsRef<str> + Sync)) -> Option<SqlUser> {
        let get_user_result = sqlx::query_as::<_, SqlUser>(
            r#"
            SELECT *
            FROM users
            WHERE username = $1
        "#,
        )
        .bind(username.as_ref())
        .fetch_optional(self)
        .await;

        // TODO: transform in match to handle also Err and stuff gracefully.
        if let Ok(Some(user)) = get_user_result {
            Some(user)
        } else {
            if let Err(e) = get_user_result {
                warn!(
                    "Error fetching user by username {}: {}",
                    username.as_ref(),
                    e
                );
            }
            None
        }
    }

    /// Fetches a user by ID from the PostgreSQL database.
    /// Handles the special case for `userid` 1, returning a default guest user.
    /// Otherwise, queries the database for the user ID.
    #[instrument(skip_all)]
    async fn get_user_by_id(&self, userid: i64) -> Result<Option<SqlUser>> {
        if userid == 1 {
            warn!("Trying to log in default user");
            return Ok(Some(SqlUser {
                id: userid,
                username: "Guest".to_string(),
                anonymous: true,
                password_mcf: "".to_string(),
                last_access: chrono::Utc::now(),
            }));
        }
        debug!("Loading user with id {}", userid);
        sqlx::query_as::<_, SqlUser>(
            r#"
            SELECT *
            FROM users
            WHERE id = $1
        "#,
        )
        .bind(userid)
        .fetch_optional(self)
        .await
        .map_err(|e| anyhow!(e)) // Maps sqlx::Error to anyhow::Error
    }

    /// Inserts a new user record into the `users` table in the PostgreSQL database.
    /// Assumes the user is not anonymous (sets `anonymous` column to `false`).
    #[instrument(skip_all)]
    async fn insert_user(&self, user: SqlUser) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (
                username,
                anonymous,
                password_mcf,
                last_access
            ) VALUES (
                $1,
                $2,
                $3,
                $4
            );
        "#,
        )
        .bind(user.username)
        .bind(false) // Explicitly set anonymous to false for inserted users
        .bind(user.password_mcf)
        .bind(user.last_access)
        .execute(self)
        .await?; // Propagates potential database errors

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql_user::SqlUser;
    use chrono::Utc;

    // Helper to create a SqlUser instance for testing
    fn create_test_user(username: &str) -> SqlUser {
        SqlUser {
            id: 0, // ID is usually set by the database
            username: username.to_string(),
            anonymous: false,
            password_mcf: format!("mcf_for_{}", username),
            last_access: Utc::now(),
        }
    }

    #[sqlx::test(migrations = "../model/migrations")]
    async fn test_insert_and_get_user_by_username(pool: PgPool) -> Result<()> {
        let test_user = create_test_user("test_user_1");
        let username = test_user.username.clone();

        // Insert the user
        pool.insert_user(test_user.clone()).await?;

        // Retrieve the user by username
        let fetched_user_opt = pool.get_user_by_username(&username).await;

        assert!(
            fetched_user_opt.is_some(),
            "User should be found by username"
        );
        let fetched_user = fetched_user_opt.unwrap();

        assert_eq!(fetched_user.username, username);
        assert_eq!(fetched_user.password_mcf, test_user.password_mcf);
        assert!(!fetched_user.anonymous);
        // Note: ID and last_access might differ slightly (DB generation, time precision)
        // assert_eq!(fetched_user.last_access, test_user.last_access); // Be cautious with timestamp comparisons

        Ok(())
    }

    #[sqlx::test(migrations = "../model/migrations")]
    async fn test_get_user_by_username_not_found(pool: PgPool) -> Result<()> {
        let fetched_user_opt = pool.get_user_by_username(&"non_existent_user").await;
        assert!(fetched_user_opt.is_none(), "User should not be found");
        Ok(())
    }

    #[sqlx::test(migrations = "../model/migrations")]
    async fn test_insert_and_get_user_by_id(pool: PgPool) -> Result<()> {
        let test_user = create_test_user("test_user_for_id");

        // Insert the user
        pool.insert_user(test_user.clone()).await?;

        // Retrieve the user by username first to get the assigned ID
        let inserted_user = pool
            .get_user_by_username(&test_user.username)
            .await
            .expect("User should exist after insert");
        let user_id = inserted_user.id;
        assert!(user_id > 0, "User ID should be assigned by the database");

        // Retrieve the user by ID
        let fetched_user_opt_res = pool.get_user_by_id(user_id).await;

        assert!(
            fetched_user_opt_res.is_ok(),
            "Fetching user by ID should succeed"
        );
        let fetched_user_opt = fetched_user_opt_res.unwrap();
        assert!(fetched_user_opt.is_some(), "User should be found by ID");
        let fetched_user = fetched_user_opt.unwrap();

        assert_eq!(fetched_user.id, user_id);
        assert_eq!(fetched_user.username, test_user.username);
        assert_eq!(fetched_user.password_mcf, test_user.password_mcf);

        Ok(())
    }

    #[sqlx::test(migrations = "../model/migrations")]
    async fn test_get_user_by_id_not_found(pool: PgPool) -> Result<()> {
        let non_existent_id = 999999;
        let fetched_user_opt_res = pool.get_user_by_id(non_existent_id).await;

        assert!(
            fetched_user_opt_res.is_ok(),
            "Fetching non-existent user by ID should not error"
        );
        let fetched_user_opt = fetched_user_opt_res.unwrap();
        assert!(
            fetched_user_opt.is_none(),
            "User with non-existent ID should not be found"
        );

        Ok(())
    }

    #[sqlx::test(migrations = "../model/migrations")]
    async fn test_get_user_by_id_guest_user(pool: PgPool) -> Result<()> {
        let guest_user_id = 1;
        let fetched_user_opt_res = pool.get_user_by_id(guest_user_id).await;

        assert!(
            fetched_user_opt_res.is_ok(),
            "Fetching guest user should succeed"
        );
        let fetched_user_opt = fetched_user_opt_res.unwrap();
        assert!(
            fetched_user_opt.is_some(),
            "Guest user (ID 1) should be returned"
        );
        let fetched_user = fetched_user_opt.unwrap();

        assert_eq!(fetched_user.id, guest_user_id);
        assert_eq!(fetched_user.username, "Guest");
        assert!(fetched_user.anonymous);
        assert_eq!(fetched_user.password_mcf, "");

        Ok(())
    }
}
