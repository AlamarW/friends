use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{AppState, EditState, LoadState, Screen};
use crate::data::friend::Friend;
use crate::data::storage::save_friends;

pub enum EventAction {
    Quit,
    Fetch(String),
    None,
}

pub fn handle_key(state: &mut AppState, key: KeyEvent) -> EventAction {
    match &state.screen.clone() {
        Screen::FriendsList => handle_list(state, key),
        Screen::FriendDetail => handle_detail(state, key),
        Screen::EditFriend => handle_edit(state, key),
        Screen::AddFriend => handle_edit(state, key),
        Screen::AppletView(applet_key) => handle_applet(state, key, applet_key.clone()),
        Screen::ConfirmDelete => handle_confirm_delete(state, key),
    }
}

fn handle_list(state: &mut AppState, key: KeyEvent) -> EventAction {
    match key.code {
        KeyCode::Char('q') => return EventAction::Quit,
        KeyCode::Char('j') | KeyCode::Down => {
            if !state.friends.is_empty() {
                state.selected_friend =
                    (state.selected_friend + 1).min(state.friends.len() - 1);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.selected_friend = state.selected_friend.saturating_sub(1);
        }
        KeyCode::Enter => {
            if !state.friends.is_empty() {
                state.screen = Screen::FriendDetail;
                state.selected_applet = 0;
            }
        }
        KeyCode::Char('a') => {
            let registry = state.registry.clone();
            state.edit = Some(EditState::blank_for_registry(&registry));
            state.screen = Screen::AddFriend;
        }
        KeyCode::Char('e') => {
            let registry = state.registry.clone();
            if let Some(friend) = state.current_friend().cloned() {
                state.edit = Some(EditState::from_friend_and_registry(&friend, &registry));
                state.screen = Screen::EditFriend;
            }
        }
        KeyCode::Char('d') => {
            if !state.friends.is_empty() {
                state.screen = Screen::ConfirmDelete;
            }
        }
        _ => {}
    }
    EventAction::None
}

fn handle_detail(state: &mut AppState, key: KeyEvent) -> EventAction {
    let registry = state.registry.clone();
    let applet_keys: Vec<String> = state
        .current_friend()
        .map(|fr| {
            registry
                .available_for(fr)
                .iter()
                .map(|a| a.key().to_string())
                .collect()
        })
        .unwrap_or_default();

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            state.screen = Screen::FriendsList;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if !applet_keys.is_empty() {
                state.selected_applet = (state.selected_applet + 1).min(applet_keys.len() - 1);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.selected_applet = state.selected_applet.saturating_sub(1);
        }
        KeyCode::Enter => {
            if let Some(key) = applet_keys.get(state.selected_applet).cloned() {
                return open_applet(state, key);
            }
        }
        KeyCode::Char('e') => {
            if let Some(friend) = state.current_friend().cloned() {
                state.edit = Some(EditState::from_friend_and_registry(&friend, &registry));
                state.screen = Screen::EditFriend;
            }
        }
        _ => {}
    }
    EventAction::None
}

fn open_applet(state: &mut AppState, key: String) -> EventAction {
    state.applet_scroll = 0;
    state.applet_data = LoadState::Loading;
    state.active_applet_key = Some(key.clone());
    state.screen = Screen::AppletView(key.clone());
    EventAction::Fetch(key)
}

fn handle_applet(state: &mut AppState, key: KeyEvent, applet_key: String) -> EventAction {
    match key.code {
        KeyCode::Esc => {
            state.screen = Screen::FriendDetail;
        }
        KeyCode::Char('q') => {
            state.screen = Screen::FriendsList;
        }
        KeyCode::Char('r') => {
            state.applet_scroll = 0;
            state.applet_data = LoadState::Loading;
            return EventAction::Fetch(applet_key);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            state.applet_scroll += 1;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.applet_scroll = state.applet_scroll.saturating_sub(1);
        }
        KeyCode::Enter => {
            open_url_for_applet(state);
        }
        _ => {}
    }
    EventAction::None
}

fn open_url_for_applet(state: &AppState) {
    if let LoadState::Loaded(items) = &state.applet_data {
        if let Some(item) = items.get(state.applet_scroll) {
            if let Some(url) = &item.url {
                let _ = open::that(url);
            }
        }
    }
}

