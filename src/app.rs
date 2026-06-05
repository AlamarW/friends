use crate::api::open_library::Book;
use crate::api::remotive::Job;
use crate::api::wikipedia::WikiSummary;
use crate::apps::AppletKind;
use crate::data::friend::Friend;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    FriendsList,
    FriendDetail,
    EditFriend,
    AddFriend,
    AppletView(AppletKind),
    ConfirmDelete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadState<T> {
    Idle,
    Loading,
    Loaded(T),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditField {
    Name,
    Email,
    Notes,
    Interests,
    JobRole,
    JobSkills,
    JobLocation,
    JobRemote,
    JobLevel,
    BookGenres,
    BookAuthors,
}

impl EditField {
    pub fn all() -> Vec<EditField> {
        vec![
            EditField::Name,
            EditField::Email,
            EditField::Notes,
            EditField::Interests,
            EditField::JobRole,
            EditField::JobSkills,
            EditField::JobLocation,
            EditField::JobRemote,
            EditField::JobLevel,
            EditField::BookGenres,
            EditField::BookAuthors,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            EditField::Name => "Name",
            EditField::Email => "Email",
            EditField::Notes => "Notes",
            EditField::Interests => "Interests (comma-sep)",
            EditField::JobRole => "Job: Desired Role",
            EditField::JobSkills => "Job: Skills (comma-sep)",
            EditField::JobLocation => "Job: Location",
            EditField::JobRemote => "Job: Remote Pref (remote/hybrid/onsite)",
            EditField::JobLevel => "Job: Level (junior/mid/senior)",
            EditField::BookGenres => "Books: Genres (comma-sep)",
            EditField::BookAuthors => "Books: Authors (comma-sep)",
        }
    }
}

pub struct EditState {
    pub fields: Vec<(EditField, String)>,
    pub cursor: usize,
}

impl EditState {
    pub fn from_friend(friend: &Friend) -> Self {
        let jh = friend.job_hunt.as_ref();
        let bp = friend.book_profile.as_ref();
        let fields = vec![
            (EditField::Name, friend.name.clone()),
            (EditField::Email, friend.email.clone().unwrap_or_default()),
            (EditField::Notes, friend.notes.clone().unwrap_or_default()),
            (EditField::Interests, friend.interests.join(", ")),
            (EditField::JobRole, jh.map(|j| j.desired_role.clone()).unwrap_or_default()),
            (EditField::JobSkills, jh.map(|j| j.skills.join(", ")).unwrap_or_default()),
            (EditField::JobLocation, jh.and_then(|j| j.location.clone()).unwrap_or_default()),
            (EditField::JobRemote, jh.and_then(|j| j.remote_preference.clone()).unwrap_or_default()),
            (EditField::JobLevel, jh.and_then(|j| j.experience_level.clone()).unwrap_or_default()),
            (EditField::BookGenres, bp.map(|b| b.genres.join(", ")).unwrap_or_default()),
            (EditField::BookAuthors, bp.map(|b| b.authors.join(", ")).unwrap_or_default()),
        ];
        Self { fields, cursor: 0 }
    }

    pub fn blank() -> Self {
        let fields = EditField::all()
            .into_iter()
            .map(|f| (f, String::new()))
            .collect();
        Self { fields, cursor: 0 }
    }

    pub fn get(&self, field: &EditField) -> &str {
        self.fields
            .iter()
            .find(|(f, _)| f == field)
            .map(|(_, v)| v.as_str())
            .unwrap_or("")
    }

    pub fn get_mut(&mut self, field: &EditField) -> &mut String {
        self.fields
            .iter_mut()
            .find(|(f, _)| f == field)
            .map(|(_, v)| v)
            .unwrap()
    }

    pub fn apply_to_friend(&self, friend: &mut Friend) {
        friend.name = self.get(&EditField::Name).to_string();
        let email = self.get(&EditField::Email).trim().to_string();
        friend.email = if email.is_empty() { None } else { Some(email) };
        let notes = self.get(&EditField::Notes).trim().to_string();
        friend.notes = if notes.is_empty() { None } else { Some(notes) };
        friend.interests = parse_csv(self.get(&EditField::Interests));

        let job_role = self.get(&EditField::JobRole).trim().to_string();
        if !job_role.is_empty() {
            let jh = friend.job_hunt.get_or_insert_with(Default::default);
            jh.desired_role = job_role;
            jh.skills = parse_csv(self.get(&EditField::JobSkills));
            let loc = self.get(&EditField::JobLocation).trim().to_string();
            jh.location = if loc.is_empty() { None } else { Some(loc) };
            let remote = self.get(&EditField::JobRemote).trim().to_string();
            jh.remote_preference = if remote.is_empty() { None } else { Some(remote) };
            let level = self.get(&EditField::JobLevel).trim().to_string();
            jh.experience_level = if level.is_empty() { None } else { Some(level) };
        } else {
            friend.job_hunt = None;
        }

        let genres = parse_csv(self.get(&EditField::BookGenres));
        let authors = parse_csv(self.get(&EditField::BookAuthors));
        if !genres.is_empty() || !authors.is_empty() {
            let bp = friend.book_profile.get_or_insert_with(Default::default);
            bp.genres = genres;
            bp.authors = authors;
        } else {
            friend.book_profile = None;
        }
    }
}

pub(crate) fn parse_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

pub struct AppState {
    pub screen: Screen,
    pub friends: Vec<Friend>,
    pub selected_friend: usize,
    pub selected_applet: usize,
    pub edit: Option<EditState>,
    pub jobs: LoadState<Vec<Job>>,
    pub books: LoadState<Vec<Book>>,
    pub wiki: LoadState<Vec<WikiSummary>>,
    pub applet_scroll: usize,
    pub status_msg: Option<String>,
}

impl AppState {
    pub fn new(friends: Vec<Friend>) -> Self {
        Self {
            screen: Screen::FriendsList,
            friends,
            selected_friend: 0,
            selected_applet: 0,
            edit: None,
            jobs: LoadState::Idle,
            books: LoadState::Idle,
            wiki: LoadState::Idle,
            applet_scroll: 0,
            status_msg: None,
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
    use crate::data::friend::{BookProfile, Friend, JobHuntProfile};

    fn friend_with_all_profiles() -> Friend {
        Friend {
            id: "id-1".to_string(),
            name: "Alice".to_string(),
            email: Some("alice@example.com".to_string()),
            notes: Some("great friend".to_string()),
            interests: vec!["climbing".to_string(), "Byzantine history".to_string()],
            job_hunt: Some(JobHuntProfile {
                desired_role: "Engineer".to_string(),
                skills: vec!["Rust".to_string(), "Go".to_string()],
                location: Some("Austin".to_string()),
                remote_preference: Some("remote".to_string()),
                experience_level: Some("senior".to_string()),
            }),
            book_profile: Some(BookProfile {
                genres: vec!["sci-fi".to_string()],
                authors: vec!["Le Guin".to_string()],
            }),
        }
    }

    // --- parse_csv ---

    #[test]
    fn test_parse_csv_normal() {
        assert_eq!(parse_csv("rust, python, go"), vec!["rust", "python", "go"]);
    }

    #[test]
    fn test_parse_csv_empty_string() {
        assert!(parse_csv("").is_empty());
    }

    #[test]
    fn test_parse_csv_whitespace_only() {
        assert!(parse_csv("  ,  ,  ").is_empty());
    }

    #[test]
    fn test_parse_csv_trailing_comma() {
        assert_eq!(parse_csv("rust,"), vec!["rust"]);
    }

    #[test]
    fn test_parse_csv_single_item() {
        assert_eq!(parse_csv("  rust  "), vec!["rust"]);
    }

    // --- EditState::blank ---

    #[test]
    fn test_edit_state_blank_all_empty() {
        let edit = EditState::blank();
        assert_eq!(edit.cursor, 0);
        assert!(edit.fields.len() == EditField::all().len());
        for (_, val) in &edit.fields {
            assert!(val.is_empty(), "expected empty, got {val:?}");
        }
    }

    // --- EditState::from_friend ---

    #[test]
    fn test_edit_state_from_friend_basic() {
        let friend = Friend::new("Bob".to_string());
        let edit = EditState::from_friend(&friend);
        assert_eq!(edit.get(&EditField::Name), "Bob");
        assert_eq!(edit.get(&EditField::Email), "");
        assert_eq!(edit.get(&EditField::JobRole), "");
    }

    #[test]
    fn test_edit_state_from_friend_all_fields() {
        let friend = friend_with_all_profiles();
        let edit = EditState::from_friend(&friend);

        assert_eq!(edit.get(&EditField::Name), "Alice");
        assert_eq!(edit.get(&EditField::Email), "alice@example.com");
        assert_eq!(edit.get(&EditField::Notes), "great friend");
        assert_eq!(edit.get(&EditField::Interests), "climbing, Byzantine history");
        assert_eq!(edit.get(&EditField::JobRole), "Engineer");
        assert_eq!(edit.get(&EditField::JobSkills), "Rust, Go");
        assert_eq!(edit.get(&EditField::JobLocation), "Austin");
        assert_eq!(edit.get(&EditField::JobRemote), "remote");
        assert_eq!(edit.get(&EditField::JobLevel), "senior");
        assert_eq!(edit.get(&EditField::BookGenres), "sci-fi");
        assert_eq!(edit.get(&EditField::BookAuthors), "Le Guin");
    }

    // --- EditState::get / get_mut ---

    #[test]
    fn test_edit_state_get_returns_value() {
        let mut edit = EditState::blank();
        *edit.get_mut(&EditField::Name) = "Dave".to_string();
        assert_eq!(edit.get(&EditField::Name), "Dave");
    }

    // --- apply_to_friend ---

    #[test]
    fn test_apply_name_and_email() {
        let mut edit = EditState::blank();
        *edit.get_mut(&EditField::Name) = "Bob".to_string();
        *edit.get_mut(&EditField::Email) = "bob@example.com".to_string();

        let mut friend = Friend::new("old".to_string());
        edit.apply_to_friend(&mut friend);

        assert_eq!(friend.name, "Bob");
        assert_eq!(friend.email.as_deref(), Some("bob@example.com"));
    }

    #[test]
    fn test_apply_empty_email_sets_none() {
        let mut edit = EditState::blank();
        *edit.get_mut(&EditField::Name) = "Bob".to_string();

        let mut friend = Friend::new("Bob".to_string());
        friend.email = Some("old@example.com".to_string());
        edit.apply_to_friend(&mut friend);

        assert!(friend.email.is_none());
    }

    #[test]
    fn test_apply_creates_job_hunt() {
        let mut edit = EditState::blank();
        *edit.get_mut(&EditField::Name) = "Alice".to_string();
        *edit.get_mut(&EditField::JobRole) = "Engineer".to_string();
        *edit.get_mut(&EditField::JobSkills) = "Rust, Go".to_string();
        *edit.get_mut(&EditField::JobLocation) = "Austin".to_string();
        *edit.get_mut(&EditField::JobRemote) = "remote".to_string();
        *edit.get_mut(&EditField::JobLevel) = "senior".to_string();

        let mut friend = Friend::new("Alice".to_string());
        edit.apply_to_friend(&mut friend);

        let jh = friend.job_hunt.as_ref().unwrap();
        assert_eq!(jh.desired_role, "Engineer");
        assert_eq!(jh.skills, vec!["Rust", "Go"]);
        assert_eq!(jh.location.as_deref(), Some("Austin"));
        assert_eq!(jh.remote_preference.as_deref(), Some("remote"));
        assert_eq!(jh.experience_level.as_deref(), Some("senior"));
    }

    #[test]
    fn test_apply_clears_job_hunt_when_role_empty() {
        let mut edit = EditState::from_friend(&friend_with_all_profiles());
        *edit.get_mut(&EditField::JobRole) = String::new();

        let mut friend = friend_with_all_profiles();
        edit.apply_to_friend(&mut friend);

        assert!(friend.job_hunt.is_none());
    }

    #[test]
    fn test_apply_creates_book_profile_from_genres() {
        let mut edit = EditState::blank();
        *edit.get_mut(&EditField::Name) = "Alice".to_string();
        *edit.get_mut(&EditField::BookGenres) = "sci-fi, fantasy".to_string();

        let mut friend = Friend::new("Alice".to_string());
        edit.apply_to_friend(&mut friend);

        let bp = friend.book_profile.as_ref().unwrap();
        assert_eq!(bp.genres, vec!["sci-fi", "fantasy"]);
        assert!(bp.authors.is_empty());
    }

    #[test]
    fn test_apply_creates_book_profile_from_authors_only() {
        let mut edit = EditState::blank();
        *edit.get_mut(&EditField::Name) = "Alice".to_string();
        *edit.get_mut(&EditField::BookAuthors) = "Le Guin".to_string();

        let mut friend = Friend::new("Alice".to_string());
        edit.apply_to_friend(&mut friend);

        assert!(friend.book_profile.is_some());
    }

    #[test]
    fn test_apply_clears_book_profile_when_both_empty() {
        let mut edit = EditState::from_friend(&friend_with_all_profiles());
        *edit.get_mut(&EditField::BookGenres) = String::new();
        *edit.get_mut(&EditField::BookAuthors) = String::new();

        let mut friend = friend_with_all_profiles();
        edit.apply_to_friend(&mut friend);

        assert!(friend.book_profile.is_none());
    }

    #[test]
    fn test_apply_interests_csv() {
        let mut edit = EditState::blank();
        *edit.get_mut(&EditField::Name) = "Alice".to_string();
        *edit.get_mut(&EditField::Interests) = "climbing, jazz, Rust".to_string();

        let mut friend = Friend::new("Alice".to_string());
        edit.apply_to_friend(&mut friend);

        assert_eq!(friend.interests, vec!["climbing", "jazz", "Rust"]);
    }

    #[test]
    fn test_apply_roundtrip_full_friend() {
        let original = friend_with_all_profiles();
        let edit = EditState::from_friend(&original);
        let mut restored = original.clone();
        edit.apply_to_friend(&mut restored);

        assert_eq!(restored.name, original.name);
        assert_eq!(restored.email, original.email);
        assert_eq!(restored.interests, original.interests);
        let jh = restored.job_hunt.as_ref().unwrap();
        assert_eq!(jh.desired_role, "Engineer");
        assert_eq!(jh.skills, vec!["Rust", "Go"]);
        let bp = restored.book_profile.as_ref().unwrap();
        assert_eq!(bp.genres, vec!["sci-fi"]);
    }

    // --- AppState ---

    #[test]
    fn test_app_state_current_friend_empty() {
        let state = AppState::new(vec![]);
        assert!(state.current_friend().is_none());
    }

    #[test]
    fn test_app_state_current_friend() {
        let state = AppState::new(vec![Friend::new("Alice".to_string())]);
        assert_eq!(state.current_friend().unwrap().name, "Alice");
    }

    #[test]
    fn test_app_state_initial_screen() {
        let state = AppState::new(vec![]);
        assert_eq!(state.screen, Screen::FriendsList);
    }

    #[test]
    fn test_edit_field_labels_are_nonempty() {
        for field in EditField::all() {
            assert!(!field.label().is_empty());
        }
    }
}
