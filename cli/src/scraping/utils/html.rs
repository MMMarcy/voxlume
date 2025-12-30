use scraper::{Html, Selector};

/// Returns the outer HTML of the first element matching ".post".
/// Returns `None` if no such element is found.
/// # Panics
///   - If the post selector fails.
#[must_use]
pub fn extract_only_post_info(body: &str) -> Option<String> {
    // When you're parsing a snippet and not a full HTML document,
    // `parse_fragment` is more appropriate.
    let document = Html::parse_fragment(body);

    // Create a selector. We use .expect() here for simplicity, assuming the selector is always valid.
    // In production, you might want to handle the Result of Selector::parse more gracefully.
    let post_selector = Selector::parse(".post").expect("Invalid CSS selector for .post");

    // document.select(...) returns an iterator.
    // .next() attempts to get the first item from the iterator.
    // This returns an Option<ElementRef>, which is Some(element) if found, or None if not.
    // We can use .map() to transform the Some(element) into Some(element.html()).
    // If it's None, it remains None.
    document
        .select(&post_selector)
        .next()
        .map(|element| element.html())
}

/// Returns the outer HTML of the first element matching `main_table`.
/// Returns `None` if no such element is found.
/// # Panics
///   - If the main table is not found.
#[must_use]
pub fn extract_only_new_submissions_table(body: &str) -> Option<String> {
    let document = Html::parse_fragment(body);
    let table_selector =
        Selector::parse(".main_table").expect("Invalid CSS selector for .main_table");

    document
        .select(&table_selector)
        .next()
        .map(|element| element.html())
}
