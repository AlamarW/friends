pub mod book_recs;
pub mod job_feed;
pub mod wiki_prep;

use std::collections::HashMap;

use anyhow::Result;

use crate::data::friend::Friend;

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub key: &'static str,
    pub label: &'static str,
    pub hint: Option<&'static str>,
}

impl FieldDef {
    pub fn required(key: &'static str, label: &'static str) -> Self {
        Self { key, label, hint: None }
    }
    pub fn optional(key: &'static str, label: &'static str, hint: Option<&'static str>) -> Self {
        Self { key, label, hint }
    }
}

#[derive(Debug, Clone)]
pub struct AppletItem {
    pub title: String,
    pub subtitle: Option<String>,
    pub url: Option<String>,
}

#[async_trait::async_trait]
pub trait Applet: Send + Sync {
    fn key(&self) -> &str;
    fn name(&self) -> &str;
    fn fields(&self) -> Vec<FieldDef>;
    fn is_unlocked(&self, profile: &HashMap<String, String>) -> bool;
    async fn fetch(&self, profile: &HashMap<String, String>) -> Result<Vec<AppletItem>>;
}

pub struct AppletRegistry {
    applets: Vec<Box<dyn Applet>>,
}

impl AppletRegistry {
    pub fn new() -> Self {
        Self { applets: vec![] }
    }

    pub fn register(&mut self, applet: Box<dyn Applet>) {
        self.applets.push(applet);
    }

    pub fn available_for(&self, friend: &Friend) -> Vec<&dyn Applet> {
        self.applets
            .iter()
            .filter(|a| a.is_unlocked(&friend.profile_for(a.key())))
            .map(|a| a.as_ref())
            .collect()
    }

    pub fn by_key(&self, key: &str) -> Option<&dyn Applet> {
        self.applets.iter().find(|a| a.key() == key).map(|a| a.as_ref())
    }

    pub fn all(&self) -> &[Box<dyn Applet>] {
        &self.applets
    }
}

pub fn build_registry() -> AppletRegistry {
    let mut r = AppletRegistry::new();
    r.register(Box::new(job_feed::JobFeedApplet));
    r.register(Box::new(book_recs::BookRecsApplet));
    r.register(Box::new(wiki_prep::WikiPrepApplet));
    r
}

pub(crate) fn parse_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> AppletRegistry {
        build_registry()
    }

    fn friend_with(applet_key: &str, field: &str, value: &str) -> Friend {
        let mut f = Friend::new("Alice".to_string());
        let mut profile = HashMap::new();
        profile.insert(field.to_string(), value.to_string());
        f.extra.insert(applet_key.to_string(), profile);
        f
    }

    #[test]
    fn test_registry_available_for_empty_friend() {
        let r = registry();
        assert!(r.available_for(&Friend::new("Alice".to_string())).is_empty());
    }

    #[test]
    fn test_registry_job_feed_unlocks() {
        let r = registry();
        let f = friend_with("job_feed", "desired_role", "Engineer");
        let applets = r.available_for(&f);
        assert!(applets.iter().any(|a| a.key() == "job_feed"));
    }

    #[test]
    fn test_registry_job_feed_requires_nonempty_role() {
        let r = registry();
        let f = friend_with("job_feed", "desired_role", "");
        assert!(r.available_for(&f).is_empty());
    }

    #[test]
    fn test_registry_book_recs_unlocks_with_genres() {
        let r = registry();
        let f = friend_with("book_recs", "genres", "sci-fi");
        assert!(r.available_for(&f).iter().any(|a| a.key() == "book_recs"));
    }

    #[test]
    fn test_registry_book_recs_unlocks_with_authors() {
        let r = registry();
        let f = friend_with("book_recs", "authors", "Le Guin");
        assert!(r.available_for(&f).iter().any(|a| a.key() == "book_recs"));
    }

    #[test]
    fn test_registry_wiki_prep_unlocks_with_interests() {
        let r = registry();
        let f = friend_with("wiki_prep", "interests", "climbing");
        assert!(r.available_for(&f).iter().any(|a| a.key() == "wiki_prep"));
    }

    #[test]
    fn test_registry_all_three_unlock_together() {
        let r = registry();
        let mut f = Friend::new("Alice".to_string());
        f.extra.insert("job_feed".to_string(), [("desired_role".to_string(), "Eng".to_string())].into());
        f.extra.insert("book_recs".to_string(), [("genres".to_string(), "sci-fi".to_string())].into());
        f.extra.insert("wiki_prep".to_string(), [("interests".to_string(), "jazz".to_string())].into());
        assert_eq!(r.available_for(&f).len(), 3);
    }

    #[test]
    fn test_registry_by_key_found() {
        let r = registry();
        assert!(r.by_key("job_feed").is_some());
        assert!(r.by_key("book_recs").is_some());
        assert!(r.by_key("wiki_prep").is_some());
    }

    #[test]
    fn test_registry_by_key_not_found() {
        let r = registry();
        assert!(r.by_key("nonexistent").is_none());
    }

    #[test]
    fn test_applet_names() {
        let r = registry();
        let names: Vec<&str> = r.all().iter().map(|a| a.name()).collect();
        assert!(names.contains(&"Job Feed"));
        assert!(names.contains(&"Book Recs"));
        assert!(names.contains(&"Conversation Prep"));
    }

    #[test]
    fn test_parse_csv_basic() {
        assert_eq!(parse_csv("rust, go, python"), vec!["rust", "go", "python"]);
    }

    #[test]
    fn test_parse_csv_empty() {
        assert!(parse_csv("").is_empty());
        assert!(parse_csv("  ,  ").is_empty());
    }

    #[test]
    fn test_parse_csv_single_entry() {
        assert_eq!(parse_csv("rust"), vec!["rust"]);
        assert_eq!(parse_csv("  rust  "), vec!["rust"]);
    }

    #[test]
    fn test_parse_csv_trailing_comma() {
        assert_eq!(parse_csv("rust, go,"), vec!["rust", "go"]);
    }

    #[test]
    fn test_parse_csv_consecutive_commas() {
        assert_eq!(parse_csv("rust,,go"), vec!["rust", "go"]);
    }
}
