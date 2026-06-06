use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::{Applet, AppletItem, FieldDef};

pub struct DiscordApplet;

#[async_trait]
impl Applet for DiscordApplet {
    fn key(&self) -> &str {
        "discord"
    }

    fn name(&self) -> &str {
        "Discord DM"
    }

    fn fields(&self) -> Vec<FieldDef> {
        vec![
            FieldDef::optional("discord_user_id", "User ID", Some("numeric snowflake ID")),
            FieldDef::optional(
                "default_message",
                "Default Message",
                Some("sent when you press [c] — default: Thinking of you!"),
            ),
        ]
    }

    // Bot token comes from DISCORD_BOT_TOKEN env var, not per-friend profile.
    fn is_unlocked(&self, profile: &HashMap<String, String>) -> bool {
        profile
            .get("discord_user_id")
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
    }

    async fn fetch(&self, profile: &HashMap<String, String>) -> Result<Vec<AppletItem>> {
        crate::api::discord::fetch_dm_history(profile).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile_with_id(id: &str) -> HashMap<String, String> {
        [("discord_user_id".to_string(), id.to_string())].into()
    }

    #[test]
    fn test_discord_key_and_name() {
        let a = DiscordApplet;
        assert_eq!(a.key(), "discord");
        assert_eq!(a.name(), "Discord DM");
    }

    #[test]
    fn test_discord_fields_has_user_id_and_default_message() {
        let fields = DiscordApplet.fields();
        assert!(fields.iter().any(|f| f.key == "discord_user_id"));
        assert!(fields.iter().any(|f| f.key == "default_message"));
        assert!(!fields.iter().any(|f| f.key == "bot_token"), "token must not be a per-friend field");
    }

    #[test]
    fn test_discord_unlocked_with_user_id() {
        assert!(DiscordApplet.is_unlocked(&profile_with_id("123456789012345678")));
    }

    #[test]
    fn test_discord_locked_missing_user_id() {
        assert!(!DiscordApplet.is_unlocked(&HashMap::new()));
    }

    #[test]
    fn test_discord_locked_whitespace_only_user_id() {
        assert!(!DiscordApplet.is_unlocked(&profile_with_id("   ")));
    }
}
