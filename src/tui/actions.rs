use crossterm::event::{KeyCode, KeyEvent};
use super::app::App;

pub async fn handle_key(app: &mut App, key: KeyEvent) -> anyhow::Result<bool> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Ok(true),
        KeyCode::Tab => {
            app.next_panel();
            Ok(false)
        }
        KeyCode::BackTab => {
            app.prev_panel();
            Ok(false)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.cursor_up();
            Ok(false)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.cursor_down();
            Ok(false)
        }
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
            Ok(false)
        }
        _ => Ok(false),
    }
}
