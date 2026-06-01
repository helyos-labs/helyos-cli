pub mod actions;
pub mod app;
pub mod event;
pub mod ui;
pub mod widgets;

use std::io;

use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::client::NexaClient;
use app::App;
use event::EventHandler;

pub async fn run(client: NexaClient, server_url: &str, token: Option<&str>) -> anyhow::Result<()> {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, client, server_url, token).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    client: NexaClient,
    server_url: &str,
    token: Option<&str>,
) -> anyhow::Result<()> {
    let mut app = App::new(client);
    let mut events = EventHandler::new(std::time::Duration::from_secs(2), server_url, token);

    app.refresh().await;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        match events.next().await? {
            event::AppEvent::Tick => {
                app.clear_stale_status();
                app.refresh().await;
            }
            event::AppEvent::Key(key) => {
                if actions::handle_key(&mut app, key).await? {
                    return Ok(());
                }
            }
            event::AppEvent::ClusterEvent(event) => {
                app.push_event(event);
            }
        }
    }
}
