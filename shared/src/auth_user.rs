use anyhow::{anyhow, Result};
use async_trait::async_trait;
use axum_session_auth::{Authentication, HasPermission};
use axum_session_sqlx::SessionPgPool;
use chrono;
use entities_lib::entities::user::User;
use sqlx::PgPool;
use tracing::{error, info, instrument, span, Level};

use crate::{db_trait::DbConnectionLike, password_handler::PasswordHandlerLike, sql_user::SqlUser};

pub type AuthSession = axum_session_auth::AuthSession<SqlUser, i64, SessionPgPool, PgPool>;

#[async_trait]
impl Authentication<SqlUser, i64, PgPool> for SqlUser {
    // This is run when the user has logged in and has not yet been Cached in the system.
    // Once ran it will load and cache the user.
    #[instrument(skip_all)]
    async fn load_user(userid: i64, pool: Option<&PgPool>) -> Result<SqlUser> {
        let span = span!(Level::TRACE, "load_user");
        let _guard = span.enter();
        let db_connection = pool.ok_or(anyhow!("Connection pool not available".to_string()))?;

        match db_connection.get_user_by_id(userid).await? {
            Some(user) => {
                info!("User (id:{}) loaded", userid);
                Ok(user)
            }
            None => {
                error!(
                    "User (id:{}) couldn't be loaded as it wasn't found in the db.",
                    userid
                );
                Err(anyhow!("No user found"))
            }
        }
    }

    // This function is used internally to determine if they are logged in or not.
    fn is_authenticated(&self) -> bool {
        !self.anonymous
    }

    fn is_active(&self) -> bool {
        !self.anonymous
    }

    fn is_anonymous(&self) -> bool {
        self.anonymous
    }
}

#[async_trait]
impl HasPermission<PgPool> for SqlUser {
    async fn has(&self, _perm: &str, _pool: &Option<&PgPool>) -> bool {
        true
    }
}

impl SqlUser {
    pub fn into_user(self) -> User {
        return User {
            id: self.id,
            username: self.username.into(),
            last_access: self.last_access.timestamp(),
        };
    }

    pub async fn create_local_user(
        username: String,
        password: String,
        password_handler: &impl PasswordHandlerLike,
    ) -> SqlUser {
        let password_hash = password_handler
            .generate_password_hash(&password)
            .expect("It shouldn't be possible for this to fail :/");
        return SqlUser {
            id: -1,
            username,
            anonymous: false,
            password_mcf: password_hash,
            last_access: chrono::Utc::now(),
        };
    }

    #[instrument(skip_all)]
    pub async fn login_user(
        username: String,
        password: String,
        db_connection_pool: &impl DbConnectionLike,
        password_handler: &impl PasswordHandlerLike,
    ) -> Result<SqlUser> {
        let maybe_user = db_connection_pool.get_user_by_username(&username).await;

        match maybe_user {
            Some(user) => password_handler
                .validate_password_against_mcf(&user.password_mcf, &password)
                .map(|_| user),
            None => Err(anyhow!(String::from("User not found"))),
        }
    }

    #[instrument(skip_all)]
    pub async fn register_user(self: Self, db_connection_pool: &PgPool) -> Result<SqlUser> {
        let username = &self.username.clone();
        db_connection_pool.insert_user(self).await?;
        match db_connection_pool.get_user_by_username(username).await {
            Some(user) => Ok(user),
            None => Err(anyhow!("Problems in inserting user into the db".to_string())),
        }
    }
}
