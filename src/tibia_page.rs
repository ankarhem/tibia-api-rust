use scraper::{ElementRef, Selector};

use crate::{Result, ServerError};

pub struct TibiaPage {
    document: scraper::Html,
}

impl TibiaPage {
    pub fn new(page: &str) -> Self {
        Self {
            document: scraper::Html::parse_document(page),
        }
    }

    pub fn get_main_content(&self) -> Result<ElementRef> {
        let selector = Selector::parse(".main-content").expect("Selector to be valid");

        let element_ref = self
            .document
            .select(&selector)
            .next()
            .ok_or(ServerError::ScrapeUnexpectedPageContent);

        element_ref
    }

    pub fn get_tables(&self) -> Result<Vec<ElementRef>> {
        let table_selector =
            Selector::parse(".TableContainer table").expect("Selector to be valid");
        let tables: Vec<ElementRef> = self.get_main_content()?.select(&table_selector).collect();
        Ok(tables)
    }
}

pub fn sanitize_string(page: &str) -> String {
    let sanitized = page
        .trim()
        .replace("\\n", "")
        .replace("\\\"", "'")
        .replace("\\u00A0", " ")
        .replace("\\u0026", "&")
        .replace("\\u0026#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&amp;", "&");

    sanitized
}
