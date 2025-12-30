use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::{Author, Category, Keyword, Reader, Series};

pub type Page = u32;
pub type Limit = u32;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum GetAudioBookRequestType {
    MostRecent(Page),
    ByAuthor(Author, Page),
    ByReader(Reader, Page),
    ByCategory(Category, Page),
    ByKeyword(Keyword, Page),
    BySeries(Series, Page),
    ById(String),
    ByIdList(Vec<i64>),
    AllExcept(Vec<Category>, Vec<Keyword>),
}
