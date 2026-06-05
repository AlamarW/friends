use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{AppState, EditState, LoadState, Screen};
use crate::apps::{available_applets, AppletKind};
use crate::data::friend::Friend;
use crate::data::storage::save_friends;

pub enum EventAction {
    Quit,
    FetchJobs,
    FetchBooks,
    FetchWiki,
    None,
}

pub fn handle_key(state: &mut AppState, key: KeyEvent) -> EventAction {
    match &state.screen.clone() {
        Screen::FriendsList => handle_list(state, key),
        Screen::FriendDetail => handle_detail(state, key),
        Screen::EditFriend => handle_edit(state, key),
        Screen::AddFriend => handle_edit(state, key),
        Screen::AppletView(kind) => handle_applet(state, key, kind.clone()),
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
            state.edit = Some(EditState::blank());
            state.screen = Screen::AddFriend;
        }
        KeyCode::Char('e') => {
            if let Some(friend) = state.current_friend() {
                state.edit = Some(EditState::from_friend(friend));
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
    let applets = state
        .current_friend()
        .map(|fr| available_applets(fr))
        .unwrap_or_default();

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            state.screen = Screen::FriendsList;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if !applets.is_empty() {
                state.selected_applet = (state.selected_applet + 1).min(applets.len() - 1);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.selected_applet = state.selected_applet.saturating_sub(1);
        }
        KeyCode::Enter => {
            if let Some(kind) = applets.get(state.selected_applet) {
                let action = open_applet(state, kind.clone());
                return action;
            }
        }
        KeyCode::Char('e') => {
            if let Some(friend) = state.current_friend() {
                state.edit = Some(EditState::from_friend(friend));
                state.screen = Screen::EditFriend;
            }
        }
        _ => {}
    }
    EventAction::None
}

fn open_applet(state: &mut AppState, kind: AppletKind) -> EventAction {
    state.applet_scroll = 0;
    state.screen = Screen::AppletView(kind.clone());
    match kind {
        AppletKind::JobFeed => {
            state.jobs = LoadState::Loading;
            EventAction::FetchJobs
        }
        AppletKind::BookRecs => {
            state.books = LoadState::Loading;
            EventAction::FetchBooks
        }
        AppletKind::WikiPrep => {
            state.wiki = LoadState::Loading;
            EventAction::FetchWiki
        }
    }
}

fn handle_applet(state: &mut AppState, key: KeyEvent, kind: AppletKind) -> EventAction {
    match key.code {
        KeyCode::Esc => {
            state.screen = Screen::FriendDetail;
        }
        KeyCode::Char('q') => {
            state.screen = Screen::FriendsList;
        }
        KeyCode::Char('r') => {
            state.applet_scroll = 0;
            return match kind {
                AppletKind::JobFeed => {
                    state.jobs = LoadState::Loading;
                    EventAction::FetchJobs
                }
                AppletKind::BookRecs => {
                    state.books = LoadState::Loading;
                    EventAction::FetchBooks
                }
                AppletKind::WikiPrep => {
                    state.wiki = LoadState::Loading;
                    EventAction::FetchWiki
                }
            };
        }
        KeyCode::Char('j') | KeyCode::Down => {
            state.applet_scroll += 1;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            state.applet_scroll = state.applet_scroll.saturating_sub(1);
        }
        KeyCode::Enter => {
            open_url_for_applet(state, &kind);
        }
        _ => {}
    }
    EventAction::None
}

fn open_url_for_applet(state: &AppState, kind: &AppletKind) {
    let url = match kind {
        AppletKind::JobFeed => {
            if let LoadState::Loaded(jobs) = &state.jobs {
                jobs.get(state.applet_scroll).map(|j| j.url.clone())
            } else {
                None
            }
        }
        AppletKind::BookRecs => {
            if let LoadState::Loaded(books) = &state.books {
                books.get(state.applet_scroll).map(|b| {
                    format!("https://openlibrary.org{}", b.key)
                })
            } else {
                None
            }
        }
        AppletKind::WikiPrep => None,
    };

    if let Some(url) = url {
        let _ = open::that(&url);
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
                edit.cursor = (edit.cursor + 1).min(edit.fields.len() - 1);
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
                let (field, _) = &edit.fields[cursor].clone();
                let val = edit.get_mut(field);
                val.pop();
            }
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                return EventAction::None;
            }
            if let Some(edit) = &mut state.edit {
                let cursor = edit.cursor;
                let (field, _) = &edit.fields[cursor].clone();
                let val = edit.get_mut(field);
                val.push(c);
            }
        }
        _ => {}
    }
    EventAction::None
}

