pub mod app;
pub mod event;
pub mod ui;
pub mod actions;
pub mod widgets;

use std::io;

use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::client::NexaClient;
use app::App;
use event::EventHandler;

pub async fn run(client: NexaClient) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, client).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    client: NexaClient,
) -> anyhow::Result<()> {
    let mut app = App::new(client);
    let mut events = EventHandler::new(std::time::Duration::from_secs(2));

    app.refresh().await;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        match events.next().await? {
            event::AppEvent::Tick => {
                app.refresh().await;
            }
            event::AppEvent::Key(key) => {
                if actions::handle_key(&mut app, key).await? {
                    return Ok(());
                }
            }
        }
    }
}
