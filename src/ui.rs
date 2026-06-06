use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app::{AppState, LoadState, Screen};

pub fn draw(f: &mut Frame, state: &AppState) {
    match &state.screen {
        Screen::FriendsList | Screen::FriendDetail => draw_main(f, state),
        Screen::EditFriend | Screen::AddFriend => draw_edit(f, state),
        Screen::AppletView(key) => draw_applet(f, state, key.clone()),
        Screen::ComposeMessage(key) => draw_compose(f, state, key.clone()),
        Screen::ConfirmDelete => {
            draw_main(f, state);
            draw_confirm_delete(f, state);
        }
    }
}

fn draw_main(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    draw_friends_list(f, state, chunks[0]);
    draw_friend_detail(f, state, chunks[1]);
}

fn draw_friends_list(f: &mut Frame, state: &AppState, area: Rect) {
    let items: Vec<ListItem> = state
        .friends
        .iter()
        .map(|fr| ListItem::new(fr.name.clone()))
        .collect();

    let mut list_state = ListState::default();
    if !state.friends.is_empty() {
        list_state.select(Some(state.selected_friend));
    }

    let hint = if state.friends.is_empty() {
        " [a] add"
    } else {
        " [a]add [d]del [e]edit"
    };

    let block = Block::default()
        .title(format!(" friends {hint} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if state.screen == Screen::FriendsList {
            Color::Cyan
        } else {
            Color::DarkGray
        }));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, area, &mut list_state);
}

fn draw_friend_detail(f: &mut Frame, state: &AppState, area: Rect) {
    let Some(friend) = state.current_friend() else {
        let block = Block::default().title(" detail ").borders(Borders::ALL);
        let p = Paragraph::new("No friends yet. Press [a] to add one.")
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, area);
        return;
    };

    let available = state.registry.available_for(friend);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(available.len() as u16 + 4)])
        .split(area);

    // Info panel
    let mut lines = vec![
        Line::from(vec![
            Span::styled(&friend.name, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];
    if let Some(email) = &friend.email {
        lines.push(Line::from(format!("email:  {email}")));
    }
    if let Some(notes) = &friend.notes {
        lines.push(Line::from(format!("notes:  {notes}")));
    }

    for applet in state.registry.all() {
        let profile = friend.profile_for(applet.key());
        if !profile.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                applet.name().to_string(),
                Style::default().add_modifier(Modifier::UNDERLINED),
            )]));
            for field in applet.fields() {
                if let Some(val) = profile.get(field.key) {
                    if !val.is_empty() {
                        lines.push(Line::from(format!("  {}: {val}", field.label)));
                    }
                }
            }
        }
    }

    let detail_block = Block::default()
        .title(" detail  [e]edit ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let detail = Paragraph::new(Text::from(lines))
        .block(detail_block)
        .wrap(Wrap { trim: true });
    f.render_widget(detail, chunks[0]);

    // Applets panel
    if available.is_empty() {
        let block = Block::default()
            .title(" applets ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        let p = Paragraph::new("Fill in fields to unlock applets.")
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, chunks[1]);
    } else {
        let items: Vec<ListItem> = available
            .iter()
            .map(|a| ListItem::new(a.name().to_string()))
            .collect();

        let mut list_state = ListState::default();
        if state.screen == Screen::FriendDetail {
            list_state.select(Some(state.selected_applet));
        }

        let block = Block::default()
            .title(" applets  [enter]open ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if state.screen == Screen::FriendDetail {
                Color::Cyan
            } else {
                Color::DarkGray
            }));

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        f.render_stateful_widget(list, chunks[1], &mut list_state);
    }

    // Status message
    if let Some(msg) = &state.status_msg {
        let status_area = Rect {
            x: area.x + 1,
            y: area.y + area.height.saturating_sub(1),
            width: area.width.saturating_sub(2),
            height: 1,
        };
        let status = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(status, status_area);
    }
}

