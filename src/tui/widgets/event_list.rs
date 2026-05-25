use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::app::App;

fn action_color(action: &str) -> Color {
    match action.to_lowercase().as_str() {
        "started" | "deployed" | "scaled" => Color::Rgb(63, 185, 80),
        "pending" | "pulling" => Color::Rgb(210, 153, 34),
        "died" | "oomkilled" | "failed" => Color::Rgb(248, 81, 73),
        _ => Color::Rgb(139, 148, 158),
    }
}

pub fn render(f: &mut Frame, area: Rect, app: &App, is_active: bool) {
    let border_color = if is_active {
        Color::Rgb(88, 166, 255)
    } else {
        Color::Rgb(48, 54, 61)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" • Events ")
        .title_style(Style::default().fg(Color::Rgb(88, 166, 255)));

    if app.events.is_empty() {
        let msg = Paragraph::new("  No events yet")
            .style(Style::default().fg(Color::Rgb(72, 79, 88)))
            .block(block);
        f.render_widget(msg, area);
        return;
    }

    let lines: Vec<Line> = app
        .events
        .iter()
        .rev()
        .take((area.height as usize).saturating_sub(2))
        .map(|e| {
            let time = if e.timestamp.len() >= 19 {
                &e.timestamp[11..19]
            } else {
                &e.timestamp
            };
            let color = action_color(&e.action);
            Line::from(vec![
                Span::styled(format!("{time}  "), Style::default().fg(color)),
                Span::styled(
                    format!("{:<8}", e.kind),
                    Style::default().fg(Color::Rgb(139, 148, 158)),
                ),
                Span::styled(
                    e.name.clone(),
                    Style::default().fg(Color::Rgb(230, 237, 243)),
                ),
                Span::styled(format!(" {}", e.action), Style::default().fg(color)),
            ])
        })
        .collect();

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, area);
}
