use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::app::App;

fn gauge_color(percent: f64) -> Color {
    if percent >= 80.0 {
        Color::Rgb(248, 81, 73)
    } else if percent >= 60.0 {
        Color::Rgb(210, 153, 34)
    } else {
        Color::Rgb(63, 185, 80)
    }
}

fn gauge_line(label: &str, percent: f64, width: u16) -> Line<'static> {
    let bar_width = (width as usize).saturating_sub(12);
    let filled = ((percent / 100.0) * bar_width as f64) as usize;
    let empty = bar_width.saturating_sub(filled);
    let color = gauge_color(percent);

    Line::from(vec![
        Span::styled(
            format!("{label:>3} "),
            Style::default().fg(Color::Rgb(139, 148, 158)),
        ),
        Span::styled("█".repeat(filled), Style::default().fg(color)),
        Span::styled(
            "░".repeat(empty),
            Style::default().fg(Color::Rgb(33, 38, 45)),
        ),
        Span::styled(
            format!(" {:>3.0}%", percent),
            Style::default().fg(Color::Rgb(139, 148, 158)),
        ),
    ])
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
        .title(" ◆ Nodes ")
        .title_style(Style::default().fg(Color::Rgb(88, 166, 255)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.nodes.is_empty() {
        let msg = Paragraph::new("No node data").style(Style::default().fg(Color::Rgb(72, 79, 88)));
        f.render_widget(msg, inner);
        return;
    }

    let node_height = 3u16;
    let constraints: Vec<Constraint> = app
        .nodes
        .iter()
        .map(|_| Constraint::Length(node_height))
        .chain(std::iter::once(Constraint::Min(0)))
        .collect();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, node) in app.nodes.iter().enumerate() {
        if i >= chunks.len() - 1 {
            break;
        }
        let area = chunks[i];
        let mem_pct = if node.memory_total_bytes > 0 {
            node.memory_used_bytes as f64 / node.memory_total_bytes as f64 * 100.0
        } else {
            0.0
        };

        let lines = vec![
            Line::from(vec![
                Span::styled(
                    node.name.clone(),
                    Style::default().fg(Color::Rgb(230, 237, 243)),
                ),
                Span::styled(
                    format!("  {} · {} pods", node.role, node.pod_count),
                    Style::default().fg(Color::Rgb(139, 148, 158)),
                ),
            ]),
            gauge_line("cpu", node.cpu_usage_percent, area.width),
            gauge_line("mem", mem_pct, area.width),
        ];

        let para = Paragraph::new(lines);
        f.render_widget(para, area);
    }
}
