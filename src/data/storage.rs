use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use super::friend::Friend;

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct FriendsFile {
    friends: Vec<Friend>,
}

pub fn data_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".friends")
        .join("friends.toml")
}

pub fn load_friends() -> Result<Vec<Friend>> {
    let path = data_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let file: FriendsFile = toml::from_str(&content)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(file.friends)
}

pub fn save_friends(friends: &[Friend]) -> Result<()> {
    let path = data_path();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let file = FriendsFile {
        friends: friends.to_vec(),
    };
    let content = toml::to_string_pretty(&file)?;
    fs::write(&path, content)
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}
