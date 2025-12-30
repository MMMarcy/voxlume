use serde::{Deserialize, Serialize};

use crate::{Author, Category, Keyword, Reader, Series};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubscriptionType {
    ToAuthor(Author),
    ToReader(Reader),
    ToSeries(Series),
    ToCategory(Category),
    ToKeyword(Keyword),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionExists {
    Unknown,
    Yes,
    No,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Subscription {
    pub user_id: i64,
    pub subscription_type: SubscriptionType,
}

impl Subscription {
    pub fn render_name(&self) -> String {
        let mut s = String::new();
        match &self.subscription_type {
            SubscriptionType::ToAuthor(author) => {
                s.push_str(&author.name);
            }
            SubscriptionType::ToReader(reader) => {
                s.push_str(&reader.name);
            }
            SubscriptionType::ToSeries(series) => {
                s.push_str(&series.title);
            }
            SubscriptionType::ToCategory(category) => s.push_str(&category.value),
            SubscriptionType::ToKeyword(keyword) => s.push_str(&keyword.value),
        }
        s
    }

    pub fn render_type(&self) -> String {
        match &self.subscription_type {
            SubscriptionType::ToAuthor(_) => String::from("Author"),
            SubscriptionType::ToReader(_) => String::from("Reader"),
            SubscriptionType::ToSeries(_) => String::from("Series"),
            SubscriptionType::ToCategory(_) => String::from("Category"),
            SubscriptionType::ToKeyword(_) => String::from("Keyword"),
        }
    }
}
