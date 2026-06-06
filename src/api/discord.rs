use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;

use crate::apps::AppletItem;

#[derive(Deserialize)]
struct DmChannel {
    id: String,
}

#[derive(Deserialize)]
struct MessageAuthor {
    username: String,
    #[serde(default)]
    bot: bool,
}

#[derive(Deserialize)]
struct Message {
    content: String,
    author: MessageAuthor,
    timestamp: String,
}

fn get_token() -> Result<String> {
    std::env::var("DISCORD_BOT_TOKEN")
        .map_err(|_| anyhow!("DISCORD_BOT_TOKEN environment variable is not set"))
}

fn get_user_id(profile: &HashMap<String, String>) -> Result<&str> {
    profile
        .get("discord_user_id")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow!("discord_user_id is required"))
}

async fn open_dm_channel(client: &reqwest::Client, token: &str, user_id: &str) -> Result<String> {
    let channel: DmChannel = client
        .post("https://discord.com/api/v10/users/@me/channels")
        .header("Authorization", format!("Bot {token}"))
        .json(&serde_json::json!({ "recipient_id": user_id }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(channel.id)
}

pub async fn fetch_dm_history(profile: &HashMap<String, String>) -> Result<Vec<AppletItem>> {
    let token = get_token()?;
    let user_id = get_user_id(profile)?;

    let client = reqwest::Client::new();
    let channel_id = open_dm_channel(&client, &token, user_id).await?;

    let messages: Vec<Message> = client
        .get(format!(
            "https://discord.com/api/v10/channels/{channel_id}/messages?limit=20"
        ))
        .header("Authorization", format!("Bot {token}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if messages.is_empty() {
        return Ok(vec![AppletItem {
            title: "No messages yet — press [c] to write one".to_string(),
            subtitle: None,
            url: None,
        }]);
    }

    let items = messages
        .into_iter()
        .map(|msg| {
            let label = if msg.author.bot {
                "[You]".to_string()
            } else {
                format!("[{}]", msg.author.username)
            };
            let content = truncate_str(&msg.content, 80);
            AppletItem {
                title: format!("{label} {content}"),
                subtitle: Some(format_timestamp(&msg.timestamp)),
                url: None,
            }
        })
        .collect();

    Ok(items)
}

pub async fn send_dm(profile: &HashMap<String, String>, message: &str) -> Result<Vec<AppletItem>> {
    let token = get_token()?;
    let user_id = get_user_id(profile)?;

    let client = reqwest::Client::new();
    let channel_id = open_dm_channel(&client, &token, user_id).await?;

    client
        .post(format!(
            "https://discord.com/api/v10/channels/{channel_id}/messages"
        ))
        .header("Authorization", format!("Bot {token}"))
        .json(&serde_json::json!({ "content": message }))
        .send()
        .await?
        .error_for_status()?;

    Ok(vec![AppletItem {
        title: "Sent!".to_string(),
        subtitle: Some(truncate_str(message, 60)),
        url: None,
    }])
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}

// "2024-01-15T10:30:00.000000+00:00" → "Jan 15, 10:30"
fn format_timestamp(ts: &str) -> String {
    if ts.len() < 16 {
        return ts.to_string();
    }
    let month_abbr = match ts.get(5..7) {
        Some("01") => "Jan",
        Some("02") => "Feb",
        Some("03") => "Mar",
        Some("04") => "Apr",
        Some("05") => "May",
        Some("06") => "Jun",
        Some("07") => "Jul",
        Some("08") => "Aug",
        Some("09") => "Sep",
        Some("10") => "Oct",
        Some("11") => "Nov",
        Some("12") => "Dec",
        _ => return ts.to_string(),
    };
    let day = ts.get(8..10).unwrap_or("??").trim_start_matches('0');
    let time = ts.get(11..16).unwrap_or("??:??");
    format!("{month_abbr} {day}, {time}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile_with_id(id: &str) -> HashMap<String, String> {
        [("discord_user_id".to_string(), id.to_string())].into()
    }

    #[test]
    fn test_get_user_id_missing_returns_error() {
        assert!(get_user_id(&HashMap::new()).is_err());
    }

    #[test]
    fn test_get_user_id_whitespace_only_returns_error() {
        assert!(get_user_id(&profile_with_id("   ")).is_err());
    }

    #[test]
    fn test_get_user_id_present_returns_value() {
        assert_eq!(get_user_id(&profile_with_id("123456789")).unwrap(), "123456789");
    }

    #[test]
    fn test_get_user_id_trims_whitespace() {
        assert_eq!(get_user_id(&profile_with_id("  123  ")).unwrap(), "123");
    }

    #[test]
    fn test_format_timestamp_full() {
        assert_eq!(
            format_timestamp("2024-06-15T10:30:00.000000+00:00"),
            "Jun 15, 10:30"
        );
    }

    #[test]
    fn test_format_timestamp_january_first() {
        assert_eq!(format_timestamp("2024-01-01T08:05:00+00:00"), "Jan 1, 08:05");
    }

    #[test]
    fn test_format_timestamp_short_returns_as_is() {
        assert_eq!(format_timestamp("too-short"), "too-short");
    }

    #[test]
    fn test_truncate_str_no_truncation() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_str_truncates() {
        let result = truncate_str("hello world", 6);
        assert!(result.len() <= 8, "should be short: {result}");
        assert!(result.starts_with("hello"));
    }

    #[tokio::test]
    #[ignore]
    async fn test_fetch_history_live() {
        let user_id = std::env::var("DISCORD_TEST_USER_ID").expect("DISCORD_TEST_USER_ID not set");
        let profile = [("discord_user_id".to_string(), user_id)].into();
        let result = fetch_dm_history(&profile).await;
        assert!(result.is_ok(), "fetch failed: {:?}", result.err());
    }

    #[tokio::test]
    #[ignore]
    async fn test_send_dm_live() {
        let user_id = std::env::var("DISCORD_TEST_USER_ID").expect("DISCORD_TEST_USER_ID not set");
        let profile = [("discord_user_id".to_string(), user_id)].into();
        let result = send_dm(&profile, "Integration test — please ignore").await;
        assert!(result.is_ok(), "send failed: {:?}", result.err());
        assert_eq!(result.unwrap()[0].title, "Sent!");
    }
}