fn handle_edit(state: &mut AppState, key: KeyEvent) -> EventAction {
    let is_add = state.screen == Screen::AddFriend;

    match key.code {
        KeyCode::Esc => {
            state.edit = None;
            state.screen = if is_add {
                Screen::FriendsList
            } else {
                Screen::FriendDetail
            };
        }
        KeyCode::Tab | KeyCode::Down => {
            if let Some(edit) = &mut state.edit {
                edit.cursor = (edit.cursor + 1).min(edit.rows.len() - 1);
            }
        }
        KeyCode::BackTab | KeyCode::Up => {
            if let Some(edit) = &mut state.edit {
                edit.cursor = edit.cursor.saturating_sub(1);
            }
        }
        KeyCode::Enter => {
            save_edit(state, is_add);
        }
        KeyCode::Backspace => {
            if let Some(edit) = &mut state.edit {
                let cursor = edit.cursor;
                edit.rows[cursor].value.pop();
            }
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return EventAction::None;
            }
            if let Some(edit) = &mut state.edit {
                let cursor = edit.cursor;
                edit.rows[cursor].value.push(c);
            }
        }
        _ => {}
    }
    EventAction::None
}

fn save_edit(state: &mut AppState, is_add: bool) {
    let Some(edit) = state.edit.take() else { return };

    if edit.get("name").trim().is_empty() {
        state.edit = Some(edit);
        state.status_msg = Some("Name is required.".to_string());
        return;
    }

    if is_add {
        let mut friend = Friend::new(edit.get("name").trim().to_string());
        edit.apply_to_friend(&mut friend);
        state.friends.push(friend);
        state.selected_friend = state.friends.len() - 1;
        state.screen = Screen::FriendDetail;
        state.selected_applet = 0;
    } else {
        if let Some(friend) = state.current_friend_mut() {
            edit.apply_to_friend(friend);
        }
        state.screen = Screen::FriendDetail;
    }

    state.status_msg = None;
    let _ = save_friends(&state.friends);
}

