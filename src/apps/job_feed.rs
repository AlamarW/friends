use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use super::{Applet, AppletItem, FieldDef, parse_csv};

pub struct JobFeedApplet;

#[async_trait]
impl Applet for JobFeedApplet {
    fn key(&self) -> &str { "job_feed" }
    fn name(&self) -> &str { "Job Feed" }

    fn fields(&self) -> Vec<FieldDef> {
        vec![
            FieldDef::required("desired_role", "Desired Role"),
            FieldDef::optional("skills", "Skills", Some("comma-separated")),
            FieldDef::optional("location", "Location", None),
            FieldDef::optional("remote_preference", "Remote Pref", Some("remote/hybrid/onsite")),
            FieldDef::optional("experience_level", "Level", Some("junior/mid/senior")),
        ]
    }

    fn is_unlocked(&self, profile: &HashMap<String, String>) -> bool {
        profile.get("desired_role").map(|s| !s.trim().is_empty()).unwrap_or(false)
    }

    async fn fetch(&self, profile: &HashMap<String, String>) -> Result<Vec<AppletItem>> {
        let role = profile.get("desired_role").map(|s| s.as_str()).unwrap_or("");
        let skills = profile.get("skills").map(|s| parse_csv(s)).unwrap_or_default();
        crate::api::remotive::fetch_jobs(role, &skills).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(role: &str) -> HashMap<String, String> {
        let mut m = HashMap::new();
        if !role.is_empty() {
            m.insert("desired_role".to_string(), role.to_string());
        }
        m
    }

    #[test]
    fn test_is_unlocked_with_role() {
        assert!(JobFeedApplet.is_unlocked(&profile("Engineer")));
    }

    #[test]
    fn test_is_unlocked_empty_role() {
        assert!(!JobFeedApplet.is_unlocked(&profile("")));
    }

    #[test]
    fn test_is_unlocked_missing_role() {
        assert!(!JobFeedApplet.is_unlocked(&HashMap::new()));
    }

    #[test]
    fn test_fields_contains_required_desired_role() {
        let fields = JobFeedApplet.fields();
        let role_field = fields.iter().find(|f| f.key == "desired_role").unwrap();
        assert!(role_field.required);
    }

    #[test]
    fn test_key_and_name() {
        assert_eq!(JobFeedApplet.key(), "job_feed");
        assert_eq!(JobFeedApplet.name(), "Job Feed");
    }
}
