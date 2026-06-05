use std::sync::Arc;

use crate::app::{AppState, EditState, Screen};
use crate::apps::build_registry;
use crate::data::friend::Friend;
use crate::data::storage::{load_from, save_to};
use tempfile::TempDir;

fn registry() -> Arc<crate::apps::AppletRegistry> {
    Arc::new(build_registry())
}

fn temp_path(dir: &TempDir) -> std::path::PathBuf {
    dir.path().join("friends.toml")
}

// --- Full add-friend cycle: form → friend → applets ---

#[test]
fn test_add_friend_with_job_unlocks_job_feed() {
    let reg = build_registry();
    let mut edit = EditState::blank_for_registry(&reg);
    if let Some(v) = edit.get_mut("name") { *v = "Alice".to_string(); }
    if let Some(v) = edit.get_mut("job_feed.desired_role") { *v = "Software Engineer".to_string(); }
    if let Some(v) = edit.get_mut("job_feed.skills") { *v = "Rust, Python".to_string(); }

    let mut friend = Friend::new("Alice".to_string());
    edit.apply_to_friend(&mut friend);

    let applets = reg.available_for(&friend);
    assert!(applets.iter().any(|a| a.key() == "job_feed"));
    assert!(!applets.iter().any(|a| a.key() == "book_recs"));
    assert!(!applets.iter().any(|a| a.key() == "wiki_prep"));
}

#[test]
fn test_add_friend_with_books_unlocks_book_recs() {
    let reg = build_registry();
    let mut edit = EditState::blank_for_registry(&reg);
    if let Some(v) = edit.get_mut("name") { *v = "Bob".to_string(); }
    if let Some(v) = edit.get_mut("book_recs.genres") { *v = "sci-fi, fantasy".to_string(); }

    let mut friend = Friend::new("Bob".to_string());
    edit.apply_to_friend(&mut friend);

    let applets = reg.available_for(&friend);
    assert!(applets.iter().any(|a| a.key() == "book_recs"));
    assert!(!applets.iter().any(|a| a.key() == "job_feed"));
}

#[test]
fn test_add_friend_with_interests_unlocks_wiki_prep() {
    let reg = build_registry();
    let mut edit = EditState::blank_for_registry(&reg);
    if let Some(v) = edit.get_mut("name") { *v = "Carol".to_string(); }
    if let Some(v) = edit.get_mut("wiki_prep.interests") { *v = "Byzantine history, jazz".to_string(); }

    let mut friend = Friend::new("Carol".to_string());
    edit.apply_to_friend(&mut friend);

    let applets = reg.available_for(&friend);
    assert!(applets.iter().any(|a| a.key() == "wiki_prep"));
}

#[test]
fn test_filling_all_fields_unlocks_all_applets() {
    let reg = build_registry();
    let mut edit = EditState::blank_for_registry(&reg);
    if let Some(v) = edit.get_mut("name") { *v = "Dave".to_string(); }
    if let Some(v) = edit.get_mut("job_feed.desired_role") { *v = "Engineer".to_string(); }
    if let Some(v) = edit.get_mut("book_recs.genres") { *v = "history".to_string(); }
    if let Some(v) = edit.get_mut("wiki_prep.interests") { *v = "climbing".to_string(); }

    let mut friend = Friend::new("Dave".to_string());
    edit.apply_to_friend(&mut friend);

    let applets = reg.available_for(&friend);
    assert_eq!(applets.len(), 3);
}

// --- TOML persistence roundtrip with all field types ---

#[test]
fn test_full_toml_roundtrip_preserves_all_fields() {
    let dir = TempDir::new().unwrap();
    let path = temp_path(&dir);

    let mut friend = Friend::new("Eve".to_string());
    friend.id = "round-trip-id".to_string();
    friend.email = Some("eve@example.com".to_string());
    friend.notes = Some("test notes".to_string());
    friend.extra.insert("job_feed".to_string(), [
        ("desired_role".to_string(), "Engineer".to_string()),
        ("skills".to_string(), "Rust".to_string()),
        ("location".to_string(), "Austin".to_string()),
        ("remote_preference".to_string(), "remote".to_string()),
        ("experience_level".to_string(), "mid".to_string()),
    ].into());
    friend.extra.insert("book_recs".to_string(), [
        ("genres".to_string(), "sci-fi".to_string()),
        ("authors".to_string(), "Le Guin".to_string()),
    ].into());
    friend.extra.insert("wiki_prep".to_string(), [
        ("interests".to_string(), "Rust, climbing".to_string()),
    ].into());

    save_to(&[friend], &path).unwrap();
    let loaded = load_from(&path).unwrap();
    let f = &loaded[0];

    assert_eq!(f.name, "Eve");
    assert_eq!(f.email.as_deref(), Some("eve@example.com"));
    assert_eq!(f.extra["job_feed"]["desired_role"], "Engineer");
    assert_eq!(f.extra["job_feed"]["skills"], "Rust");
    assert_eq!(f.extra["job_feed"]["experience_level"], "mid");
    assert_eq!(f.extra["book_recs"]["genres"], "sci-fi");
    assert_eq!(f.extra["book_recs"]["authors"], "Le Guin");
    assert_eq!(f.extra["wiki_prep"]["interests"], "Rust, climbing");
}

