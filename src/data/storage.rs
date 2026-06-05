use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::friend::Friend;

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct FriendsFile {
    friends: Vec<Friend>,
}

pub fn data_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("friends")
        .join("friends.toml")
}

pub fn load_friends() -> Result<Vec<Friend>> {
    load_from(&data_path())
}

pub fn save_friends(friends: &[Friend]) -> Result<()> {
    save_to(friends, &data_path())
}

pub fn load_from(path: &Path) -> Result<Vec<Friend>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let file: FriendsFile = toml::from_str(&content)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(file.friends)
}

pub fn save_to(friends: &[Friend], path: &Path) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let file = FriendsFile {
        friends: friends.to_vec(),
    };
    let content = toml::to_string_pretty(&file)?;
    fs::write(path, content)
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::friend::{BookProfile, Friend, JobHuntProfile};
    use tempfile::TempDir;

    fn temp_path(dir: &TempDir) -> PathBuf {
        dir.path().join("friends.toml")
    }

    fn make_full_friend() -> Friend {
        Friend {
            id: "test-id-1".to_string(),
            name: "Alice".to_string(),
            email: Some("alice@example.com".to_string()),
            notes: Some("met at conference".to_string()),
            interests: vec!["Rust".to_string(), "climbing".to_string()],
            job_hunt: Some(JobHuntProfile {
                desired_role: "Software Engineer".to_string(),
                skills: vec!["Rust".to_string(), "Go".to_string()],
                location: Some("Austin, TX".to_string()),
                remote_preference: Some("remote".to_string()),
                experience_level: Some("senior".to_string()),
            }),
            book_profile: Some(BookProfile {
                genres: vec!["sci-fi".to_string()],
                authors: vec!["Le Guin".to_string()],
            }),
        }
    }

    #[test]
    fn test_load_from_nonexistent_returns_empty() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.toml");
        let result = load_from(&path).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_save_to_and_load_from_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = temp_path(&dir);
        let friends = vec![make_full_friend()];

        save_to(&friends, &path).unwrap();
        let loaded = load_from(&path).unwrap();

        assert_eq!(loaded.len(), 1);
        let f = &loaded[0];
        assert_eq!(f.name, "Alice");
        assert_eq!(f.email.as_deref(), Some("alice@example.com"));
        assert_eq!(f.notes.as_deref(), Some("met at conference"));
        assert_eq!(f.interests, vec!["Rust", "climbing"]);

        let jh = f.job_hunt.as_ref().unwrap();
        assert_eq!(jh.desired_role, "Software Engineer");
        assert_eq!(jh.skills, vec!["Rust", "Go"]);
        assert_eq!(jh.location.as_deref(), Some("Austin, TX"));
        assert_eq!(jh.remote_preference.as_deref(), Some("remote"));
        assert_eq!(jh.experience_level.as_deref(), Some("senior"));

        let bp = f.book_profile.as_ref().unwrap();
        assert_eq!(bp.genres, vec!["sci-fi"]);
        assert_eq!(bp.authors, vec!["Le Guin"]);
    }

    #[test]
    fn test_save_to_and_load_from_empty_list() {
        let dir = TempDir::new().unwrap();
        let path = temp_path(&dir);

        save_to(&[], &path).unwrap();
        let loaded = load_from(&path).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_save_to_creates_parent_directories() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nested").join("deep").join("friends.toml");

        save_to(&[], &path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_load_from_invalid_toml_returns_error() {
        let dir = TempDir::new().unwrap();
        let path = temp_path(&dir);
        fs::write(&path, "this is not valid toml [[[[").unwrap();

        let result = load_from(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load_multiple_friends() {
        let dir = TempDir::new().unwrap();
        let path = temp_path(&dir);
        let mut f2 = Friend::new("Bob".to_string());
        f2.email = Some("bob@example.com".to_string());

        save_to(&[make_full_friend(), f2], &path).unwrap();
        let loaded = load_from(&path).unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "Alice");
        assert_eq!(loaded[1].name, "Bob");
        assert!(loaded[1].job_hunt.is_none());
    }

    #[test]
    fn test_roundtrip_minimal_friend() {
        let dir = TempDir::new().unwrap();
        let path = temp_path(&dir);
        let friend = Friend::new("Charlie".to_string());

        save_to(&[friend], &path).unwrap();
        let loaded = load_from(&path).unwrap();

        assert_eq!(loaded[0].name, "Charlie");
        assert!(loaded[0].email.is_none());
        assert!(loaded[0].job_hunt.is_none());
        assert!(loaded[0].book_profile.is_none());
        assert!(loaded[0].interests.is_empty());
    }

    #[test]
    fn test_data_path_contains_friends_toml() {
        let path = data_path();
        let s = path.to_string_lossy();
        assert!(s.contains("friends.toml"));
        assert!(s.contains("friends"));
    }
}
