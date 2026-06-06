use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

use super::{Applet, AppletItem, FieldDef};

pub struct BirthdayApplet;

#[async_trait]
impl Applet for BirthdayApplet {
    fn key(&self) -> &str {
        "birthday"
    }

    fn name(&self) -> &str {
        "Birthday Tracker"
    }

    fn fields(&self) -> Vec<FieldDef> {
        vec![FieldDef::optional("birthday", "Birthday", Some("YYYY-MM-DD"))]
    }

    fn is_unlocked(&self, profile: &HashMap<String, String>) -> bool {
        profile
            .get("birthday")
            .map(|s| {
                !s.trim().is_empty()
                    && NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").is_ok()
            })
            .unwrap_or(false)
    }

    async fn fetch(&self, profile: &HashMap<String, String>) -> Result<Vec<AppletItem>> {
        crate::api::birthday::compute_birthday_items(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(birthday: &str) -> HashMap<String, String> {
        [("birthday".to_string(), birthday.to_string())].into()
    }

    #[test]
    fn test_birthday_key_and_name() {
        let a = BirthdayApplet;
        assert_eq!(a.key(), "birthday");
        assert_eq!(a.name(), "Birthday Tracker");
    }

    #[test]
    fn test_birthday_fields_contains_birthday_key() {
        let fields = BirthdayApplet.fields();
        assert!(fields.iter().any(|f| f.key == "birthday"));
    }

    #[test]
    fn test_birthday_unlocked_with_valid_date() {
        assert!(BirthdayApplet.is_unlocked(&profile("1990-06-15")));
    }

    #[test]
    fn test_birthday_locked_with_empty_field() {
        assert!(!BirthdayApplet.is_unlocked(&profile("")));
    }

    #[test]
    fn test_birthday_locked_with_invalid_date() {
        assert!(!BirthdayApplet.is_unlocked(&profile("not-a-date")));
        assert!(!BirthdayApplet.is_unlocked(&profile("15/06/1990")));
        assert!(!BirthdayApplet.is_unlocked(&profile("June 15")));
    }

    #[test]
    fn test_birthday_locked_with_missing_key() {
        assert!(!BirthdayApplet.is_unlocked(&HashMap::new()));
    }
}
