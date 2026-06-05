use crate::app::{AppState, EditField, EditState, Screen};
use crate::apps::{available_applets, AppletKind};
use crate::data::friend::{BookProfile, Friend, JobHuntProfile};
use crate::data::storage::{load_from, save_to};
use tempfile::TempDir;

fn temp_path(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join("friends.toml")
}

// --- Full add-friend cycle: form → friend → applets ---

#[test]
fn test_add_friend_with_job_unlocks_job_feed() {
    let mut edit = EditState::blank();
    *edit.get_mut(&EditField::Name) = "Alice".to_string();
    *edit.get_mut(&EditField::JobRole) = "Software Engineer".to_string();
    *edit.get_mut(&EditField::JobSkills) = "Rust, Python".to_string();

    let mut friend = Friend::new("Alice".to_string());
    edit.apply_to_friend(&mut friend);

    let applets = available_applets(&friend);
    assert!(applets.contains(&AppletKind::JobFeed));
    assert!(!applets.contains(&AppletKind::BookRecs));
    assert!(!applets.contains(&AppletKind::WikiPrep));
}

#[test]
fn test_add_friend_with_books_unlocks_book_recs() {
    let mut edit = EditState::blank();
    *edit.get_mut(&EditField::Name) = "Bob".to_string();
    *edit.get_mut(&EditField::BookGenres) = "sci-fi, fantasy".to_string();

    let mut friend = Friend::new("Bob".to_string());
    edit.apply_to_friend(&mut friend);

    let applets = available_applets(&friend);
    assert!(applets.contains(&AppletKind::BookRecs));
    assert!(!applets.contains(&AppletKind::JobFeed));
}

#[test]
fn test_add_friend_with_interests_unlocks_wiki_prep() {
    let mut edit = EditState::blank();
    *edit.get_mut(&EditField::Name) = "Carol".to_string();
    *edit.get_mut(&EditField::Interests) = "Byzantine history, jazz".to_string();

    let mut friend = Friend::new("Carol".to_string());
    edit.apply_to_friend(&mut friend);

    let applets = available_applets(&friend);
    assert!(applets.contains(&AppletKind::WikiPrep));
}

#[test]
fn test_filling_all_fields_unlocks_all_applets() {
    let mut edit = EditState::blank();
    *edit.get_mut(&EditField::Name) = "Dave".to_string();
    *edit.get_mut(&EditField::JobRole) = "Engineer".to_string();
    *edit.get_mut(&EditField::BookGenres) = "history".to_string();
    *edit.get_mut(&EditField::Interests) = "climbing".to_string();

    let mut friend = Friend::new("Dave".to_string());
    edit.apply_to_friend(&mut friend);

    let applets = available_applets(&friend);
    assert_eq!(applets.len(), 3);
}

// --- TOML persistence roundtrip with all field types ---

#[test]
fn test_full_toml_roundtrip_preserves_all_fields() {
    let dir = TempDir::new().unwrap();
    let path = temp_path(&dir);

    let mut friend = Friend {
        id: "round-trip-id".to_string(),
        name: "Eve".to_string(),
        email: Some("eve@example.com".to_string()),
        notes: Some("test notes".to_string()),
        interests: vec!["Rust".to_string(), "climbing".to_string()],
        job_hunt: Some(JobHuntProfile {
            desired_role: "Engineer".to_string(),
            skills: vec!["Rust".to_string()],
            location: Some("Austin".to_string()),
            remote_preference: Some("remote".to_string()),
            experience_level: Some("mid".to_string()),
        }),
        book_profile: Some(BookProfile {
            genres: vec!["sci-fi".to_string()],
            authors: vec!["Le Guin".to_string()],
        }),
    };

    save_to(&[friend.clone()], &path).unwrap();
    let loaded = load_from(&path).unwrap();
    friend = loaded.into_iter().next().unwrap();

    assert_eq!(friend.name, "Eve");
    assert_eq!(friend.email.as_deref(), Some("eve@example.com"));
    assert_eq!(friend.interests, vec!["Rust", "climbing"]);
    let jh = friend.job_hunt.as_ref().unwrap();
    assert_eq!(jh.desired_role, "Engineer");
    assert_eq!(jh.skills, vec!["Rust"]);
    assert_eq!(jh.experience_level.as_deref(), Some("mid"));
    let bp = friend.book_profile.as_ref().unwrap();
    assert_eq!(bp.genres, vec!["sci-fi"]);
    assert_eq!(bp.authors, vec!["Le Guin"]);
}

