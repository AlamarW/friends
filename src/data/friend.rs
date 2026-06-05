use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friend {
    #[serde(default = "new_id")]
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub extra: HashMap<String, HashMap<String, String>>,
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

impl Friend {
    pub fn new(name: String) -> Self {
        Self {
            id: new_id(),
            name,
            email: None,
            notes: None,
            extra: HashMap::new(),
        }
    }

    pub fn profile_for(&self, applet_key: &str) -> HashMap<String, String> {
        self.extra.get(applet_key).cloned().unwrap_or_default()
    }

    pub fn set_profile(&mut self, applet_key: &str, profile: HashMap<String, String>) {
        if profile.is_empty() {
            self.extra.remove(applet_key);
        } else {
            self.extra.insert(applet_key.to_string(), profile);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_friend_new_defaults() {
        let f = Friend::new("Alice".to_string());
        assert_eq!(f.name, "Alice");
        assert!(f.email.is_none());
        assert!(f.notes.is_none());
        assert!(f.extra.is_empty());
        assert!(!f.id.is_empty());
    }

    #[test]
    fn test_friend_new_generates_unique_ids() {
        let a = Friend::new("A".to_string());
        let b = Friend::new("B".to_string());
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn test_friend_toml_roundtrip_minimal() {
        let original = Friend::new("Bob".to_string());
        let toml = toml::to_string_pretty(&original).unwrap();
        let restored: Friend = toml::from_str(&toml).unwrap();
        assert_eq!(restored.name, "Bob");
        assert!(restored.email.is_none());
        assert!(restored.extra.is_empty());
    }

    #[test]
    fn test_friend_toml_roundtrip_with_extra() {
        let mut original = Friend::new("Alice".to_string());
        let mut job = HashMap::new();
        job.insert("desired_role".to_string(), "Engineer".to_string());
        job.insert("skills".to_string(), "Rust, Go".to_string());
        original.extra.insert("job_feed".to_string(), job);

        let toml = toml::to_string_pretty(&original).unwrap();
        let restored: Friend = toml::from_str(&toml).unwrap();

        let profile = restored.profile_for("job_feed");
        assert_eq!(profile.get("desired_role").map(|s| s.as_str()), Some("Engineer"));
        assert_eq!(profile.get("skills").map(|s| s.as_str()), Some("Rust, Go"));
    }

    #[test]
    fn test_profile_for_missing_key_returns_empty() {
        let f = Friend::new("Carol".to_string());
        assert!(f.profile_for("job_feed").is_empty());
    }

    #[test]
    fn test_set_profile_inserts() {
        let mut f = Friend::new("Dave".to_string());
        let mut profile = HashMap::new();
        profile.insert("interests".to_string(), "jazz".to_string());
        f.set_profile("wiki_prep", profile);
        assert_eq!(f.extra["wiki_prep"]["interests"], "jazz");
    }

    #[test]
    fn test_set_profile_empty_removes_key() {
        let mut f = Friend::new("Eve".to_string());
        let mut profile = HashMap::new();
        profile.insert("desired_role".to_string(), "Engineer".to_string());
        f.set_profile("job_feed", profile);
        assert!(f.extra.contains_key("job_feed"));
        f.set_profile("job_feed", HashMap::new());
        assert!(!f.extra.contains_key("job_feed"));
    }

    #[test]
    fn test_friend_deserializes_without_explicit_id() {
        let toml = r#"name = "Eve""#;
        let f: Friend = toml::from_str(toml).unwrap();
        assert_eq!(f.name, "Eve");
        assert!(!f.id.is_empty());
    }
}
