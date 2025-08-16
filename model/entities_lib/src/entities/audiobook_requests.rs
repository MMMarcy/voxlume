use serde::{Deserialize, Serialize};

use crate::{Author, Category, Keyword, Reader, Series};

pub type Page = u16;
pub type Limit = u16;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum GetAudioBookRequestType {
    MostRecent(Page),
    ByAuthor(Author, Page),
    ByReader(Reader, Page),
    ByCategory(Category, Page),
    ByKeyword(Keyword, Page),
    BySeries(Series, Page),
}
