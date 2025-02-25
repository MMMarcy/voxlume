use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use sea_orm::prelude::*;

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "ssr", derive(DeriveIden, EnumIter, DeriveActiveEnum))]
#[cfg_attr(
    feature = "ssr",
    sea_orm(rs_type = "String", db_type = "Enum", enum_name = "change_status")
)]
pub enum ChangeStatus {
    #[cfg_attr(feature = "ssr", sea_orm(string_value = "unknown"))]
    Unknown = 0,

    #[cfg_attr(feature = "ssr", sea_orm(string_value = "open"))]
    Open = 1,

    #[cfg_attr(feature = "ssr", sea_orm(string_value = "approved"))]
    Approved = 2,

    #[cfg_attr(feature = "ssr", sea_orm(string_value = "merged"))]
    Merged = 3,

    #[cfg_attr(feature = "ssr", sea_orm(string_value = "draft"))]
    Draft = 4,

    #[cfg_attr(feature = "ssr", sea_orm(string_value = "archived"))]
    Archived = 5,
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "ssr", derive(DeriveIden, EnumIter, DeriveActiveEnum))]
#[cfg_attr(
    feature = "ssr",
    sea_orm(rs_type = "String", db_type = "Enum", enum_name = "change_status")
)]
pub enum SourcePlatform {
    #[cfg_attr(feature = "ssr", sea_orm(string_value = "unknown"))]
    Unknown = 0,

    #[cfg_attr(feature = "ssr", sea_orm(string_value = "github"))]
    Github = 1,

    #[cfg_attr(feature = "ssr", sea_orm(string_value = "gitlab"))]
    Gitlab = 2,
}

#[derive(Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "ssr", derive(DeriveIden, EnumIter, DeriveActiveEnum))]
#[cfg_attr(
    feature = "ssr",
    sea_orm(rs_type = "String", db_type = "Enum", enum_name = "language")
)]
pub enum Language {
    #[cfg_attr(feature = "ssr", sea_orm(string_value = "unknown"))]
    Unknown = 0,

    #[cfg_attr(feature = "ssr", sea_orm(string_value = "english"))]
    English = 1,

    #[cfg_attr(feature = "ssr", sea_orm(string_value = "italian"))]
    Italian = 2,
}
