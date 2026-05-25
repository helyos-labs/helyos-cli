use crossterm::event::{KeyCode, KeyEvent};

use super::app::{ActivePanel, App, InputMode};

pub async fn handle_key(app: &mut App, key: KeyEvent) -> anyhow::Result<bool> {
    match &app.input_mode {
        InputMode::ConfirmDelete(_) => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    app.delete_selected_pod().await;
                }
                _ => {
                    app.input_mode = InputMode::Normal;
                    app.status_message = None;
                }
            }
            Ok(false)
        }
        InputMode::ScaleInput(deployment, input) => {
            let deployment = deployment.clone();
            let mut input = input.clone();
            match key.code {
                KeyCode::Enter => {
                    app.scale_deployment(&input).await;
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    input.push(c);
                    app.input_mode = InputMode::ScaleInput(deployment, input);
                }
                KeyCode::Backspace => {
                    input.pop();
                    app.input_mode = InputMode::ScaleInput(deployment, input);
                }
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                    app.status_message = None;
                }
                _ => {}
            }
            Ok(false)
        }
        InputMode::LogView(_, _) => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
                Ok(false)
            }
            _ => Ok(false),
        },
        InputMode::Normal => match key.code {
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
            KeyCode::Char('d') => {
                if app.active_panel == ActivePanel::Pods {
                    if let Some(pod) = app.selected_pod() {
                        let name = pod.container_name();
                        app.status_message = Some(format!("Delete {name}? [y/N]"));
                        app.input_mode = InputMode::ConfirmDelete(name);
                    }
                }
                Ok(false)
            }
            KeyCode::Char('s') => {
                if app.active_panel == ActivePanel::Pods {
                    if let Some(pod) = app.selected_pod() {
                        let deployment = pod.deployment_name.clone();
                        app.status_message = Some(format!("Scale {deployment} — Replicas: "));
                        app.input_mode = InputMode::ScaleInput(deployment, String::new());
                    }
                }
                Ok(false)
            }
            KeyCode::Char('l') | KeyCode::Enter => {
                if app.active_panel == ActivePanel::Pods && !app.pods.is_empty() {
                    app.open_log_view().await;
                }
                Ok(false)
            }
            _ => Ok(false),
        },
    }
}
