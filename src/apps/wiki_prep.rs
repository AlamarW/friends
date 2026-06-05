use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::{Applet, AppletItem, FieldDef, parse_csv};

pub struct WikiPrepApplet;

#[async_trait]
impl Applet for WikiPrepApplet {
    fn key(&self) -> &str { "wiki_prep" }
    fn name(&self) -> &str { "Conversation Prep" }

    fn fields(&self) -> Vec<FieldDef> {
        vec![
            FieldDef::optional("interests", "Interests", Some("comma-separated")),
        ]
    }

    fn is_unlocked(&self, profile: &HashMap<String, String>) -> bool {
        profile.get("interests").map(|s| !parse_csv(s).is_empty()).unwrap_or(false)
    }

    async fn fetch(&self, profile: &HashMap<String, String>) -> Result<Vec<AppletItem>> {
        let interests = profile.get("interests").map(|s| parse_csv(s)).unwrap_or_default();
        crate::api::wikipedia::fetch_summaries(&interests).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(interests: &str) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("interests".to_string(), interests.to_string());
        m
    }

    #[test]
    fn test_is_unlocked_with_interests() {
        assert!(WikiPrepApplet.is_unlocked(&profile("climbing, jazz")));
    }

    #[test]
    fn test_is_unlocked_empty_interests() {
        assert!(!WikiPrepApplet.is_unlocked(&profile("")));
    }

    #[test]
    fn test_is_unlocked_missing_key() {
        assert!(!WikiPrepApplet.is_unlocked(&HashMap::new()));
    }

    #[test]
    fn test_key_and_name() {
        assert_eq!(WikiPrepApplet.key(), "wiki_prep");
        assert_eq!(WikiPrepApplet.name(), "Conversation Prep");
    }
}
