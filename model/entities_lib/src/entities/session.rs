// use super::enums::{AgeGroup, ArtLiteracy, Language};
// use cfg_if::cfg_if;
// use chrono::NaiveDateTime as DateTime;
// use serde::{Deserialize, Serialize};
// use uuid::Uuid;
//
// cfg_if! { if #[cfg(feature = "ssr")] {
//
//     use sea_orm::entity::prelude::*;
//
//     #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
//     #[sea_orm(table_name = "session")]
//     pub struct Model {
//         #[cfg_attr(feature = "ssr", sea_orm(primary_key, auto_increment = false))]
//         pub id: Uuid,
//         pub paid: bool,
//         pub start_time: DateTime,
//         pub end_time: DateTime,
//         pub art_literacy: ArtLiteracy,
//         pub age_group: AgeGroup,
//         pub language: Language
//     }
//
//     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//     pub enum Relation {}
//
//     impl ActiveModelBehavior for ActiveModel {}
//
//     impl From<Model> for _SessionFE {
//        fn from(val: Model) -> Self {
//             _SessionFE {
//                 id: val.id,
//                 paid: val.paid,
//                 start_time: val.start_time,
//                 end_time: val.end_time,
//                 language: val.language,
//                 art_literacy: val.art_literacy,
//                 age_group: val.age_group
//             }
//         }
//     }
// }}
//
// #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
// pub struct _SessionFE {
//     pub id: Uuid,
//     pub paid: bool,
//     pub start_time: DateTime,
//     pub end_time: DateTime,
//     pub art_literacy: ArtLiteracy,
//     pub age_group: AgeGroup,
//     pub language: Language,
// }
//
// #[cfg(test)]
// mod model_conversion_tests {
//     use super::*;
//     use chrono::{Duration, Local};
//     use googletest::prelude::*;
//     use test_log::test;
//
//     #[test]
//     fn test_model_conversion() {
//         let now = Local::now();
//         let model = Model {
//             id: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
//             paid: true,
//             start_time: DateTime::new(now.date_naive(), now.time()),
//             end_time: DateTime::new(now.date_naive(), now.time()) + Duration::hours(1),
//             art_literacy: ArtLiteracy::Noob,
//             age_group: AgeGroup::Adult,
//             language: Language::Italian,
//         };
//
//         let session_fe = _SessionFE::from(model);
//         let constructed_instance = _SessionFE {
//             id: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
//             paid: true,
//             start_time: DateTime::new(now.date_naive(), now.time()),
//             end_time: DateTime::new(now.date_naive(), now.time()) + Duration::hours(1),
//             art_literacy: ArtLiteracy::Noob,
//             age_group: AgeGroup::Adult,
//             language: Language::Italian,
//         };
//
//         assert_that!(session_fe, eq(&constructed_instance));
//     }
// }
