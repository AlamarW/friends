use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::{Applet, AppletItem, FieldDef, parse_csv};

pub struct BookRecsApplet;

#[async_trait]
impl Applet for BookRecsApplet {
    fn key(&self) -> &str { "book_recs" }
    fn name(&self) -> &str { "Book Recs" }

    fn fields(&self) -> Vec<FieldDef> {
        vec![
            FieldDef::optional("genres", "Genres", Some("comma-separated")),
            FieldDef::optional("authors", "Authors", Some("comma-separated")),
        ]
    }

    fn is_unlocked(&self, profile: &HashMap<String, String>) -> bool {
        let has_genres = profile.get("genres").map(|s| !parse_csv(s).is_empty()).unwrap_or(false);
        let has_authors = profile.get("authors").map(|s| !parse_csv(s).is_empty()).unwrap_or(false);
        has_genres || has_authors
    }

    async fn fetch(&self, profile: &HashMap<String, String>) -> Result<Vec<AppletItem>> {
        let genres = profile.get("genres").map(|s| parse_csv(s)).unwrap_or_default();
        let authors = profile.get("authors").map(|s| parse_csv(s)).unwrap_or_default();
        crate::api::open_library::fetch_books(&genres, &authors).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(genres: &str, authors: &str) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("genres".to_string(), genres.to_string());
        m.insert("authors".to_string(), authors.to_string());
        m
    }

    #[test]
    fn test_is_unlocked_with_genres() {
        assert!(BookRecsApplet.is_unlocked(&profile("sci-fi", "")));
    }

    #[test]
    fn test_is_unlocked_with_authors_only() {
        assert!(BookRecsApplet.is_unlocked(&profile("", "Le Guin")));
    }

    #[test]
    fn test_is_unlocked_both_empty() {
        assert!(!BookRecsApplet.is_unlocked(&profile("", "")));
    }

    #[test]
    fn test_is_unlocked_missing_keys() {
        assert!(!BookRecsApplet.is_unlocked(&HashMap::new()));
    }

    #[test]
    fn test_key_and_name() {
        assert_eq!(BookRecsApplet.key(), "book_recs");
        assert_eq!(BookRecsApplet.name(), "Book Recs");
    }
}
