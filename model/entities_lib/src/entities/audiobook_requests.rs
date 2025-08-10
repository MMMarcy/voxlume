use serde::{Deserialize, Serialize};

use crate::{Author, Category, Keyword, Reader, Series};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum GetAudioBookRequestType {
    MostRecent,
    ByAuthor(Author),
    ByReader(Reader),
    ByCategory(Category),
    ByKeyword(Keyword),
    BySeries(Series),
}