fn save_edit(state: &mut AppState, is_add: bool) {
    let Some(edit) = state.edit.take() else { return };

    if edit.get(&crate::app::EditField::Name).trim().is_empty() {
        state.edit = Some(edit);
        state.status_msg = Some("Name is required.".to_string());
        return;
    }

    if is_add {
        let mut friend = Friend::new(edit.get(&crate::app::EditField::Name).trim().to_string());
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
    use crate::app::EditField;
    use crate::data::friend::{BookProfile, Friend, JobHuntProfile};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn friend_with_job() -> Friend {
        let mut f = Friend::new("Alice".to_string());
        f.job_hunt = Some(JobHuntProfile {
            desired_role: "Engineer".to_string(),
            ..Default::default()
        });
        f
    }

    fn state_with_friends(friends: Vec<Friend>) -> AppState {
        AppState::new(friends)
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
        assert!(matches!(state.screen, Screen::AppletView(AppletKind::JobFeed)));
        assert!(matches!(action, EventAction::FetchJobs));
        assert!(matches!(state.jobs, LoadState::Loading));
    }

    #[test]
    fn test_detail_applet_nav_down() {
        let mut f = friend_with_job();
        f.interests = vec!["climbing".to_string()];
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
        state.screen = Screen::AppletView(AppletKind::JobFeed);
        handle_key(&mut state, key(KeyCode::Esc));
        assert_eq!(state.screen, Screen::FriendDetail);
    }

    #[test]
    fn test_applet_q_returns_to_list() {
        let mut state = state_with_friends(vec![friend_with_job()]);
        state.screen = Screen::AppletView(AppletKind::JobFeed);
        handle_key(&mut state, key(KeyCode::Char('q')));
        assert_eq!(state.screen, Screen::FriendsList);
    }

    #[test]
    fn test_applet_r_triggers_refetch_jobs() {
        let mut state = state_with_friends(vec![friend_with_job()]);
        state.screen = Screen::AppletView(AppletKind::JobFeed);
        let action = handle_key(&mut state, key(KeyCode::Char('r')));
        assert!(matches!(action, EventAction::FetchJobs));
        assert_eq!(state.applet_scroll, 0);
    }

    #[test]
    fn test_applet_r_triggers_refetch_books() {
        let mut f = Friend::new("Alice".to_string());
        f.book_profile = Some(BookProfile {
            genres: vec!["sci-fi".to_string()],
            authors: vec![],
        });
        let mut state = state_with_friends(vec![f]);
        state.screen = Screen::AppletView(AppletKind::BookRecs);
        let action = handle_key(&mut state, key(KeyCode::Char('r')));
        assert!(matches!(action, EventAction::FetchBooks));
    }

    #[test]
    fn test_applet_r_triggers_refetch_wiki() {
        let mut f = Friend::new("Alice".to_string());
        f.interests = vec!["climbing".to_string()];
        let mut state = state_with_friends(vec![f]);
        state.screen = Screen::AppletView(AppletKind::WikiPrep);
        let action = handle_key(&mut state, key(KeyCode::Char('r')));
        assert!(matches!(action, EventAction::FetchWiki));
    }

    #[test]
    fn test_applet_scroll_down() {
        let mut state = state_with_friends(vec![friend_with_job()]);
        state.screen = Screen::AppletView(AppletKind::JobFeed);
        handle_key(&mut state, key(KeyCode::Char('j')));
        assert_eq!(state.applet_scroll, 1);
    }

    #[test]
    fn test_applet_scroll_up_clamped() {
        let mut state = state_with_friends(vec![friend_with_job()]);
        state.screen = Screen::AppletView(AppletKind::JobFeed);
        state.applet_scroll = 0;
        handle_key(&mut state, key(KeyCode::Char('k')));
        assert_eq!(state.applet_scroll, 0);
    }

    // --- Edit form ---

    #[test]
    fn test_edit_tab_advances_cursor() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::EditFriend;
        state.edit = Some(EditState::blank());
        handle_key(&mut state, key(KeyCode::Tab));
        assert_eq!(state.edit.as_ref().unwrap().cursor, 1);
    }

    #[test]
    fn test_edit_backtab_decrements_cursor() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::EditFriend;
        state.edit = Some({
            let mut e = EditState::blank();
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
        state.edit = Some(EditState::blank());
        handle_key(&mut state, key(KeyCode::Char('A')));
        handle_key(&mut state, key(KeyCode::Char('l')));
        handle_key(&mut state, key(KeyCode::Char('i')));
        let val = state.edit.as_ref().unwrap().get(&EditField::Name).to_string();
        assert_eq!(val, "Ali");
    }

    #[test]
    fn test_edit_backspace_removes_last_char() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::EditFriend;
        state.edit = Some({
            let mut e = EditState::blank();
            *e.get_mut(&EditField::Name) = "Ali".to_string();
            e
        });
        handle_key(&mut state, key(KeyCode::Backspace));
        assert_eq!(state.edit.as_ref().unwrap().get(&EditField::Name), "Al");
    }

    #[test]
    fn test_edit_esc_from_add_returns_to_list() {
        let mut state = state_with_friends(vec![]);
        state.screen = Screen::AddFriend;
        state.edit = Some(EditState::blank());
        handle_key(&mut state, key(KeyCode::Esc));
        assert_eq!(state.screen, Screen::FriendsList);
        assert!(state.edit.is_none());
    }

    #[test]
    fn test_edit_esc_from_edit_returns_to_detail() {
        let mut state = state_with_friends(vec![Friend::new("Alice".to_string())]);
        state.screen = Screen::EditFriend;
        state.edit = Some(EditState::blank());
        handle_key(&mut state, key(KeyCode::Esc));
        assert_eq!(state.screen, Screen::FriendDetail);
        assert!(state.edit.is_none());
    }

    #[test]
    fn test_edit_enter_with_empty_name_sets_status_msg() {
        let mut state = state_with_friends(vec![]);
        state.screen = Screen::AddFriend;
        state.edit = Some(EditState::blank());
        handle_key(&mut state, key(KeyCode::Enter));
        assert!(state.status_msg.is_some());
        assert_eq!(state.screen, Screen::AddFriend);
    }

    #[test]
    fn test_edit_enter_with_name_adds_friend() {
        let mut state = state_with_friends(vec![]);
        state.screen = Screen::AddFriend;
        let mut edit = EditState::blank();
        *edit.get_mut(&EditField::Name) = "NewFriend".to_string();
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
        let mut edit = EditState::from_friend(&state.friends[0]);
        *edit.get_mut(&EditField::Name) = "Alicia".to_string();
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
