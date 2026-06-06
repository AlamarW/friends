use std::sync::Arc;

use crate::apps::{AppletItem, AppletRegistry};
use crate::data::friend::Friend;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    FriendsList,
    FriendDetail,
    EditFriend,
    AddFriend,
    AppletView(String),
    ComposeMessage(String), // applet key
    ConfirmDelete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadState<T> {
    Idle,
    Loading,
    Loaded(T),
    Error(String),
}

pub struct EditRow {
    pub key: String,
    pub label: String,
    pub hint: Option<String>,
    pub value: String,
}

pub struct EditState {
    pub rows: Vec<EditRow>,
    pub cursor: usize,
}

impl EditState {
    pub fn from_friend_and_registry(friend: &Friend, registry: &AppletRegistry) -> Self {
        let mut rows = vec![
            EditRow { key: "name".to_string(), label: "Name".to_string(), hint: None, value: friend.name.clone() },
            EditRow { key: "email".to_string(), label: "Email".to_string(), hint: None, value: friend.email.clone().unwrap_or_default() },
            EditRow { key: "notes".to_string(), label: "Notes".to_string(), hint: None, value: friend.notes.clone().unwrap_or_default() },
        ];
        for applet in registry.all() {
            let profile = friend.profile_for(applet.key());
            for field in applet.fields() {
                let key = format!("{}.{}", applet.key(), field.key);
                let value = profile.get(field.key).cloned().unwrap_or_default();
                let label = format!("{}: {}", applet.name(), field.label);
                rows.push(EditRow {
                    key,
                    label,
                    hint: field.hint.map(|s| s.to_string()),
                    value,
                });
            }
        }
        Self { rows, cursor: 0 }
    }

    pub fn blank_for_registry(registry: &AppletRegistry) -> Self {
        let mut rows = vec![
            EditRow { key: "name".to_string(), label: "Name".to_string(), hint: None, value: String::new() },
            EditRow { key: "email".to_string(), label: "Email".to_string(), hint: None, value: String::new() },
            EditRow { key: "notes".to_string(), label: "Notes".to_string(), hint: None, value: String::new() },
        ];
        for applet in registry.all() {
            for field in applet.fields() {
                let key = format!("{}.{}", applet.key(), field.key);
                let label = format!("{}: {}", applet.name(), field.label);
                rows.push(EditRow {
                    key,
                    label,
                    hint: field.hint.map(|s| s.to_string()),
                    value: String::new(),
                });
            }
        }
        Self { rows, cursor: 0 }
    }

    pub fn get(&self, key: &str) -> &str {
        self.rows.iter().find(|r| r.key == key).map(|r| r.value.as_str()).unwrap_or("")
    }

    #[cfg(test)]
    pub fn get_mut(&mut self, key: &str) -> Option<&mut String> {
        self.rows.iter_mut().find(|r| r.key == key).map(|r| &mut r.value)
    }

    pub fn apply_to_friend(&self, friend: &mut Friend) {
        friend.name = self.get("name").trim().to_string();
        let email = self.get("email").trim().to_string();
        friend.email = if email.is_empty() { None } else { Some(email) };
        let notes = self.get("notes").trim().to_string();
        friend.notes = if notes.is_empty() { None } else { Some(notes) };

        let mut applet_buckets: std::collections::HashMap<String, std::collections::HashMap<String, String>> = std::collections::HashMap::new();

        for row in &self.rows {
            if let Some(dot) = row.key.find('.') {
                let applet_key = &row.key[..dot];
                let field_key = &row.key[dot + 1..];
                let value = row.value.trim().to_string();
                if !value.is_empty() {
                    applet_buckets
                        .entry(applet_key.to_string())
                        .or_default()
                        .insert(field_key.to_string(), value);
                }
            }
        }

        let applet_keys: Vec<String> = self.rows.iter()
            .filter_map(|r| r.key.find('.').map(|i| r.key[..i].to_string()))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for key in &applet_keys {
            let profile = applet_buckets.remove(key.as_str()).unwrap_or_default();
            friend.set_profile(key, profile);
        }
    }
}

