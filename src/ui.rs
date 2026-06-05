use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::app::{AppState, LoadState, Screen};
use crate::apps::{available_applets, AppletKind};
use crate::api::wikipedia::WikiSummary;

pub fn draw(f: &mut Frame, state: &AppState) {
    match &state.screen {
        Screen::FriendsList | Screen::FriendDetail => draw_main(f, state),
        Screen::EditFriend | Screen::AddFriend => draw_edit(f, state),
        Screen::AppletView(kind) => draw_applet(f, state, kind.clone()),
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

    let applets = available_applets(friend);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(applets.len() as u16 + 4)])
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
    if !friend.interests.is_empty() {
        lines.push(Line::from(format!("interests: {}", friend.interests.join(", "))));
    }
    if let Some(jh) = &friend.job_hunt {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled("Job Hunt", Style::default().add_modifier(Modifier::UNDERLINED))]));
        lines.push(Line::from(format!("  role:   {}", jh.desired_role)));
        if !jh.skills.is_empty() {
            lines.push(Line::from(format!("  skills: {}", jh.skills.join(", "))));
        }
        if let Some(loc) = &jh.location {
            lines.push(Line::from(format!("  loc:    {loc}")));
        }
    }
    if let Some(bp) = &friend.book_profile {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled("Books", Style::default().add_modifier(Modifier::UNDERLINED))]));
        if !bp.genres.is_empty() {
            lines.push(Line::from(format!("  genres:  {}", bp.genres.join(", "))));
        }
        if !bp.authors.is_empty() {
            lines.push(Line::from(format!("  authors: {}", bp.authors.join(", "))));
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
    if applets.is_empty() {
        let block = Block::default()
            .title(" applets ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        let p = Paragraph::new("Fill in fields to unlock applets.")
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, chunks[1]);
    } else {
        let items: Vec<ListItem> = applets
            .iter()
            .map(|a| ListItem::new(a.label()))
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
    let fields = &edit.fields;
    let visible_count = (inner.height / field_height) as usize;
    let start = if edit.cursor >= visible_count {
        edit.cursor - visible_count + 1
    } else {
        0
    };

    for (i, (field, value)) in fields.iter().enumerate().skip(start).take(visible_count) {
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
        let block = Block::default()
            .title(format!(" {} ", field.label()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let content = Paragraph::new(value.as_str())
            .block(block)
            .style(Style::default().fg(if is_active { Color::White } else { Color::Gray }));
        f.render_widget(content, field_area);

        if is_active {
            f.set_cursor_position((
                field_area.x + 1 + value.len() as u16,
                field_area.y + 1,
            ));
        }
    }
}

fn draw_applet(f: &mut Frame, state: &AppState, kind: AppletKind) {
    let area = f.area();
    match kind {
        AppletKind::JobFeed => draw_job_feed(f, state, area),
        AppletKind::BookRecs => draw_book_recs(f, state, area),
        AppletKind::WikiPrep => draw_wiki_prep(f, state, area),
    }
}

fn draw_job_feed(f: &mut Frame, state: &AppState, area: Rect) {
    let friend_name = state.current_friend().map(|fr| fr.name.as_str()).unwrap_or("");
    let role = state
        .current_friend()
        .and_then(|fr| fr.job_hunt.as_ref())
        .map(|jh| jh.desired_role.as_str())
        .unwrap_or("");

    let title = format!(" Job Feed: {friend_name} ({role})  [esc]back [r]refresh [enter]open URL ");

    match &state.jobs {
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
            let p = Paragraph::new("Press [r] to fetch jobs.").block(block);
            f.render_widget(p, area);
        }
        LoadState::Loaded(jobs) => {
            if jobs.is_empty() {
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new("No jobs found.").block(block);
                f.render_widget(p, area);
                return;
            }

            let items: Vec<ListItem> = jobs
                .iter()
                .map(|j| {
                    let loc = if j.candidate_required_location.is_empty() {
                        "Remote".to_string()
                    } else {
                        j.candidate_required_location.clone()
                    };
                    let salary = if j.salary.is_empty() {
                        String::new()
                    } else {
                        format!("  {}", j.salary)
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:<45}", truncate(&j.title, 44)),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!("{:<20}", truncate(&j.company_name, 19))),
                        Span::styled(
                            format!("{:<15}", truncate(&loc, 14)),
                            Style::default().fg(Color::Green),
                        ),
                        Span::styled(salary, Style::default().fg(Color::Yellow)),
                    ]))
                })
                .collect();

            let mut list_state = ListState::default();
            list_state.select(Some(state.applet_scroll));

            let block = Block::default().title(title).borders(Borders::ALL);
            let list = List::new(items)
                .block(block)
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("> ");
            f.render_stateful_widget(list, area, &mut list_state);
        }
    }
}

fn draw_book_recs(f: &mut Frame, state: &AppState, area: Rect) {
    let friend_name = state.current_friend().map(|fr| fr.name.as_str()).unwrap_or("");
    let title = format!(" Book Recs: {friend_name}  [esc]back [r]refresh [enter]open page ");

    match &state.books {
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
            let p = Paragraph::new("Press [r] to fetch books.").block(block);
            f.render_widget(p, area);
        }
        LoadState::Loaded(books) => {
            if books.is_empty() {
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new("No books found.").block(block);
                f.render_widget(p, area);
                return;
            }

            let items: Vec<ListItem> = books
                .iter()
                .map(|b| {
                    let authors = b.authors.join(", ");
                    let year = b
                        .first_publish_year
                        .map(|y| format!(" ({y})"))
                        .unwrap_or_default();
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:<50}", truncate(&b.title, 49)),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!("{:<35}", truncate(&authors, 34))),
                        Span::styled(year, Style::default().fg(Color::DarkGray)),
                    ]))
                })
                .collect();

            let mut list_state = ListState::default();
            list_state.select(Some(state.applet_scroll));

            let block = Block::default().title(title).borders(Borders::ALL);
            let list = List::new(items)
                .block(block)
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("> ");
            f.render_stateful_widget(list, area, &mut list_state);
        }
    }
}

fn draw_wiki_prep(f: &mut Frame, state: &AppState, area: Rect) {
    let friend_name = state.current_friend().map(|fr| fr.name.as_str()).unwrap_or("");
    let title = format!(" Conversation Prep: {friend_name}  [esc]back [r]refresh [j/k]scroll ");

    match &state.wiki {
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
            let p = Paragraph::new("Press [r] to fetch summaries.").block(block);
            f.render_widget(p, area);
        }
        LoadState::Loaded(summaries) => {
            if summaries.is_empty() {
                let block = Block::default().title(title).borders(Borders::ALL);
                let p = Paragraph::new("No summaries found.").block(block);
                f.render_widget(p, area);
                return;
            }

            let lines = render_wiki_summaries(summaries);
            let block = Block::default().title(title).borders(Borders::ALL);
            let p = Paragraph::new(Text::from(lines))
                .block(block)
                .wrap(Wrap { trim: false })
                .scroll((state.applet_scroll as u16, 0));
            f.render_widget(p, area);
        }
    }
}

fn render_wiki_summaries(summaries: &[WikiSummary]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for summary in summaries {
        lines.push(Line::from(vec![Span::styled(
            summary.title.clone(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        // Word-wrap the extract into lines of ~80 chars
        for para in summary.extract.split('\n') {
            if para.is_empty() {
                lines.push(Line::from(""));
            } else {
                lines.push(Line::from(para.to_string()));
            }
        }
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "─".repeat(60),
            Style::default().fg(Color::DarkGray),
        )]));
        lines.push(Line::from(""));
    }
    lines
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
