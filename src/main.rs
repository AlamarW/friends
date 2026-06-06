mod api;
mod app;
mod apps;
mod data;
mod events;
mod ui;
#[cfg(test)]
mod tests;

use std::io;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

use app::{AppState, LoadState};
use apps::{AppletItem, build_registry};
use data::storage::load_friends;
use events::{EventAction, handle_key};

#[tokio::main]
async fn main() -> Result<()> {
    let friends = load_friends().unwrap_or_default();
    let registry = Arc::new(build_registry());
    let mut state = AppState::new(friends, registry);

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
    type FetchResult = (String, Result<Vec<AppletItem>, String>);
    let (tx, mut rx) = mpsc::channel::<FetchResult>(8);

    loop {
        terminal.draw(|f| ui::draw(f, state))?;

        while let Ok((key, result)) = rx.try_recv() {
            apply_fetch_result(state, key, result);
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let action = handle_key(state, key);
                match action {
                    EventAction::Quit => break,
                    EventAction::Fetch(applet_key) => {
                        let profile = state
                            .current_friend()
                            .map(|fr| fr.profile_for(&applet_key));
                        let registry = state.registry.clone();
                        if let Some(profile) = profile {
                            let key = applet_key;
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                let result = match registry.by_key(&key) {
                                    Some(applet) => applet
                                        .fetch(&profile)
                                        .await
                                        .map_err(|e| e.to_string()),
                                    None => Err(format!("Unknown applet: {key}")),
                                };
                                let _ = tx.send((key, result)).await;
                            });
                        }
                    }
                    EventAction::Send(applet_key, message) => {
                        let profile = state
                            .current_friend()
                            .map(|fr| fr.profile_for(&applet_key));
                        if let Some(profile) = profile {
                            let key = applet_key;
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                let result = crate::api::discord::send_dm(&profile, &message)
                                    .await
                                    .map_err(|e| e.to_string());
                                let _ = tx.send((key, result)).await;
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

fn apply_fetch_result(
    state: &mut AppState,
    key: String,
    result: Result<Vec<AppletItem>, String>,
) {
    if state.active_applet_key.as_deref() != Some(&key) {
        return;
    }
    state.applet_data = match result {
        Ok(items) => LoadState::Loaded(items),
        Err(e) => LoadState::Error(e),
    };
}