// --- Multiple friends stay independent ---

#[test]
fn test_multiple_friends_have_independent_applets() {
    let dir = TempDir::new().unwrap();
    let path = temp_path(&dir);
    let reg = build_registry();

    let mut alice = Friend::new("Alice".to_string());
    alice.extra.insert("job_feed".to_string(), [
        ("desired_role".to_string(), "Engineer".to_string()),
    ].into());

    let mut bob = Friend::new("Bob".to_string());
    bob.extra.insert("wiki_prep".to_string(), [
        ("interests".to_string(), "jazz".to_string()),
    ].into());

    let carol = Friend::new("Carol".to_string());

    save_to(&[alice, bob, carol], &path).unwrap();
    let loaded = load_from(&path).unwrap();

    let alice_applets = reg.available_for(&loaded[0]);
    let bob_applets = reg.available_for(&loaded[1]);
    let carol_applets = reg.available_for(&loaded[2]);

    assert!(alice_applets.iter().any(|a| a.key() == "job_feed"));
    assert!(!alice_applets.iter().any(|a| a.key() == "wiki_prep"));

    assert!(bob_applets.iter().any(|a| a.key() == "wiki_prep"));
    assert!(!bob_applets.iter().any(|a| a.key() == "job_feed"));

    assert!(carol_applets.is_empty());
}

// --- EditState full cycle: friend → edit state → back to friend ---

#[test]
fn test_edit_state_cycle_preserves_data() {
    let reg = build_registry();
    let mut original = Friend::new("Frank".to_string());
    original.id = "test-id".to_string();
    original.email = Some("frank@example.com".to_string());
    original.extra.insert("job_feed".to_string(), [
        ("desired_role".to_string(), "Designer".to_string()),
        ("skills".to_string(), "Figma, CSS".to_string()),
        ("remote_preference".to_string(), "hybrid".to_string()),
    ].into());
    original.extra.insert("wiki_prep".to_string(), [
        ("interests".to_string(), "jazz".to_string()),
    ].into());

    let edit = EditState::from_friend_and_registry(&original, &reg);
    edit.apply_to_friend(&mut original);

    assert_eq!(original.name, "Frank");
    assert_eq!(original.email.as_deref(), Some("frank@example.com"));
    assert_eq!(original.extra["job_feed"]["desired_role"], "Designer");
    assert_eq!(original.extra["job_feed"]["skills"], "Figma, CSS");
    assert_eq!(original.extra["job_feed"]["remote_preference"], "hybrid");
    assert_eq!(original.extra["wiki_prep"]["interests"], "jazz");
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
    let mut state = AppState::new(friends, registry());
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
    let items = crate::api::remotive::fetch_jobs("Software Engineer", &["Rust".to_string()])
        .await
        .unwrap();
    assert!(!items.is_empty(), "expected at least one job from Remotive");
}

#[tokio::test]
#[ignore]
async fn test_open_library_returns_results() {
    let items = crate::api::open_library::fetch_books(
        &["science fiction".to_string()],
        &[],
    )
    .await
    .unwrap();
    assert!(!items.is_empty(), "expected at least one book from Open Library");
}

#[tokio::test]
#[ignore]
async fn test_wikipedia_returns_summaries() {
    let items = crate::api::wikipedia::fetch_summaries(
        &["Rust (programming language)".to_string()],
    )
    .await
    .unwrap();
    assert_eq!(items.len(), 1);
    assert!(items[0].subtitle.as_ref().map(|s| !s.is_empty()).unwrap_or(false));
}
