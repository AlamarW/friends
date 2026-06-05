mod api;
mod app;
mod apps;
mod data;
mod events;
mod ui;
#[cfg(test)]
mod tests;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

use app::{AppState, LoadState, Screen};
use data::storage::load_friends;
use events::{EventAction, handle_key};

#[tokio::main]
async fn main() -> Result<()> {
    let friends = load_friends().unwrap_or_default();
    let mut state = AppState::new(friends);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &mut state).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run<B>(terminal: &mut Terminal<B>, state: &mut AppState) -> Result<()>
where
    B: ratatui::backend::Backend,
    B::Error: Send + Sync + 'static,
{
    let (tx, mut rx) = mpsc::channel::<FetchResult>(8);

    loop {
        terminal.draw(|f| ui::draw(f, state))?;

        // Drain any fetch results first
        while let Ok(result) = rx.try_recv() {
            apply_fetch_result(state, result);
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let action = handle_key(state, key);
                match action {
                    EventAction::Quit => break,
                    EventAction::FetchJobs => {
                        if let Some(friend) = state.current_friend() {
                            if let Some(profile) = friend.job_hunt.clone() {
                                let tx = tx.clone();
                                tokio::spawn(async move {
                                    let result = api::remotive::fetch_jobs(&profile).await;
                                    let _ = tx.send(FetchResult::Jobs(result.map_err(|e| e.to_string()))).await;
                                });
                            }
                        }
                    }
                    EventAction::FetchBooks => {
                        if let Some(friend) = state.current_friend() {
                            if let Some(profile) = friend.book_profile.clone() {
                                let tx = tx.clone();
                                tokio::spawn(async move {
                                    let result = api::open_library::fetch_books(&profile).await;
                                    let _ = tx.send(FetchResult::Books(result.map_err(|e| e.to_string()))).await;
                                });
                            }
                        }
                    }
                    EventAction::FetchWiki => {
                        if let Some(friend) = state.current_friend() {
                            let interests = friend.interests.clone();
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                let result = api::wikipedia::fetch_summaries(&interests).await;
                                let _ = tx.send(FetchResult::Wiki(result.map_err(|e| e.to_string()))).await;
                            });
                        }
                    }
                    EventAction::None => {}
                }
            }
        }
    }

    Ok(())
}

enum FetchResult {
    Jobs(Result<Vec<api::remotive::Job>, String>),
    Books(Result<Vec<api::open_library::Book>, String>),
    Wiki(Result<Vec<api::wikipedia::WikiSummary>, String>),
}

fn apply_fetch_result(state: &mut AppState, result: FetchResult) {
    match result {
        FetchResult::Jobs(Ok(jobs)) => state.jobs = LoadState::Loaded(jobs),
        FetchResult::Jobs(Err(e)) => state.jobs = LoadState::Error(e),
        FetchResult::Books(Ok(books)) => state.books = LoadState::Loaded(books),
        FetchResult::Books(Err(e)) => state.books = LoadState::Error(e),
        FetchResult::Wiki(Ok(summaries)) => state.wiki = LoadState::Loaded(summaries),
        FetchResult::Wiki(Err(e)) => state.wiki = LoadState::Error(e),
    }
    if matches!(&state.screen, Screen::AppletView(_)) {}
}
