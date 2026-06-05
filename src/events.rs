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