fn draw_edit(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let Some(edit) = &state.edit else { return };

    let title = if state.screen == Screen::AddFriend {
        " Add Friend  [tab]next [enter]save [esc]cancel "
    } else {
        " Edit Friend  [tab]next [enter]save [esc]cancel "
    };

    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let field_height = 3u16;
    let visible_count = (inner.height / field_height) as usize;
    let start = if edit.cursor >= visible_count {
        edit.cursor - visible_count + 1
    } else {
        0
    };

    for (i, row) in edit.rows.iter().enumerate().skip(start).take(visible_count) {
        let y = inner.y + ((i - start) as u16) * field_height;
        if y + field_height > inner.y + inner.height {
            break;
        }

        let field_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: field_height,
        };

        let is_active = i == edit.cursor;
        let border_color = if is_active { Color::Cyan } else { Color::DarkGray };
        let label = match &row.hint {
            Some(hint) => format!(" {} ({hint}) ", row.label),
            None => format!(" {} ", row.label),
        };
        let block = Block::default()
            .title(label)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let content = Paragraph::new(row.value.as_str())
            .block(block)
            .style(Style::default().fg(if is_active { Color::White } else { Color::Gray }));
        f.render_widget(content, field_area);

        if is_active {
            f.set_cursor_position((
                field_area.x + 1 + row.value.len() as u16,
                field_area.y + 1,
            ));
        }
    }
}

fn draw_compose(f: &mut Frame, state: &AppState, _applet_key: String) {
    let area = f.area();
    let friend_name = state.current_friend().map(|fr| fr.name.as_str()).unwrap_or("");
    let title = format!(" Send Message: {friend_name}  [enter]send [esc]back ");

    let block = Block::default().title(title).borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let field_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: 3,
    };
    let field_block = Block::default()
        .title(" Message ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let content = Paragraph::new(state.compose_draft.as_str())
        .block(field_block)
        .style(Style::default().fg(Color::White));
    f.render_widget(content, field_area);

    f.set_cursor_position((
        field_area.x + 1 + state.compose_draft.len() as u16,
        field_area.y + 1,
    ));

    if inner.height > 4 {
        let hint_area = Rect {
            x: inner.x,
            y: inner.y + 3,
            width: inner.width,
            height: 1,
        };
        let hint = Paragraph::new("Edit the message above, then press [enter] to send.")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(hint, hint_area);
    }
}

fn draw_applet(f: &mut Frame, state: &AppState, key: String) {
    let area = f.area();
    let applet_name = state.registry.by_key(&key).map(|a| a.name()).unwrap_or("Applet");
    let friend_name = state.current_friend().map(|fr| fr.name.as_str()).unwrap_or("");
    let compose_hint = if key == "discord" { " [c]compose" } else { "" };
    let title = format!(" {applet_name}: {friend_name}  [esc]back [r]refresh{compose_hint} [enter]open URL ");

    match &state.applet_data {
        LoadState::Loading => {
            let block = Block::default().title(title).borders(Borders::ALL);
            let p = Paragraph::new("Loading...").block(block);
            f.render_widget(p, area);
        }
        LoadState::Error(e) => {
            let block = Block::default().title(title).borders(Borders::ALL);
            let p = Paragraph::new(format!("Error: {e}")).block(block).wrap(Wrap { trim: true });
            f.render_widget(p, area);
        }
        LoadState::Idle => {
            let block = Block::default().title(title).borders(Borders::ALL);
            let p = Paragraph::new("Press [r] to fetch.").block(block);
            f.render_widget(p, area);
        }
        LoadState::Loaded(items) => {
            if items.is_empty() {
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new("No results found.").block(block);
                f.render_widget(p, area);
                return;
            }

            let list_items: Vec<ListItem> = items
                .iter()
                .map(|item| {
                    let mut lines = vec![Line::from(vec![Span::styled(
                        item.title.clone(),
                        Style::default().add_modifier(Modifier::BOLD),
                    )])];
                    if let Some(sub) = &item.subtitle {
                        lines.push(Line::from(vec![Span::styled(
                            format!("  {}", truncate(sub, 90)),
                            Style::default().fg(Color::Gray),
                        )]));
                    }
                    ListItem::new(lines)
                })
                .collect();

            let mut list_state = ListState::default();
            list_state.select(Some(state.applet_scroll));

            let block = Block::default().title(title).borders(Borders::ALL);
            let list = List::new(list_items)
                .block(block)
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("> ");
            f.render_stateful_widget(list, area, &mut list_state);
        }
    }
}

fn draw_confirm_delete(f: &mut Frame, state: &AppState) {
    let name = state
        .current_friend()
        .map(|fr| fr.name.as_str())
        .unwrap_or("this friend");
    let area = centered_rect(50, 20, f.area());
    let block = Block::default()
        .title(" Confirm Delete ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    let text = format!("Delete {name}?\n\n[y] yes    [n] no");
    let p = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
