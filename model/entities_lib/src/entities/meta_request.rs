use crate::entities::audiobook_requests::{Limit, Page};
use crate::{Author, Category, Keyword, Reader, Series};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(tag = "req_type", content = "args")]
pub enum MetaRequest {
    CategoriesByPublishedAudiobooks(Page, Limit),
    KeywordsByPublishedAudiobooks(Page, Limit),
    AuthorsByPublishedAudiobooks(Page, Limit),
    ReaderByPublishedAudiobooks(Page, Limit),
    SeriesBySubscriber(Page, Limit),
    CategoriesByNSubscribers(Page, Limit),
    KeywordsByNSubscribers(Page, Limit),
    AuthorByNSubscribers(Page, Limit),
    ReaderByNSubscribers(Page, Limit),
    SeriesByNSubscribers(Page, Limit),
    CategoriesAlphabetically(Page, Limit),
    KeywordsAlphabetically(Page, Limit),
    AuthorsAlphabetically(Page, Limit),
    ReadersAlphabetically(Page, Limit),
    SeriesAlphabetically(Page, Limit),
    CountAllAudiobooks,
    CountAudiobooksForCategory(Category),
    CountAudiobooksForKeyword(Keyword),
    CountAudiobooksForAuthor(Author),
    CountAudiobooksForReader(Reader),
    CountAudiobooksInSeries(Series),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum MetaResponse {
    Categories(Vec<Category>),
    Keywords(Vec<Keyword>),
    Authors(Vec<Author>),
    Readers(Vec<Reader>),
    Series(Vec<Series>),
    Count(u32),
}