fn handle_confirm_delete(state: &mut AppState, key: KeyEvent) -> EventAction {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if !state.friends.is_empty() {
                state.friends.remove(state.selected_friend);
                if state.selected_friend >= state.friends.len() && !state.friends.is_empty() {
                    state.selected_friend = state.friends.len() - 1;
                } else if state.friends.is_empty() {
                    state.selected_friend = 0;
                }
                let _ = save_friends(&state.friends);
            }
            state.screen = Screen::FriendsList;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            state.screen = Screen::FriendsList;
        }
        _ => {}
    }
    EventAction::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::app::EditState;
    use crate::apps::build_registry;
    use crate::data::friend::Friend;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn friend_with_job() -> Friend {
        let mut f = Friend::new("Alice".to_string());
        f.extra.insert("job_feed".to_string(), [
            ("desired_role".to_string(), "Engineer".to_string()),
        ].into());
        f
    }

    fn state_with_friends(friends: Vec<Friend>) -> AppState {
        AppState::new(friends, Arc::new(build_registry()))
    }

    // --- FriendsList ---

    #[test]
    fn test_list_quit_returns_quit_action() {
        let mut state = state_with_friends(vec![]);
        let action = handle_key(&mut state, key(KeyCode::Char('q')));
        assert!(matches!(action, EventAction::Quit));
    }

    #[test]
    fn test_list_nav_down_increments_selection() {
        let mut state = state_with_friends(vec![
            Friend::new("Alice".to_string()),
            Friend::new("Bob".to_string()),
        ]);
        handle_key(&mut state, key(KeyCode::Char('j')));
        assert_eq!(state.selected_friend, 1);
    }

    #[test]
    fn test_list_nav_down_clamped_at_last() {
        let mut state = state_with_friends(vec![
            Friend::new("Alice".to_string()),
            Friend::new("Bob".to_string()),
        ]);
        state.selected_friend = 1;
        handle_key(&mut state, key(KeyCode::Char('j')));
        assert_eq!(state.selected_friend, 1);
    }

    #[test]
    fn test_list_nav_down_on_empty_is_noop() {
        let mut state = state_with_friends(vec![]);
        handle_key(&mut state, key(KeyCode::Down));
        assert_eq!(state.selected_friend, 0);
    }

    #[test]
    fn test_list_nav_up_decrements_selection() {
        let mut state = state_with_friends(vec![
            Friend::new("Alice".to_string()),
            Friend::new("Bob".to_string()),
        ]);
        state.selected_friend = 1;
        handle_key(&mut state, key(KeyCode::Char('k')));
        assert_eq!(state.selected_friend, 0);
    }

    #[test]
    fn test_list_nav_up_clamped_at_zero() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        handle_key(&mut state, key(KeyCode::Up));
        assert_eq!(state.selected_friend, 0);
    }

    #[test]
    fn test_list_enter_goes_to_detail() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        handle_key(&mut state, key(KeyCode::Enter));
        assert_eq!(state.screen, Screen::FriendDetail);
        assert_eq!(state.selected_applet, 0);
    }

    #[test]
    fn test_list_enter_on_empty_stays_on_list() {
        let mut state = state_with_friends(vec![]);
        handle_key(&mut state, key(KeyCode::Enter));
        assert_eq!(state.screen, Screen::FriendsList);
    }

    #[test]
    fn test_list_add_opens_add_screen() {
        let mut state = state_with_friends(vec![]);
        handle_key(&mut state, key(KeyCode::Char('a')));
        assert_eq!(state.screen, Screen::AddFriend);
        assert!(state.edit.is_some());
    }

    #[test]
    fn test_list_edit_opens_edit_screen() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        handle_key(&mut state, key(KeyCode::Char('e')));
        assert_eq!(state.screen, Screen::EditFriend);
        assert!(state.edit.is_some());
    }

    #[test]
    fn test_list_edit_on_empty_is_noop() {
        let mut state = state_with_friends(vec![]);
        handle_key(&mut state, key(KeyCode::Char('e')));
        assert_eq!(state.screen, Screen::FriendsList);
    }

    #[test]
    fn test_list_delete_opens_confirm() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        handle_key(&mut state, key(KeyCode::Char('d')));
        assert_eq!(state.screen, Screen::ConfirmDelete);
    }

    #[test]
    fn test_list_delete_on_empty_is_noop() {
        let mut state = state_with_friends(vec![]);
        handle_key(&mut state, key(KeyCode::Char('d')));
        assert_eq!(state.screen, Screen::FriendsList);
    }

    // --- FriendDetail ---

    #[test]
    fn test_detail_esc_returns_to_list() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::FriendDetail;
        handle_key(&mut state, key(KeyCode::Esc));
        assert_eq!(state.screen, Screen::FriendsList);
    }

    #[test]
    fn test_detail_q_returns_to_list() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::FriendDetail;
        handle_key(&mut state, key(KeyCode::Char('q')));
        assert_eq!(state.screen, Screen::FriendsList);
    }

    #[test]
    fn test_detail_e_opens_edit() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::FriendDetail;
        handle_key(&mut state, key(KeyCode::Char('e')));
        assert_eq!(state.screen, Screen::EditFriend);
    }

    #[test]
    fn test_detail_enter_opens_job_feed_applet() {
        let mut state = state_with_friends(vec![friend_with_job()]);
        state.screen = Screen::FriendDetail;
        let action = handle_key(&mut state, key(KeyCode::Enter));
        assert!(matches!(&state.screen, Screen::AppletView(k) if k == "job_feed"));
        assert!(matches!(&action, EventAction::Fetch(k) if k == "job_feed"));
        assert!(matches!(state.applet_data, LoadState::Loading));
    }

    #[test]
    fn test_detail_applet_nav_down() {
        let mut f = friend_with_job();
        f.extra.insert("wiki_prep".to_string(), [
            ("interests".to_string(), "climbing".to_string()),
        ].into());
        let mut state = state_with_friends(vec![f]);
        state.screen = Screen::FriendDetail;
        handle_key(&mut state, key(KeyCode::Char('j')));
        assert_eq!(state.selected_applet, 1);
    }

    #[test]
    fn test_detail_applet_nav_up_clamped() {
        let mut state = state_with_friends(vec![friend_with_job()]);
        state.screen = Screen::FriendDetail;
        state.selected_applet = 0;
        handle_key(&mut state, key(KeyCode::Char('k')));
        assert_eq!(state.selected_applet, 0);
    }

    // --- Applet view ---

    #[test]
    fn test_applet_esc_returns_to_detail() {
        let mut state = state_with_friends(vec![friend_with_job()]);
        state.screen = Screen::AppletView("job_feed".to_string());
        handle_key(&mut state, key(KeyCode::Esc));
        assert_eq!(state.screen, Screen::FriendDetail);
    }

    #[test]
    fn test_applet_q_returns_to_list() {
        let mut state = state_with_friends(vec![friend_with_job()]);
        state.screen = Screen::AppletView("job_feed".to_string());
        handle_key(&mut state, key(KeyCode::Char('q')));
        assert_eq!(state.screen, Screen::FriendsList);
    }

    #[test]
    fn test_applet_r_triggers_refetch_jobs() {
        let mut state = state_with_friends(vec![friend_with_job()]);
        state.screen = Screen::AppletView("job_feed".to_string());
        let action = handle_key(&mut state, key(KeyCode::Char('r')));
        assert!(matches!(&action, EventAction::Fetch(k) if k == "job_feed"));
        assert_eq!(state.applet_scroll, 0);
    }

    #[test]
    fn test_applet_r_triggers_refetch_books() {
        let mut f = Friend::new("Alice".to_string());
        f.extra.insert("book_recs".to_string(), [
            ("genres".to_string(), "sci-fi".to_string()),
        ].into());
        let mut state = state_with_friends(vec![f]);
        state.screen = Screen::AppletView("book_recs".to_string());
        let action = handle_key(&mut state, key(KeyCode::Char('r')));
        assert!(matches!(&action, EventAction::Fetch(k) if k == "book_recs"));
    }

    #[test]
    fn test_applet_r_triggers_refetch_wiki() {
        let mut f = Friend::new("Alice".to_string());
        f.extra.insert("wiki_prep".to_string(), [
            ("interests".to_string(), "climbing".to_string()),
        ].into());
        let mut state = state_with_friends(vec![f]);
        state.screen = Screen::AppletView("wiki_prep".to_string());
        let action = handle_key(&mut state, key(KeyCode::Char('r')));
        assert!(matches!(&action, EventAction::Fetch(k) if k == "wiki_prep"));
    }

    #[test]
    fn test_applet_scroll_down() {
        let mut state = state_with_friends(vec![friend_with_job()]);
        state.screen = Screen::AppletView("job_feed".to_string());
        handle_key(&mut state, key(KeyCode::Char('j')));
        assert_eq!(state.applet_scroll, 1);
    }

    #[test]
    fn test_applet_scroll_up_clamped() {
        let mut state = state_with_friends(vec![friend_with_job()]);
        state.screen = Screen::AppletView("job_feed".to_string());
        state.applet_scroll = 0;
        handle_key(&mut state, key(KeyCode::Char('k')));
        assert_eq!(state.applet_scroll, 0);
    }

    // --- Edit form ---

    #[test]
    fn test_edit_tab_advances_cursor() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::EditFriend;
        state.edit = Some(EditState::blank_for_registry(&build_registry()));
        handle_key(&mut state, key(KeyCode::Tab));
        assert_eq!(state.edit.as_ref().unwrap().cursor, 1);
    }

    #[test]
    fn test_edit_backtab_decrements_cursor() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::EditFriend;
        state.edit = Some({
            let mut e = EditState::blank_for_registry(&build_registry());
            e.cursor = 3;
            e
        });
        handle_key(&mut state, key(KeyCode::BackTab));
        assert_eq!(state.edit.as_ref().unwrap().cursor, 2);
    }

    #[test]
    fn test_edit_char_appended_to_active_field() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::EditFriend;
        state.edit = Some(EditState::blank_for_registry(&build_registry()));
        handle_key(&mut state, key(KeyCode::Char('A')));
        handle_key(&mut state, key(KeyCode::Char('l')));
        handle_key(&mut state, key(KeyCode::Char('i')));
        let val = state.edit.as_ref().unwrap().get("name").to_string();
        assert_eq!(val, "Ali");
    }

    #[test]
    fn test_edit_backspace_removes_last_char() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::EditFriend;
        state.edit = Some({
            let mut e = EditState::blank_for_registry(&build_registry());
            if let Some(v) = e.get_mut("name") { *v = "Ali".to_string(); }
            e
        });
        handle_key(&mut state, key(KeyCode::Backspace));
        assert_eq!(state.edit.as_ref().unwrap().get("name"), "Al");
    }

    #[test]
    fn test_edit_esc_from_add_returns_to_list() {
        let mut state = state_with_friends(vec![]);
        state.screen = Screen::AddFriend;
        state.edit = Some(EditState::blank_for_registry(&build_registry()));
        handle_key(&mut state, key(KeyCode::Esc));
        assert_eq!(state.screen, Screen::FriendsList);
        assert!(state.edit.is_none());
    }

    #[test]
    fn test_edit_esc_from_edit_returns_to_detail() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::EditFriend;
        state.edit = Some(EditState::blank_for_registry(&build_registry()));
        handle_key(&mut state, key(KeyCode::Esc));
        assert_eq!(state.screen, Screen::FriendDetail);
        assert!(state.edit.is_none());
    }

    #[test]
    fn test_edit_enter_with_empty_name_sets_status_msg() {
        let mut state = state_with_friends(vec![]);
        state.screen = Screen::AddFriend;
        state.edit = Some(EditState::blank_for_registry(&build_registry()));
        handle_key(&mut state, key(KeyCode::Enter));
        assert!(state.status_msg.is_some());
        assert_eq!(state.screen, Screen::AddFriend);
    }

    #[test]
    fn test_edit_enter_with_name_adds_friend() {
        let mut state = state_with_friends(vec![]);
        state.screen = Screen::AddFriend;
        let mut edit = EditState::blank_for_registry(&build_registry());
        if let Some(v) = edit.get_mut("name") { *v = "NewFriend".to_string(); }
        state.edit = Some(edit);
        handle_key(&mut state, key(KeyCode::Enter));
        assert_eq!(state.friends.len(), 1);
        assert_eq!(state.friends[0].name, "NewFriend");
        assert_eq!(state.screen, Screen::FriendDetail);
    }

    #[test]
    fn test_edit_enter_updates_existing_friend() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::EditFriend;
        let mut edit = EditState::from_friend_and_registry(&state.friends[0], &build_registry());
        if let Some(v) = edit.get_mut("name") { *v = "Alicia".to_string(); }
        state.edit = Some(edit);
        handle_key(&mut state, key(KeyCode::Enter));
        assert_eq!(state.friends[0].name, "Alicia");
        assert_eq!(state.screen, Screen::FriendDetail);
    }

    // --- ConfirmDelete ---

    #[test]
    fn test_confirm_delete_y_removes_friend() {
        let mut state = state_with_friends(vec![
            Friend::new("Alice".to_string()),
            Friend::new("Bob".to_string()),
        ]);
        state.screen = Screen::ConfirmDelete;
        handle_key(&mut state, key(KeyCode::Char('y')));
        assert_eq!(state.friends.len(), 1);
        assert_eq!(state.screen, Screen::FriendsList);
    }

    #[test]
    fn test_confirm_delete_uppercase_y_removes_friend() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::ConfirmDelete;
        handle_key(&mut state, key(KeyCode::Char('Y')));
        assert!(state.friends.is_empty());
        assert_eq!(state.selected_friend, 0);
    }

    #[test]
    fn test_confirm_delete_n_cancels() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::ConfirmDelete;
        handle_key(&mut state, key(KeyCode::Char('n')));
        assert_eq!(state.friends.len(), 1);
        assert_eq!(state.screen, Screen::FriendsList);
    }

    #[test]
    fn test_confirm_delete_esc_cancels() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::ConfirmDelete;
        handle_key(&mut state, key(KeyCode::Esc));
        assert_eq!(state.friends.len(), 1);
        assert_eq!(state.screen, Screen::FriendsList);
    }

    #[test]
    fn test_confirm_delete_adjusts_selection_when_last_deleted() {
        let mut state = state_with_friends(vec![
            Friend::new("Alice".to_string()),
            Friend::new("Bob".to_string()),
        ]);
        state.selected_friend = 1;
        state.screen = Screen::ConfirmDelete;
        handle_key(&mut state, key(KeyCode::Char('y')));
        assert_eq!(state.selected_friend, 0);
    }
}
