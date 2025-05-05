use anyhow::{anyhow, Result};
use async_trait::async_trait;
use shaku::Interface;
use sqlx::PgPool;
use tracing::instrument;
use tracing::{debug, warn};

use crate::sql_user::SqlUser;

#[async_trait]
pub trait DbConnectionLike: Interface + Send + Sync {
    async fn get_user_by_username(&self, username: &(impl AsRef<str> + Sync)) -> Option<SqlUser>;
    async fn get_user_by_id(&self, userid: i64) -> Result<Option<SqlUser>>;
    async fn insert_user(&self, user: SqlUser) -> Result<()>;
}

#[async_trait]
impl DbConnectionLike for PgPool {
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
            None
        }
    }

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
        .map_err(|e| anyhow!(e))
    }

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
        .bind(false)
        .bind(user.password_mcf)
        .bind(user.last_access)
        .execute(self)
        .await?;

        Ok(())
    }
}
