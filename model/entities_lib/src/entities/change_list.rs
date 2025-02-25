#![allow(clippy::doc_markdown)]
#![allow(clippy::used_underscore_items)]

//! Module defining a change list.
//!
//! This is inspired from Google's lingo and in ChIRP's case it stands
//! for a GitHub's pull request or a GitLab's merge request.
//!
//! Therefore, this is an umbrella term that encompasses an atomic change that
//! the creator is trying to submit.

use super::enums::{ChangeStatus, SourcePlatform};
use cfg_if::cfg_if;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
cfg_if! { if #[cfg(feature = "ssr")]{

    use sea_orm::entity::prelude::*;

    /// Represents a change list item as a SeaORM entity.
    ///
    /// # Fields
    ///
    /// * `id`: The unique identifier for the change list item (UUID).
    /// * `creation_time`: The timestamp when the change list item was created.
    /// * `last_update_time`: The timestamp when the change list item was last updated.
    /// * `source_platform`: The platform where the change originated.
    /// * `change_status`: The current status of the change.
    ///
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
    #[sea_orm(table_name = "change_list")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        /// The unique identifier for the change list item.
        pub id: Uuid,
        /// The timestamp when the change list item was created.
        pub creation_time: NaiveDateTime,
        /// The timestamp when the change list item was last updated.
        pub last_update_time: NaiveDateTime,
        /// The platform where the change originated.
        pub source_platform: SourcePlatform,
        /// The current status of the change.
        pub change_status: ChangeStatus
    }


    /// Defines the relations for the `ChangeList` entity.
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}

    impl From<Model> for _ChangeList {
       /// Converts a `Model` into a `_ChangeList`.
       ///
       /// This allows for easy conversion between the SeaORM entity and the internal
       /// representation of a change list item.
       ///
       /// # Arguments
       ///
       /// * `val`: The `Model` to convert.
       ///
       /// # Returns
       ///
       /// A new `_ChangeList` with the same data as the `Model`.
       fn from(val: Model) -> Self {
            _ChangeList {
                id: val.id,
                creation_time: val.creation_time,
                last_update_time: val.last_update_time,
                source_platform: val.source_platform,
                change_status: val.change_status,
            }
        }
    }

}}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct _ChangeList {
    pub id: Uuid,
    pub creation_time: NaiveDateTime,
    pub last_update_time: NaiveDateTime,
    pub source_platform: SourcePlatform,
    pub change_status: ChangeStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn test_model_to_changelist_conversion() {
        let id = Uuid::new_v4();
        let creation_time = DateTime::from_timestamp(61, 0).unwrap();
        let last_update_time = DateTime::from_timestamp(122, 0).unwrap();
        let source_platform = SourcePlatform::Github;
        let change_status = ChangeStatus::Open;

        let model = Model {
            id,
            creation_time: creation_time.naive_utc(),
            last_update_time: last_update_time.naive_utc(),
            source_platform,
            change_status,
        };

        let change_list = _ChangeList::from(model.clone());

        assert_eq!(change_list.id, model.id);
        assert_eq!(change_list.creation_time, model.creation_time);
        assert_eq!(change_list.last_update_time, model.last_update_time);
        assert_eq!(change_list.source_platform, model.source_platform);
        assert_eq!(change_list.change_status, model.change_status);
    }
}