pub struct AppState {
    pub screen: Screen,
    pub friends: Vec<Friend>,
    pub selected_friend: usize,
    pub selected_applet: usize,
    pub edit: Option<EditState>,
    pub applet_data: LoadState<Vec<AppletItem>>,
    pub active_applet_key: Option<String>,
    pub applet_scroll: usize,
    pub status_msg: Option<String>,
    pub compose_draft: String,
    pub registry: Arc<AppletRegistry>,
}

impl AppState {
    pub fn new(friends: Vec<Friend>, registry: Arc<AppletRegistry>) -> Self {
        Self {
            screen: Screen::FriendsList,
            friends,
            selected_friend: 0,
            selected_applet: 0,
            edit: None,
            applet_data: LoadState::Idle,
            active_applet_key: None,
            applet_scroll: 0,
            status_msg: None,
            compose_draft: String::new(),
            registry,
        }
    }

    pub fn current_friend(&self) -> Option<&Friend> {
        self.friends.get(self.selected_friend)
    }

    pub fn current_friend_mut(&mut self) -> Option<&mut Friend> {
        self.friends.get_mut(self.selected_friend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apps::build_registry;

    fn registry() -> Arc<AppletRegistry> {
        Arc::new(build_registry())
    }

    fn full_friend() -> Friend {
        let mut f = Friend::new("Alice".to_string());
        f.email = Some("alice@example.com".to_string());
        f.notes = Some("great friend".to_string());
        f.extra.insert("job_feed".to_string(), [
            ("desired_role".to_string(), "Engineer".to_string()),
            ("skills".to_string(), "Rust, Go".to_string()),
            ("location".to_string(), "Austin".to_string()),
        ].into());
        f.extra.insert("book_recs".to_string(), [
            ("genres".to_string(), "sci-fi".to_string()),
            ("authors".to_string(), "Le Guin".to_string()),
        ].into());
        f.extra.insert("wiki_prep".to_string(), [
            ("interests".to_string(), "climbing, jazz".to_string()),
        ].into());
        f
    }

    // --- EditState ---

    #[test]
    fn test_blank_rows_all_empty() {
        let edit = EditState::blank_for_registry(&build_registry());
        assert!(edit.rows.len() > 3, "should have core + applet rows");
        for row in &edit.rows {
            assert!(row.value.is_empty(), "row {} should be empty", row.key);
        }
        assert_eq!(edit.cursor, 0);
    }

    #[test]
    fn test_from_friend_core_fields() {
        let f = full_friend();
        let edit = EditState::from_friend_and_registry(&f, &build_registry());
        assert_eq!(edit.get("name"), "Alice");
        assert_eq!(edit.get("email"), "alice@example.com");
        assert_eq!(edit.get("notes"), "great friend");
    }

    #[test]
    fn test_from_friend_applet_fields() {
        let f = full_friend();
        let edit = EditState::from_friend_and_registry(&f, &build_registry());
        assert_eq!(edit.get("job_feed.desired_role"), "Engineer");
        assert_eq!(edit.get("job_feed.skills"), "Rust, Go");
        assert_eq!(edit.get("book_recs.genres"), "sci-fi");
        assert_eq!(edit.get("wiki_prep.interests"), "climbing, jazz");
    }

    #[test]
    fn test_from_friend_missing_applet_fields_empty() {
        let f = Friend::new("Bob".to_string());
        let edit = EditState::from_friend_and_registry(&f, &build_registry());
        assert_eq!(edit.get("job_feed.desired_role"), "");
        assert_eq!(edit.get("book_recs.genres"), "");
    }

    #[test]
    fn test_get_mut_updates_value() {
        let mut edit = EditState::blank_for_registry(&build_registry());
        if let Some(v) = edit.get_mut("name") { *v = "Alice".to_string(); }
        assert_eq!(edit.get("name"), "Alice");
    }

    #[test]
    fn test_apply_core_fields() {
        let mut edit = EditState::blank_for_registry(&build_registry());
        if let Some(v) = edit.get_mut("name") { *v = "Bob".to_string(); }
        if let Some(v) = edit.get_mut("email") { *v = "bob@example.com".to_string(); }

        let mut friend = Friend::new("old".to_string());
        edit.apply_to_friend(&mut friend);

        assert_eq!(friend.name, "Bob");
        assert_eq!(friend.email.as_deref(), Some("bob@example.com"));
    }

    #[test]
    fn test_apply_whitespace_only_email_sets_none() {
        let mut edit = EditState::blank_for_registry(&build_registry());
        if let Some(v) = edit.get_mut("name") { *v = "Bob".to_string(); }
        if let Some(v) = edit.get_mut("email") { *v = "   ".to_string(); }
        let mut friend = Friend::new("Bob".to_string());
        edit.apply_to_friend(&mut friend);
        assert!(friend.email.is_none());
    }

    #[test]
    fn test_apply_whitespace_only_notes_sets_none() {
        let mut edit = EditState::blank_for_registry(&build_registry());
        if let Some(v) = edit.get_mut("name") { *v = "Bob".to_string(); }
        if let Some(v) = edit.get_mut("notes") { *v = "   ".to_string(); }
        let mut friend = Friend::new("Bob".to_string());
        friend.notes = Some("old note".to_string());
        edit.apply_to_friend(&mut friend);
        assert!(friend.notes.is_none());
    }

    #[test]
    fn test_apply_preserves_unregistered_applet_data() {
        let f = full_friend();
        let edit = EditState::from_friend_and_registry(&f, &build_registry());
        let mut friend = f.clone();
        friend.extra.insert("unknown_applet".to_string(), [
            ("secret_field".to_string(), "preserved".to_string()),
        ].into());
        edit.apply_to_friend(&mut friend);
        assert_eq!(friend.extra["unknown_applet"]["secret_field"], "preserved");
    }

    #[test]
    fn test_apply_empty_email_sets_none() {
        let mut edit = EditState::blank_for_registry(&build_registry());
        if let Some(v) = edit.get_mut("name") { *v = "Bob".to_string(); }
        let mut friend = Friend::new("Bob".to_string());
        friend.email = Some("old@example.com".to_string());
        edit.apply_to_friend(&mut friend);
        assert!(friend.email.is_none());
    }

    #[test]
    fn test_apply_writes_applet_fields() {
        let mut edit = EditState::blank_for_registry(&build_registry());
        if let Some(v) = edit.get_mut("name") { *v = "Alice".to_string(); }
        if let Some(v) = edit.get_mut("job_feed.desired_role") { *v = "Engineer".to_string(); }
        if let Some(v) = edit.get_mut("job_feed.skills") { *v = "Rust".to_string(); }

        let mut friend = Friend::new("Alice".to_string());
        edit.apply_to_friend(&mut friend);

        assert_eq!(friend.extra["job_feed"]["desired_role"], "Engineer");
        assert_eq!(friend.extra["job_feed"]["skills"], "Rust");
    }

    #[test]
    fn test_apply_clears_applet_when_all_fields_empty() {
        let f = full_friend();
        let mut edit = EditState::from_friend_and_registry(&f, &build_registry());
        // Clear all job_feed fields
        for row in edit.rows.iter_mut().filter(|r| r.key.starts_with("job_feed.")) {
            row.value.clear();
        }
        let mut friend = f.clone();
        edit.apply_to_friend(&mut friend);
        assert!(!friend.extra.contains_key("job_feed"));
        // Other applets should be untouched
        assert!(friend.extra.contains_key("book_recs"));
    }

    #[test]
    fn test_apply_roundtrip_full_friend() {
        let original = full_friend();
        let edit = EditState::from_friend_and_registry(&original, &build_registry());
        let mut restored = original.clone();
        edit.apply_to_friend(&mut restored);
        assert_eq!(restored.name, original.name);
        assert_eq!(restored.email, original.email);
        assert_eq!(restored.extra["job_feed"]["desired_role"], "Engineer");
        assert_eq!(restored.extra["book_recs"]["genres"], "sci-fi");
        assert_eq!(restored.extra["wiki_prep"]["interests"], "climbing, jazz");
    }

    // --- AppState ---

    #[test]
    fn test_app_state_initial_screen() {
        let state = AppState::new(vec![], registry());
        assert_eq!(state.screen, Screen::FriendsList);
    }

    #[test]
    fn test_app_state_current_friend_empty() {
        let state = AppState::new(vec![], registry());
        assert!(state.current_friend().is_none());
    }

    #[test]
    fn test_app_state_current_friend() {
        let state = AppState::new(vec![Friend::new("Alice".to_string())], registry());
        assert_eq!(state.current_friend().unwrap().name, "Alice");
    }
}
