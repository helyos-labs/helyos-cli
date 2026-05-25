use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use crate::tui::app::App;

pub fn pod_status_color(status: &str) -> Color {
    match status.to_lowercase().as_str() {
        "running" => Color::Rgb(63, 185, 80),
        "pending" | "creating" => Color::Rgb(86, 212, 221),
        "restarting" | "degraded" => Color::Rgb(210, 153, 34),
        "failed" | "crashloopbackoff" => Color::Rgb(248, 81, 73),
        "stopped" | "stopping" => Color::Rgb(72, 79, 88),
        _ => Color::Rgb(230, 237, 243),
    }
}

pub fn build_table(app: &App, is_active: bool) -> Table<'static> {
    let border_color = if is_active {
        Color::Rgb(88, 166, 255)
    } else {
        Color::Rgb(48, 54, 61)
    };

    let header = Row::new(vec![
        Cell::from("NAME"),
        Cell::from("STATUS"),
        Cell::from("CPU"),
        Cell::from("MEM"),
        Cell::from("RESTARTS"),
        Cell::from("AGE"),
    ])
    .style(Style::default().fg(Color::Rgb(72, 79, 88)));

    let rows: Vec<Row> = app
        .pods
        .iter()
        .enumerate()
        .map(|(i, pod)| {
            let status_str = pod.status.to_string();
            let status_color = pod_status_color(&status_str);
            let selected = i == app.pod_cursor;

            let row = Row::new(vec![
                Cell::from(pod.container_name()),
                Cell::from(format!("● {}", status_str.to_lowercase()))
                    .style(Style::default().fg(status_color)),
                Cell::from("—"),
                Cell::from("—"),
                Cell::from(pod.restart_count.to_string()),
                Cell::from(crate::output::format_age(&pod.created_at)),
            ]);

            if selected && is_active {
                row.style(
                    Style::default()
                        .bg(Color::Rgb(22, 27, 34))
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                row
            }
        })
        .collect();

    let count = format!("{} total", app.pods.len());
    let title = format!(" ● Pods  {count} ");

    Table::new(
        rows,
        [
            Constraint::Min(20),
            Constraint::Length(16),
            Constraint::Length(5),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(6),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title)
            .title_style(Style::default().fg(Color::Rgb(88, 166, 255))),
    )
    .row_highlight_style(Style::default().bg(Color::Rgb(22, 27, 34)))
}
