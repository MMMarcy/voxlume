use sqlx::FromRow;

#[derive(FromRow, Clone, Debug)]
pub struct SqlUser {
    pub id: i64,
    pub username: String,
    pub anonymous: bool,
    pub password_mcf: String,
    pub last_access: chrono::DateTime<chrono::Utc>,
}
