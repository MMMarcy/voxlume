use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_trait::async_trait;
use axum_session_auth::{Authentication, HasPermission};
use axum_session_sqlx::SessionPgPool;
use chrono;
use entities_lib::entities::user::User;
use log::{error, info};
use sqlx::{FromRow, PgPool};

#[derive(FromRow, Clone, Debug)]
pub struct SqlUser {
    pub id: i64,
    pub username: String,
    pub anonymous: bool,
    pub password_mcf: String,
    pub last_access: chrono::DateTime<chrono::Utc>,
}

pub type AuthSession = axum_session_auth::AuthSession<SqlUser, i64, SessionPgPool, PgPool>;

#[async_trait]
impl Authentication<SqlUser, i64, PgPool> for SqlUser {
    // This is run when the user has logged in and has not yet been Cached in the system.
    // Once ran it will load and cache the user.
    async fn load_user(userid: i64, pool: Option<&PgPool>) -> Result<SqlUser> {
        if userid == 1 {
            return Ok(SqlUser {
                id: userid,
                username: "Guest".to_string(),
                anonymous: true,
                password_mcf: "".to_string(),
                last_access: chrono::Utc::now(),
            });
        }
        let db = pool.unwrap();
        let get_user_result = sqlx::query_as::<_, SqlUser>(
            r#"
            SELECT *
            FROM users
            WHERE id = $1
        "#,
        )
        .bind(userid)
        .fetch_optional(db)
        .await?;

        match get_user_result {
            Some(user) => Ok(user),
            None => Err(anyhow!("No user found")),
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
        argon2_params: Argon2<'_>,
    ) -> SqlUser {
        let salt = SaltString::generate(&mut OsRng);
        // TODO: Check if the following expect can fail.
        let password_hash = argon2_params
            .hash_password(password.as_bytes(), &salt)
            .expect("Couldn't hash password")
            .to_string();

        return SqlUser {
            id: -1,
            username,
            anonymous: false,
            password_mcf: password_hash,
            last_access: chrono::Utc::now(),
        };
    }

    async fn get_user_from_username(
        username: String,
        db_connection_pool: &PgPool,
    ) -> Option<SqlUser> {
        let get_user_result = sqlx::query_as::<_, SqlUser>(
            r#"
            SELECT *
            FROM users
            WHERE username = $1
        "#,
        )
        .bind(username)
        .fetch_optional(db_connection_pool)
        .await;

        if let Ok(Some(user)) = get_user_result {
            Some(user)
        } else {
            None
        }
    }

    pub async fn login_user(
        username: String,
        password: String,
        db_connection_pool: &PgPool,
        argon2_params: Argon2<'_>,
    ) -> Result<SqlUser> {
        let maybe_user = Self::get_user_from_username(username, db_connection_pool).await;

        if let Some(user) = maybe_user {
            // TODO: Fix this with a match.
            let parsed_hash =
                PasswordHash::new(&user.password_mcf).expect("Couldn't parse stored hash");
            match argon2_params.verify_password(password.as_bytes(), &parsed_hash) {
                Ok(_) => return Ok(user),
                Err(_) => {
                    error!("Password's don't match");
                    return Err(anyhow!(String::from("Password's don't match")));
                }
            }
        }
        return Err(anyhow!(String::from("User not found")));
    }

    pub async fn register_user(self: Self, db_connection_pool: &PgPool) -> Result<SqlUser> {
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
        .bind(self.username.clone())
        .bind(false)
        .bind(self.password_mcf.clone())
        .bind(self.last_access.clone())
        .execute(db_connection_pool)
        .await?;

        Ok(
            Self::get_user_from_username(self.username, db_connection_pool)
                .await
                .unwrap(),
        )
    }
}
