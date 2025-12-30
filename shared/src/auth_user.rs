use anyhow::{Result, anyhow};
use async_trait::async_trait;
use axum_session_auth::{Authentication, HasPermission};
use axum_session_sqlx::SessionPgPool;
use chrono;
use entities_lib::entities::user::User;
use sqlx::PgPool;
use tracing::{Level, error, info, instrument, span};

use crate::db_trait::DbConnectionLike;
use crate::password_handler::PasswordHandlerLike;
use crate::sql_user::SqlUser;

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

        if let Some(user) = db_connection.get_user_by_id(userid).await? {
            info!("User (id:{}) loaded", userid);
            Ok(user)
        } else {
            error!(
                "User (id:{}) couldn't be loaded as it wasn't found in the db.",
                userid
            );
            Err(anyhow!("No user found"))
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
    #[allow(clippy::ref_option_ref)]
    async fn has(&self, _perm: &str, _pool: &Option<&PgPool>) -> bool {
        true
    }
}

impl SqlUser {
    /// .
    #[must_use]
    pub fn into_user(self) -> User {
        User {
            id: self.id,
            username: self.username,
            last_access: self.last_access.timestamp(),
        }
    }

    /// Creates as local user that will be then stored into the users table.
    ///
    /// # Panics
    ///
    /// This function shouldn't panic as any valid String reference can be hashed.
    /// If this is not the case, I was wrong.
    pub fn create_local_user(
        username: String,
        password: &String,
        password_handler: &impl PasswordHandlerLike,
    ) -> SqlUser {
        let password_hash = password_handler
            .generate_password_hash(password)
            .expect("It shouldn't be possible for this to fail :/");
        SqlUser {
            id: -1,
            username,
            anonymous: false,
            password_mcf: password_hash,
            last_access: chrono::Utc::now(),
        }
    }

    /// Logins a user.
    ///
    /// # Errors
    /// Either fir the user wasn't found or there was a technical problem with the user lookup.
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
                .map(|()| user),
            None => Err(anyhow!(String::from("User not found"))),
        }
    }

    /// Registers a user.
    ///
    /// The function first inserts a user into the DB and retrieves it so that we can
    /// propagate the user id created by postgres.
    ///
    /// # Errors
    /// If any of the insert/retrieve operations fail.
    #[instrument(skip_all)]
    pub async fn register_user(
        self,
        db_connection_pool: &impl DbConnectionLike,
    ) -> Result<SqlUser> {
        let username = &self.username.clone();
        db_connection_pool.insert_user(self).await?;
        match db_connection_pool.get_user_by_username(username).await {
            Some(user) => Ok(user),
            None => Err(anyhow!(
                "Problems in inserting user into the db".to_string()
            )),
        }
    }
}
