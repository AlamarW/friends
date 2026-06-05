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
    pub interests: Vec<String>,
    pub job_hunt: Option<JobHuntProfile>,
    pub book_profile: Option<BookProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JobHuntProfile {
    pub desired_role: String,
    #[serde(default)]
    pub skills: Vec<String>,
    pub location: Option<String>,
    pub remote_preference: Option<String>,
    pub experience_level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookProfile {
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub authors: Vec<String>,
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
            interests: Vec::new(),
            job_hunt: None,
            book_profile: None,
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
        assert!(f.interests.is_empty());
        assert!(f.job_hunt.is_none());
        assert!(f.book_profile.is_none());
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
        assert!(restored.job_hunt.is_none());
    }

    #[test]
    fn test_friend_toml_roundtrip_with_job_hunt() {
        let mut original = Friend::new("Alice".to_string());
        original.job_hunt = Some(JobHuntProfile {
            desired_role: "Engineer".to_string(),
            skills: vec!["Rust".to_string()],
            location: Some("Austin".to_string()),
            remote_preference: Some("remote".to_string()),
            experience_level: Some("senior".to_string()),
        });
        let toml = toml::to_string_pretty(&original).unwrap();
        let restored: Friend = toml::from_str(&toml).unwrap();
        let jh = restored.job_hunt.unwrap();
        assert_eq!(jh.desired_role, "Engineer");
        assert_eq!(jh.skills, vec!["Rust"]);
        assert_eq!(jh.location.as_deref(), Some("Austin"));
    }

    #[test]
    fn test_friend_toml_roundtrip_with_book_profile() {
        let mut original = Friend::new("Carol".to_string());
        original.book_profile = Some(BookProfile {
            genres: vec!["sci-fi".to_string(), "history".to_string()],
            authors: vec!["Le Guin".to_string()],
        });
        let toml = toml::to_string_pretty(&original).unwrap();
        let restored: Friend = toml::from_str(&toml).unwrap();
        let bp = restored.book_profile.unwrap();
        assert_eq!(bp.genres, vec!["sci-fi", "history"]);
        assert_eq!(bp.authors, vec!["Le Guin"]);
    }

    #[test]
    fn test_friend_toml_roundtrip_with_interests() {
        let mut original = Friend::new("Dave".to_string());
        original.interests = vec!["climbing".to_string(), "jazz".to_string()];
        let toml = toml::to_string_pretty(&original).unwrap();
        let restored: Friend = toml::from_str(&toml).unwrap();
        assert_eq!(restored.interests, vec!["climbing", "jazz"]);
    }

    #[test]
    fn test_friend_deserializes_without_explicit_id() {
        let toml = r#"name = "Eve""#;
        let f: Friend = toml::from_str(toml).unwrap();
        assert_eq!(f.name, "Eve");
        assert!(!f.id.is_empty());
    }

    #[test]
    fn test_job_hunt_profile_default() {
        let jh = JobHuntProfile::default();
        assert!(jh.desired_role.is_empty());
        assert!(jh.skills.is_empty());
        assert!(jh.location.is_none());
    }

    #[test]
    fn test_book_profile_default() {
        let bp = BookProfile::default();
        assert!(bp.genres.is_empty());
        assert!(bp.authors.is_empty());
    }
}
