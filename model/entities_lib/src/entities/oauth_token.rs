#![allow(clippy::doc_markdown)]
#![allow(clippy::used_underscore_items)]

use super::enums::SourcePlatform;
use chrono::NaiveDateTime;

use uuid::Uuid;

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "oauth_tokens")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: SourcePlatform,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub scope: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {

    use crate::entities::enums::SourcePlatform;
    use crate::entities::{oauth_token, user};
    use sea_orm::prelude::*;
    use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, Set};
    use sea_orm::{DatabaseBackend, MockDatabase};

    #[tokio::test]
    async fn test_cascade_delete() -> Result<(), DbErr> {
        let user_id = Uuid::new_v4();
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![user::Model {
                id: user_id,
                username: "test_user".to_owned(),
                email: "test@example.com".to_owned(),
            }]])
            .append_query_results(
                // Mock oauth_token insertion result (can be empty vec, as we only care about it being executed)
                vec![vec![oauth_token::Model {
                    id: Uuid::new_v4(),
                    user_id,
                    provider: SourcePlatform::Github,
                    access_token: "token".to_string(),
                    refresh_token: None,
                    expires_at: None,
                    scope: None,
                }]],
            )
            .append_exec_results(vec![
                // Mock the user deletion result
                sea_orm::MockExecResult {
                    last_insert_id: 0,
                    rows_affected: 1,
                },
            ])
            .append_query_results(vec![
                // Mock the oauth tokens fetch result after user deletion (should be empty)
                Vec::<oauth_token::Model>::new(),
            ])
            .into_connection();

        // 1. Insert a user
        let user_model = user::ActiveModel {
            username: Set("test_user".to_owned()),
            email: Set("test@example.com".to_owned()),
            ..Default::default()
        };
        let inserted_user: user::Model = user_model.insert(&db).await?;

        // 2. Insert an OAuth token for the user.
        let token_model = oauth_token::ActiveModel {
            user_id: Set(inserted_user.id),
            provider: Set(SourcePlatform::Github),
            access_token: Set("token".to_owned()),
            ..Default::default()
        };

        token_model.insert(&db).await?;

        // 3. Delete the user.
        user::Entity::delete_by_id(inserted_user.id)
            .exec(&db)
            .await?;

        // 4. Verify that the OAuth tokens are also deleted.
        let remaining_tokens: Vec<oauth_token::Model> =
            oauth_token::Entity::find().all(&db).await?;
        assert!(
            remaining_tokens.is_empty(),
            "OAuth tokens should have been deleted"
        );

        Ok(())
    }
}