// --- Multiple friends stay independent ---

#[test]
fn test_multiple_friends_have_independent_applets() {
    let dir = TempDir::new().unwrap();
    let path = temp_path(&dir);

    let mut alice = Friend::new("Alice".to_string());
    alice.job_hunt = Some(JobHuntProfile {
        desired_role: "Engineer".to_string(),
        ..Default::default()
    });

    let mut bob = Friend::new("Bob".to_string());
    bob.interests = vec!["jazz".to_string()];

    let carol = Friend::new("Carol".to_string());

    save_to(&[alice, bob, carol], &path).unwrap();
    let loaded = load_from(&path).unwrap();

    assert_eq!(available_applets(&loaded[0]), vec![AppletKind::JobFeed]);
    assert_eq!(available_applets(&loaded[1]), vec![AppletKind::WikiPrep]);
    assert!(available_applets(&loaded[2]).is_empty());
}

// --- EditState full cycle: friend → edit state → back to friend ---

#[test]
fn test_edit_state_cycle_preserves_data() {
    let mut original = Friend {
        id: "test-id".to_string(),
        name: "Frank".to_string(),
        email: Some("frank@example.com".to_string()),
        notes: None,
        interests: vec!["jazz".to_string()],
        job_hunt: Some(JobHuntProfile {
            desired_role: "Designer".to_string(),
            skills: vec!["Figma".to_string(), "CSS".to_string()],
            location: None,
            remote_preference: Some("hybrid".to_string()),
            experience_level: None,
        }),
        book_profile: None,
    };

    let edit = EditState::from_friend(&original);
    edit.apply_to_friend(&mut original);

    assert_eq!(original.name, "Frank");
    assert_eq!(original.email.as_deref(), Some("frank@example.com"));
    assert_eq!(original.interests, vec!["jazz"]);
    let jh = original.job_hunt.as_ref().unwrap();
    assert_eq!(jh.desired_role, "Designer");
    assert_eq!(jh.skills, vec!["Figma", "CSS"]);
    assert_eq!(jh.remote_preference.as_deref(), Some("hybrid"));
}

// --- AppState navigation integration ---

#[test]
fn test_app_state_selection_wraps_to_last_after_delete() {
    use crate::events::handle_key;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let friends = vec![
        Friend::new("Alice".to_string()),
        Friend::new("Bob".to_string()),
        Friend::new("Carol".to_string()),
    ];
    let mut state = AppState::new(friends);
    state.selected_friend = 2;
    state.screen = Screen::ConfirmDelete;

    handle_key(&mut state, KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE));

    assert_eq!(state.friends.len(), 2);
    assert_eq!(state.selected_friend, 1);
}

// --- Live API tests (skipped by default, run with: cargo test -- --ignored) ---

#[tokio::test]
#[ignore]
async fn test_remotive_returns_results() {
    let profile = JobHuntProfile {
        desired_role: "Software Engineer".to_string(),
        skills: vec!["Rust".to_string()],
        ..Default::default()
    };
    let jobs = crate::api::remotive::fetch_jobs(&profile).await.unwrap();
    assert!(!jobs.is_empty(), "expected at least one job from Remotive");
}

#[tokio::test]
#[ignore]
async fn test_open_library_returns_results() {
    let profile = BookProfile {
        genres: vec!["science fiction".to_string()],
        authors: vec![],
    };
    let books = crate::api::open_library::fetch_books(&profile).await.unwrap();
    assert!(!books.is_empty(), "expected at least one book from Open Library");
}

#[tokio::test]
#[ignore]
async fn test_wikipedia_returns_summaries() {
    let interests = vec!["Rust (programming language)".to_string()];
    let summaries = crate::api::wikipedia::fetch_summaries(&interests).await.unwrap();
    assert_eq!(summaries.len(), 1);
    assert!(!summaries[0].extract.is_empty());
}
